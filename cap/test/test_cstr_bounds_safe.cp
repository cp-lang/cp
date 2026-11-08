import { print } from "cap:io";

fn modify_safe(*buf: [10]u8): void {
    buf[2] = 42;
}

fn main(): void {
    let host_buf: [10]u8 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    modify_safe(&host_buf);

    let s: cstring = "hello";
    let c = s[1];
    print(`char: ${c}`);

    let slice = host_buf[0..5];
    slice[1] = 88;
    print(`slice[1]: ${slice[1]}`);

    unsafe {
        let ptr: [*]u8! = &host_buf[0];
        ptr[0] = 100;
    }
    print(`host_buf[0]: ${host_buf[0]}`);
}
