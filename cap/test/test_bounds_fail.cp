import { print } from "cap:io";

fn main(): void {
    let host_buf: [5]u8 = [1, 2, 3, 4, 5];
    let s: cstr = &host_buf[0];
    s[10] = 99;
    print("fail\n");
}
