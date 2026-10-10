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
    let c = a;  // Should cause compile error: Use of moved value 'a'
}

fn main() {
    syncFunc();
}
