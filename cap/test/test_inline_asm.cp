import { test, expect_eq } from "cap:test";

fn test_basic_asm() {
    @asm("nop");
    asm volatile ("nop");
}

fn test_asm_with_io(): usize {
    let result = @asm(
        "mov %[input], %[ret]"
        : [ret] "=r" (-> usize),
        : [input] "r" (12345),
    );
    return result;
}

fn main() {
    test("basic inline asm execution", () => {
        test_basic_asm();
        expect_eq(1, 1);
    });

    test("inline asm with operands", () => {
        let val = test_asm_with_io();
        expect_eq(val, 12345);
    });
}
