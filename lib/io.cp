/**
 * CP Standard I/O (cap:io)
 * 工业级统一高性能 I/O 流体系与标准终端
 */

// --- 核心流错误集 ---
export error IOError {
    EndOfStream,
    UnexpectedEof,
    Closed,
    BufferFull,
    ShortWrite,
    InvalidSeek,
}

// --- 寻址基准 ---
export enum SeekOrigin {
    Start,
    Current,
    End,
}

// --- 统一流式 Trait 契约 ---

/**
 * 统一读取器契约
 * 传入目标缓冲区切片，返回实际读取字节数，0 表示 EOF
 */
export trait Reader {
    fn read(mut _, *buf: []u8): usize!;
}

/**
 * 统一写入器契约
 * 传入只读数据切片，返回实际写入字节数
 */
export trait Writer {
    fn write(mut _, buf: []const u8): usize!;
}

/**
 * 统一资源关闭契约
 */
export trait Closer {
    fn close(mut _): void;
}

/**
 * 统一流寻址契约
 */
export trait Seeker {
    fn seek(mut _, offset: i64, origin: SeekOrigin): u64!;
}

// --- 内存切片与缓冲流 ---

/**
 * 内存只读流：直接在源切片上无堆分配零拷贝读取与寻址
 */
export class BufferReader implements Reader, Seeker {
    data: []const u8;
    pos: usize;

    pub fn init(data: []const u8) {
        _.data = data;
        _.pos = 0;
    }

    pub fn read(mut _, *buf: []u8): usize! {
        if (_.pos >= _.data.len) {
            return 0; // EOF
        }
        let available = _.data.len - _.pos;
        let to_copy = buf.len;
        if (to_copy > available) {
            to_copy = available;
        }
        let i: usize = 0;
        while (i < to_copy) {
            buf[i] = _.data[_.pos + i];
            i = i + 1;
        }
        _.pos = _.pos + to_copy;
        return to_copy;
    }

    pub fn seek(mut _, offset: i64, origin: SeekOrigin): u64! {
        let new_pos: i64 = 0;
        match (origin) {
            SeekOrigin.Start => {
                new_pos = offset;
            },
            SeekOrigin.Current => {
                new_pos = (_.pos as i64) + offset;
            },
            SeekOrigin.End => {
                new_pos = (_.data.len as i64) + offset;
            },
        }
        if (new_pos < 0 || (new_pos as usize) > _.data.len) {
            return err.InvalidSeek;
        }
        _.pos = new_pos as usize;
        return _.pos as u64;
    }

    pub fn remaining(): usize {
        if (_.pos >= _.data.len) {
            return 0;
        }
        return _.data.len - _.pos;
    }

    pub fn reset(mut _) {
        _.pos = 0;
    }
}

/**
 * 内存写入器：栈/堆定长缓冲区写入流
 */
export class BufferWriter implements Writer {
    buf: [4096]u8;
    written: usize;

    pub fn init() {
        _.buf = [0 as u8; 4096];
        _.written = 0;
    }

    pub fn write(mut _, data: []const u8): usize! {
        let available: usize = (4096 as usize) - _.written;
        if (available == 0) {
            return err.BufferFull;
        }
        let to_write = data.len;
        if (to_write > available) {
            to_write = available;
        }
        let i: usize = 0;
        while (i < to_write) {
            _.buf[_.written + i] = data[i];
            i = i + 1;
        }
        _.written = _.written + to_write;
        return to_write;
    }

    pub fn bytes(): []const u8 {
        return _.buf[0.._.written];
    }

    pub fn len(): usize {
        return _.written;
    }

    pub fn reset(mut _) {
        _.written = 0;
    }
}

// --- 高性能带缓冲流 ---

/**
 * 带缓冲读取器：内部 256 字节缓存，消灭高频系统调用损耗
 */
export class BufferedReader<R: Reader> implements Reader {
    reader: *R;
    cache: [256]u8;
    cache_len: usize;
    cache_pos: usize;

