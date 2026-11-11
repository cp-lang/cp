// Regression test for extern const and extern var declarations
const StaticStringImpl = anyopaque;

@c extern const IPv4: StaticStringImpl;
@c extern const IPv6: StaticStringImpl;

extern const ExternalFlag: i32;
extern var ExternalCounter: u32;

@c {
    extern const BlockConst: i32;
    var BlockVar: u32;
}

pub fn main(): void {
    // Verified AST and codegen compilation of extern declarations
}
