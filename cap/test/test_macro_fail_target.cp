// cap/test/test_macro_fail_target.cp
// Negative test: Macro applied to an invalid target (method instead of class/struct)

class Service {
    @Json
    fn doSomething(): void {
    }
}

fn main(): void {
}
