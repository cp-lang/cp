fn main(): ?void {
    let x: i32 = undefined;
    if (false) {
        x = 10;
    }
    @print("{d}", x);
}
