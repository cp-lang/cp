trait PluginError {
    fn code(_) : i32;
}

struct CustomErr {
    c: i32;
}

impl PluginError for CustomErr {
    fn code(_) : i32 {
        return _.c;
    }
}

// Error: Bare `dyn Trait` on the stack is forbidden; must use `Box<dyn Trait>`
fn handleErr(e: dyn PluginError) : i32 {
    return 0;
}

fn main() {
    @print("bare dyn test");
}
