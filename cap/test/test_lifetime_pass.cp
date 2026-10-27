import { print } from "cap:io";

class Tensor {
    size: i32;

    fn init(s: i32) {
        this.size = s;
    }

    fn release() {
        @print("Tensor(size={d}) released!", this.size);
    }
}

// 借鉴 C# 的 ref struct：零开销借用局部数据，无需 <'a> 标注！
ref struct TensorView {
    target: *Tensor;
    offset: i32;

    fn init(t: *Tensor, off: i32) {
        this.target = t;
        this.offset = off;
    }

    fn addOffset() {
        this.target.size = this.target.size + this.offset;
    }
}

// 第 1 级：单参自动省略生命周期
fn borrowSingle(t: Tensor) -> &Tensor {
    return &t;
}

// 第 2 级：双参默认交集推导（无需写 <'a>）
fn pickFirstDefault(a: Tensor, b: Tensor) -> &Tensor {
    return &a;
}

// 第 3 级：极简参数名绑定（&x Tensor，表明返回值仅借自 x，与 y 无关！）
fn selectFirstExplicit(x: Tensor, y: Tensor) -> &x Tensor {
    return &x;
}

fn testLifetimesAndRefStruct() {
    // 1. 测试 NLL (非词法生命周期)：借用在最后一次使用后自动结束，随后可安全 Move
    let t1 = new Tensor(10);
    let r1 = &t1;
    @print("NLL borrow read: {d}", r1.size); // r1 最后一次使用！

    // 2. 测试 ref struct (栈绑定视图) 零开销原地修改局部数据 (*t1 可变借用)
    let view = new TensorView(*t1, 5);
    view.addOffset();
    @print("After ref struct mutation: {d}", t1.size); // view 最后一次使用！

    // 此时 r1 和 view 的生命周期均已通过 NLL 宣告结束，t1 可以安全 Move 给 t2！
    let t2 = t1;
    @print("Moved to t2 successfully: {d}", t2.size);

    // 3. 测试第 3 级参数名绑定 (&x Tensor)：内层 y 提前释放，外层依然能安全使用借自 x 的返回值
    let kept: &Tensor;
    {
        let tempY = new Tensor(99);
        kept = selectFirstExplicit(t2, tempY);
    } // tempY 在此处自动触发 release()，但 kept 仅绑定到 t2，完全合法！
    @print("Explicit &x lifetime survived inner scope: {d}", kept.size);
}

fn main() {
    testLifetimesAndRefStruct();
}
