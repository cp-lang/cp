// cap/test/test_error_recovery_fail.cp
// Negative test: Multiple syntax errors in a single module must all be collected
// via panic-mode synchronization instead of aborting at the first error.

fn test_recovery() {
    let mut a = 1;
    let b = 2;
    if x > 0 {
        return;
    }
    let mut c = 3;
}
