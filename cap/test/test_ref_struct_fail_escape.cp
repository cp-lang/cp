import { print } from "cap:io";

class Tensor {
    size: i32;
    fn init(s: i32) { this.size = s; }
    fn release() { @print("Tensor released"); }
}

// 栈绑定视图 (C# ref struct)
ref struct TensorView {
    target: &Tensor;
    fn init(t: &Tensor) { this.target = t; }
}

fn badRefStructEscape() -> TensorView {
    let localT = new Tensor(100);
    let view = new TensorView(&localT);
    // ❌ 试图将借用了局部变量的 ref struct 返回逃逸出函数！
    return view;
}

fn main() {
    let v = badRefStructEscape();
}
