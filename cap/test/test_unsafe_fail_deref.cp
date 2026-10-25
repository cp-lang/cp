fn testDeref(p: *i32!) {
    // Dereferencing raw pointer outside unsafe block should fail sema
    let v = p.*;
}

fn main() {
}
