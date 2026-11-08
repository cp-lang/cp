// Negative test: CP does not support Zig pipe capture syntax '|val|' or '|*val|'
fn main(): void {
    let opt: i32? = 10;
    if (opt) |*val| {
        val = 20;
    }
}
