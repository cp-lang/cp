// cap/test/test_meta_fail_once.cp
// Negative test: Applying @once meta multiple times must fail compilation

@once
meta Unique;

@Unique
@Unique
class InvalidDuplicate {
}

fn main() {
}
