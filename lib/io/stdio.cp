/**
 * CP Standard Console and Stdio (cap:io)
 * 标准控制台与终端流输入输出
 */

export class Console {
    pub fn log(this, fmt: string, a1: any, a2: any): void {
        if (@is_beam) {
            @print(fmt, a1, a2);
        } else {
            @io_write_stdout(fmt, a1, a2);
        }
    }

    pub fn err(this, fmt: string, a1: any): void {
        if (@is_beam) {
            @print_err(fmt, a1);
        } else {
            @io_write_stderr(fmt, a1);
        }
    }
}

export const console = new Console();

export class Stdio {
    pub fn readLine(this): string {
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

export fn println(msg: any): void {
    @print("{any}\n", msg);
}

export fn eprint(msg: any): void {
    @print_err("{any}", msg);
}

export fn eprintln(msg: any): void {
    @print_err("{any}\n", msg);
}
