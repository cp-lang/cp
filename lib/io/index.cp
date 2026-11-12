/**
 * CP Standard I/O (cap:io)
 * 工业级统一高性能 I/O 流体系、管道与标准终端
 */

export {
    IOError,
    SeekOrigin,
    Reader,
    Writer,
    Closer,
    Seeker
} from "./traits";

export {
    BufferReader,
    BufferWriter,
    BufferedReader,
    BufferedWriter,
    copyStream,
    copyN
} from "./buffer_stream";

export {
    Console,
    console,
    Stdio,
    stdio,
    print,
    println,
    eprint,
    eprintln
} from "./stdio";

export {
    MaxBuf,
    MaxBufKind
} from "./max_buf";

export {
    PipeReader,
    PipeWriter,
    Pipe
} from "./pipe";
