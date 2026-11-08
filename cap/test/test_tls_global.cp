import { test, expect_eq } from "cap:test";

@tls let temp_path_buf: [1024]u8 = undefined;
threadlocal let big_temp_path_buf: [16384]u8 = undefined;

fn main() {
    test("threadlocal storage buffer read/write", () => {
        temp_path_buf[0] = 42;
        temp_path_buf[1] = 99;
        expect_eq(temp_path_buf[0], 42);
        expect_eq(temp_path_buf[1], 99);

        big_temp_path_buf[0] = 7;
        big_temp_path_buf[16383] = 88;
        expect_eq(big_temp_path_buf[0], 7);
        expect_eq(big_temp_path_buf[16383], 88);
    });
}