    pub fn init(reader: *R) {
        _.reader = reader;
        _.cache = [0 as u8; 256];
        _.cache_len = 0;
        _.cache_pos = 0;
    }

    pub fn readByte(mut _): u8! {
        if (_.cache_pos >= _.cache_len) {
            let slice = _.cache[0..256];
            let n = _.reader.read(slice)?;
            if (n == 0) {
                return err.EndOfStream;
            }
            _.cache_len = n;
            _.cache_pos = 0;
        }
        let b = _.cache[_.cache_pos];
        _.cache_pos = _.cache_pos + 1;
        return b;
    }

    pub fn read(mut _, *buf: []u8): usize! {
        if (buf.len == 0) {
            return 0;
        }
        let out_idx: usize = 0;
        while (out_idx < buf.len) {
            if (_.cache_pos >= _.cache_len) {
                let slice = _.cache[0..256];
                let n = _.reader.read(slice)?;
                if (n == 0) {
                    break;
                }
                _.cache_len = n;
                _.cache_pos = 0;
            }
            let avail = _.cache_len - _.cache_pos;
            let needed = buf.len - out_idx;
            let to_take = needed;
            if (to_take > avail) {
                to_take = avail;
            }
            let i: usize = 0;
            while (i < to_take) {
                buf[out_idx + i] = _.cache[_.cache_pos + i];
                i = i + 1;
            }
            _.cache_pos = _.cache_pos + to_take;
            out_idx = out_idx + to_take;
        }
        return out_idx;
    }
}

/**
 * 带缓冲写入器：内部 256 字节缓存聚合写入与自动 flush
 */
export class BufferedWriter<W: Writer> implements Writer {
    writer: *W;
    cache: [256]u8;
    cache_len: usize;

    pub fn init(writer: *W) {
        _.writer = writer;
        _.cache = [0 as u8; 256];
        _.cache_len = 0;
    }

    pub fn flush(mut _): void! {
        if (_.cache_len > 0) {
            let slice = _.cache[0.._.cache_len];
            let _ = _.writer.write(slice)?;
            _.cache_len = 0;
        }
    }

    pub fn write(mut _, data: []const u8): usize! {
        let in_idx: usize = 0;
        while (in_idx < data.len) {
            let avail = (256 as usize) - _.cache_len;
            if (avail == 0) {
                flush()?;
                avail = 256 as usize;
            }
            let rem = data.len - in_idx;
            let to_copy = rem;
            if (to_copy > avail) {
                to_copy = avail;
            }
            let i: usize = 0;
            while (i < to_copy) {
                _.cache[_.cache_len + i] = data[in_idx + i];
                i = i + 1;
            }
            _.cache_len = _.cache_len + to_copy;
            in_idx = in_idx + to_copy;
        }
        return data.len;
    }
}

// --- 泛型流管道操作 ---

/**
 * 将数据从读取器全量流式写入到写入器直至 EOF
 * 返回累计传输的总字节数
 */
export fn copyStream<R: Reader, W: Writer>(mut r: R, mut w: W): usize! {
    let chunk_buf: [256]u8 = [0 as u8; 256];
    let slice = chunk_buf[0..256];
    let total: usize = 0;
    while (true) {
        let n = r.read(slice)?;
        if (n == 0) {
            break;
        }
        let written = w.write(chunk_buf[0..n])?;
        total = total + written;
    }
    return total;
}

/**
 * 精确从读取器复制 n 字节至写入器
 */
export fn copyN<R: Reader, W: Writer>(mut r: R, mut w: W, limit: usize): usize! {
    let chunk_buf: [256]u8 = [0 as u8; 256];
    let total: usize = 0;
    while (total < limit) {
        let needed = limit - total;
        let to_read = (256 as usize);
        if (needed < to_read) {
            to_read = needed;
        }
        let slice = chunk_buf[0..to_read];
        let n = r.read(slice)?;
        if (n == 0) {
            break;
        }
        let written = w.write(chunk_buf[0..n])?;
        total = total + written;
    }
    return total;
}

// --- 控制台与标准流 ---

