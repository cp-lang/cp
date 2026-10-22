// cap/test/test_reflect_fail_of_unmarked.cp
// Negative test: verify compile error when calling reflect.of() on an unmarked class instance

class PlainAccount {
    balance: f32;
}

fn main() {
    let acc = new PlainAccount();
    let info = reflect.of(acc);
}
