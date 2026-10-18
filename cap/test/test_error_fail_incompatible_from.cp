enum DbErr {
    ConnLost,
    Deadlock
}

enum AppErr {
    InvalidInput,
    OutOfMemory
}

fn queryDb() : i32 ! DbErr {
    return Err(DbErr.ConnLost);
}

// Error: `AppErr` has no `from DbErr` declaration, so `?` cannot auto-convert `DbErr` -> `AppErr`
fn runService() : i32 ! AppErr {
    let rows = queryDb()?;
    return rows;
}

fn main() {
    let r = runService() ? -1;
    @print("{d}", r);
}
