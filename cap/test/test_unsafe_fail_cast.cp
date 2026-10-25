fn main() {
    let mut num: i32 = 42;
    // Cast to raw pointer outside unsafe block should fail sema
    let p: *i32! = &num as *i32!;
}
