enum IoErr {
    NotFound,
    PermDenied
}

fn readData() : ?string ! IoErr {
    return Err(IoErr.NotFound);
}

fn main() {
    // Error: `??` is reserved strictly for `?T` (null coalescing), cannot be applied to `T ! E`
    let val = readData() ?? "fallback";
    @print("{s}", val);
}
