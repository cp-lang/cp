import { test, expect_eq } from "cap:test";
import { randomUUID, hash } from "cap:crypto";

fn main(): void {
    test("Crypto hash (SHA256)", () => {
        let h = hash("hello");
        expect_eq(h, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    });

    test("Crypto randomUUID", () => {
        let uuid = randomUUID();
        // Since it's random we can't test exact equality, but we can test length
        expect_eq(uuid.len, 36);
    });
}
