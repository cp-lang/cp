fn main(): ?void {
    let limit = 10;
    let i = 0;
    while (i < limit) {
        i = i + 1;
    }
    @print("Limit reached: {d}", limit);
}