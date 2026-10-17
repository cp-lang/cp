/**
 * CP Test SDK (cap:test)
 * Zero-overhead native test runner
 */

export class Expectation {
    actual: i32, // placeholder, wait, I can just use macros directly in the test!
}

export fn test(name: string, callback: any): void {
    @test(name, callback);
}

export fn expect_eq(actual: i32, expected: i32): void {
    @expect_eq(actual, expected);
}
