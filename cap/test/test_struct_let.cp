import { print } from "cap:io";

const S = struct {
    pub fn testFn(): i32 {
        let mut x: i32 = 1;
        x = x + 1;
        return x;
    }
};

pub fn main(): void {
    let res: i32 = S.testFn();
    if (res != 2) {
        print("Test failed: res != 2\n");
        return;
    }
    print("test_struct_let passed!\n");
}
