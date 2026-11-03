import { print } from "cap:io";

struct BufferHelper {
    len: usize = 0;

    @inline
    fn inc(*_, amount: usize): void {
        len = len + amount;
    }

    @inline
    fn getLen(_): usize {
        return len;
    }
}

fn main(): void {
    let buf = BufferHelper{ len: 5 };
    buf.inc(10);
    if (buf.getLen() != 15) {
        print("Root inline test failed: expected 15, got {d}\n", [buf.getLen()]);
        return;
    }
    print("Root and inline feature tests passed successfully!\n");
}
