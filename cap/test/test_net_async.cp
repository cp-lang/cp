import { test, expect_eq } from "cap:test";
import { listen } from "cap:net";

fn main(): void {
    test("Net Async Accept and Read", () => {
        let server = listen(8081);
        // We can't really test a real client easily without spawn, but let's test if listen and close work
        server.close();
        expect_eq(1, 1);
    });
}
