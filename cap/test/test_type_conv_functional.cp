import { test, expect_eq } from "cap:test";

enum SocketKind {
    Tcp = 0,
    Udp = 1,
    Unix = 2,
}

fn main() {
    test("enum <-> int functional casts", () => {
        let k_udp = SocketKind(1);
        expect_eq(k_udp as i32, 1);

        let k_unix = SocketKind(2);
        expect_eq(k_unix as i32, 2);

        let int_udp = i32(SocketKind.Udp);
        expect_eq(int_udp, 1);

        let uint_unix = u32(SocketKind.Unix);
        expect_eq(uint_unix, 2);
    });

    test("scalar functional casts", () => {
        let f = f32(100);
        expect_eq(f as i32, 100);

        let b_true = bool(1);
        expect_eq(b_true, true);

        let b_false = bool(0);
        expect_eq(b_false, false);
    });

    test("string(val) and String(val)", () => {
        let s_enum = string(SocketKind.Tcp);
        expect_eq(s_enum, "Tcp");

        let s_num = string(12345);
        expect_eq(s_num, "12345");

        let s_bool = String(true);
        expect_eq(s_bool, "true");
    });

    test("built-in .toString() on enums and primitives", () => {
        let enum_str = SocketKind.Unix.toString();
        expect_eq(enum_str, "Unix");

        let dyn_k = SocketKind.Udp;
        let dyn_enum_str = dyn_k.toString();
        expect_eq(dyn_enum_str, "Udp");

        let num_val: i32 = 456;
        let num_str = num_val.toString();
        expect_eq(num_str, "456");

        let lit_str = (789).toString();
        expect_eq(lit_str, "789");
    });
}
