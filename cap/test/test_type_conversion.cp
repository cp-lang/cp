import { print } from "cap:io";

enum Status {
    Ok = 0,
    Err = 1
}

fn test_layer1_implicit_coercion() {
    // 1. Literal fitting
    let a: u64 = 100;
    let b: f32 = 3.14;
    let c: u8 = 255;
    let d: i8 = -128;

    // 2. Numeric safe widening
    let small_u8: u8 = 42;
    let big_u32: u32 = small_u8;
    let big_usize: usize = small_u8;
    let small_i16: i16 = -100;
    let big_i64: i64 = small_i16;
    let f_val: f64 = small_u8;

    // 3. Optional wrapping
    let opt_str: string? = "hello";
    let opt_num: i32? = 123;
}

fn test_layer2_explicit_casts() {
    // 1. Narrowing / Truncation
    let big: u64 = 1000;
    let small = big as u8;

    // 2. Sign reinterpretation
    let signed: i32 = -1;
    let unsigned = signed as u32;

    // 3. Float & Int conversion
    let pi: f64 = 3.14159;
    let int_pi = pi as i32;
    let f_from_i = int_pi as f32;

    // 4. Enum & Int conversion
    let code = Status.Ok as u32;
}

fn test_layer3_raw_pointers() {
    let mut num: i32 = 42;
    unsafe {
        let p: *i32! = &num as *i32!;
        let raw_u8: *u8! = p as *u8!;
        let addr: usize = p as usize;
        let new_ptr: *i32! = addr as *i32!;
    }
}

fn main() {
    test_layer1_implicit_coercion();
    test_layer2_explicit_casts();
    test_layer3_raw_pointers();
    @print("Four-tier type conversion tests passed successfully!");
}
