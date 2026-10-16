import { print } from "cap:io";

class Tensor {
    size: i32;
    fn init(s: i32) { this.size = s; }
    fn release() { @print("Tensor released"); }
}

fn badDanglingScope() {
    let outerRef: &Tensor; // depth 1
    {
        let innerTensor = new Tensor(42); // depth 2
        outerRef = &innerTensor; // ❌ Error: Borrowed value 'innerTensor' does not live long enough
    }
    @print("Read dangling: {d}", outerRef.size);
}

fn main() {
    badDanglingScope();
}
