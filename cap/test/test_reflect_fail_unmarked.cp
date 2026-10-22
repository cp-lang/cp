// cap/test/test_reflect_fail_unmarked.cp
// Negative test: verify compile error when reflecting an unmarked class

class UnmarkedUser {
    id: i32;
    name: string;
}

fn main() {
    let info = reflect.type<UnmarkedUser>();
}
