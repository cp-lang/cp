import { print } from "cap:io";

class MockTensor {
    data: i32;

    fn init(d: i32) {
        this.data = d;
    }
    
    fn release() {
        @print("MockTensor release called!");
    }
}

fn syncFunc() {
    let a = new MockTensor(42);
    let b = a;  // Move ownership
    // b goes out of scope here and should be released.
}

fn main() {
    syncFunc();
}
