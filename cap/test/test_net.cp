import { test, expect_eq } from "cap:test";
import { listen } from "cap:net";

fn main(): void {
    test("Net Server Listen", () => {
        let server = listen(8080);
        expect_eq(server.fd, 100);
    });
}
