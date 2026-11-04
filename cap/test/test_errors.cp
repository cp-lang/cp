error ModelError { NotFound, OutOfVRAM }

fn trigger_err(): i32! {
    return err.NotFound;
}

fn main(): void? {
    defer { @print("Deferred action."); }
    errdefer { @print("Error deferred action."); }
    let data: string? = null;
    @print("Running...");
    let val = trigger_err() catch 42;
    @print("Caught val: {d}", val);
}