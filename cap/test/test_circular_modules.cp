import { print } from "cap:io";
import { NodeA } from "./fixtures/circ_a.cp";
import { NodeB } from "./fixtures/circ_b.cp";

pub fn main(): void {
    print("Testing Two-Pass Circular Module Resolution...");

    let a = new NodeA(101, null);
    let b = new NodeB(202, null);

    a.attach_b(&b);
    b.attach_a(&a);

    @expect_eq(a.get_id(), 101);
    @expect_eq(b.get_tag(), 202);

    if (partner_b := a.b) {
        @expect_eq(partner_b.get_tag(), 202);
    } else {
        @panic("Expected partner_b to be present");
    }

    if (partner_a := b.a) {
        @expect_eq(partner_a.get_id(), 101);
    } else {
        @panic("Expected partner_a to be present");
    }

    print("Test Passed: Circular modules resolved and linked successfully!");
}
