// cap/test/test_pipe_fail_placeholder.cp
// Negative test: standalone topic placeholder '.' cannot be used outside pipeline '|>'

fn add_num(a: i32, b: i32): i32 {
    return a + b;
}

fn main(): void {
    // Error: Standalone '.' used outside pipeline operator
    let invalid = add_num(., 5);
}
