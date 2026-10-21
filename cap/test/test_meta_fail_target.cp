// cap/test/test_meta_fail_target.cp
// Negative test: Applying @target(method) to a class must fail compilation

@target(method)
meta Get(path: string);

@Get("/wrong_target")
class InvalidController {
}

fn main() {
}
