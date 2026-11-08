import { test, expect_eq } from "cap:test";

enum SocketKind {
    Tcp = 0,
    Udp = 1,
    Unix = 2,
}

fn main() {
    test("enum to int cast", () => {
        let kind = SocketKind.Udp;
        let int_val = kind as i32;
        let u_val = kind as u32;
        expect_eq(int_val, 1);
        expect_eq(u_val, 1);
    });

    test("int to enum cast", () => {
        let raw = 2;
        let kind = raw as SocketKind;
        expect_eq(kind as i32, 2);

        let kind0 = 0 as SocketKind;
        expect_eq(kind0 as i32, 0);
    });
}
