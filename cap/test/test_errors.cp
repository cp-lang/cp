error ModelError { NotFound, OutOfVRAM }

fn main(): ?void {
    defer { @print("Deferred action."); }
    errdefer { @print("Error deferred action."); }
    let data: ?string = null;
    @print("Running...");
    return error.ModelError.NotFound;
}