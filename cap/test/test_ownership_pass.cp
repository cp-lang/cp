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
    // a goes out of scope and should automatically call release()
}

fn main() {
    syncFunc();
}
