// 反向测试：for 循环缺少圆括号，验证 E0101 智能诊断提示
fn main(): void {
    let list = [1, 2, 3];
    for item of list {
        let x = item;
    }
}
