/**
 * CP Standard I/O Traits (cap:io)
 * 统一流式 Trait 契约与错误集
 */

export error IOError {
    EndOfStream,
    UnexpectedEof,
    Closed,
    BufferFull,
    ShortWrite,
    InvalidSeek,
}

export enum SeekOrigin {
    Start,
    Current,
    End,
}

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