export class Console {
    fn log(this, fmt: string, a1: any, a2: any): void {
        if (@is_beam) {
            @print(fmt, a1, a2);
        } else {
            @io_write_stdout(fmt, a1, a2);
        }
    }

    fn err(this, fmt: string, a1: any): void {
        if (@is_beam) {
            @print_err(fmt, a1);
        } else {
            @io_write_stderr(fmt, a1);
        }
    }
}

export const console = new Console();

export class Stdio {
    fn readLine(this): string {
        if (@is_beam) {
            return @yield_to_beam(90, "read_line"); 
        } else {
            return @io_read_stdin_line();
        }
    }
}

export const stdio = new Stdio();

export fn print(msg: any): void {
    @print("{any}", msg);
}

// --- 工业级管道流与缓冲区越界守卫 (Ported from Bun.js io/PipeReader & io/MaxBuf) ---

/**
 * 缓冲区最大字节限制守卫 (Ported from Bun.js MaxBuf)
 * 防止未受信进程或超大流耗尽内存
 */
export class MaxBuf {
    remaining_bytes: i64;

    pub fn init(limit: i64) {
        _.remaining_bytes = limit;
    }

    pub fn onReadBytes(mut _, bytes: usize): bool! {
        if (_.remaining_bytes < 0) {
            return false;
        }
        _.remaining_bytes = _.remaining_bytes - (bytes as i64);
        if (_.remaining_bytes < 0) {
            return err.BufferFull;
        }
        return true;
    }
}

/**
 * 原生非阻塞/阻塞管道读取器
 * 满足 Reader 与 Closer Trait 契约
 */
export class PipeReader implements Reader, Closer {
    fd: i32;
    closed: bool;

    pub fn init(fd: i32) {
        _.fd = fd;
        _.closed = false;
    }

    pub fn read(mut _, *buf: []u8): usize! {
        if (_.closed || _.fd < 0) {
            return err.Closed;
        }
        if (buf.len == 0) {
            return 0;
        }
        const rc = std.os.linux.read(_.fd, buf.ptr, buf.len);
        const err_code = std.posix.errno(rc);
        if (err_code != .SUCCESS) {
            if (err_code == .AGAIN) return 0;
            return err.EndOfStream;
        }
        return rc;
    }

    pub fn close(mut _): void {
        if (!_.closed && _.fd >= 0) {
            _ = std.os.linux.close(_.fd);
            _.closed = true;
            _.fd = -1;
        }
    }
}

/**
 * 原生非阻塞/阻塞管道写入器
 * 满足 Writer 与 Closer Trait 契约
 */
export class PipeWriter implements Writer, Closer {
    fd: i32;
    closed: bool;

    pub fn init(fd: i32) {
        _.fd = fd;
        _.closed = false;
    }

    pub fn write(mut _, data: []const u8): usize! {
        if (_.closed || _.fd < 0) {
            return err.Closed;
        }
        if (data.len == 0) {
            return 0;
        }
        const rc = std.os.linux.write(_.fd, data.ptr, data.len);
        const err_code = std.posix.errno(rc);
        if (err_code != .SUCCESS) {
            return err.ShortWrite;
        }
        return rc;
    }

    pub fn close(mut _): void {
        if (!_.closed && _.fd >= 0) {
            _ = std.os.linux.close(_.fd);
            _.closed = true;
            _.fd = -1;
        }
    }
}

/**
 * 管道工厂：创建一对相互连通的 PipeReader 与 PipeWriter
 */
export class Pipe {
    reader: PipeReader;
    writer: PipeWriter;

    pub fn init(reader: PipeReader, writer: PipeWriter) {
        _.reader = reader;
        _.writer = writer;
    }

    pub static fn create(): Pipe! {
        var fds: [2]std.posix.fd_t = undefined;
        const rc = std.os.linux.pipe(&fds);
        if (std.posix.errno(rc) != .SUCCESS) {
            return err.Closed;
        }
        let r = new PipeReader(fds[0]);
        let w = new PipeWriter(fds[1]);
        return new Pipe(r, w);
    }
}
