/**
 * CP Standard I/O (cap:io)
 * 使用 @is_beam 区分 BEAM 虚拟终端与 Native 标准流
 */

export class Console {
    /**
     * 基础日志打印
     */
    fn log(this, fmt: string, a1: any, a2: any): void {
        if (@is_beam) {
            // BEAM 模式下，@print 已经被编译器映射为高效的 NIF 打印
            @print(fmt, a1, a2);
        } else {
            // 原生模式下，直接调用底层 stdout 驱动
            @io_write_stdout(fmt, a1, a2);
        }
    }

    /**
     * 错误输出
     */
    fn err(this, fmt: string, a1: any): void {
        if (@is_beam) {
            @print_err(fmt, a1);
        } else {
            @io_write_stderr(fmt, a1);
        }
    }
}

export const console = new Console();

/**
 * 低级标准流访问
 */
export class Stdio {
    /**
     * 同步读取一行输入
     */
    fn readLine(this): string {
        if (@is_beam) {
            // BEAM 模式下的 Stdin 通常由宿主通过消息转发
            // 这里我们调用一个特殊的委托指令
            return @yield_to_beam(90, "read_line"); 
        } else {
            return @io_read_stdin_line();
        }
    }
}

export const stdio = new Stdio();

/**
 * 高性能标准输出打印
 */
export fn print(msg: any): void {
    @print("{any}", msg);
}
