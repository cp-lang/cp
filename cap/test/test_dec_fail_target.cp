// cap/test/test_dec_fail_target.cp
// Verify semantic error when applying decorator to invalid target (field)

dec log(next) {
    let res = next();
    return res;
}

class User {
    @log
    id: i32;
}

fn main() {
    let u = new User();
}
