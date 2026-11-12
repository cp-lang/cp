/**
 * CP Buffer Streams (cap:io)
 * 内存切片与缓冲流
 */

import { IOError, SeekOrigin, Reader, Writer, Closer, Seeker } from "./traits";

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
