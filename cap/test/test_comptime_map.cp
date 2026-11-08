import { test, expect_eq } from "cap:test";

enum ACL {
    Private,
    PublicRead,
    PublicReadWrite,
}

enum MimeType {
    Html,
    Json,
    Text,
}

const ACL_MAP = @map(ACL, {
    "private": .Private,
    "public-read": .PublicRead,
    "public-read-write": .PublicReadWrite,
});

const MIME_MAP = @map({
    "text/html": MimeType.Html,
    "application/json": .Json,
    "text/plain": .Text,
});

fn main() {
    test("enum intrinsic get and has and name", () => {
        expect_eq(ACL.has("Private"), true);
        expect_eq(ACL.has("PublicRead"), true);
        expect_eq(ACL.has("Unknown"), false);

        let opt_acl = ACL.get("Private");
        expect_eq(opt_acl != null, true);
        expect_eq((opt_acl.?) as i32, ACL.Private as i32);

        let opt_none = ACL.get("NonExistent");
        expect_eq(opt_none == null, true);

        let val = ACL.PublicRead;
        expect_eq(val.name, "PublicRead");
    });

    test("comptime map lookup and verification", () => {
        expect_eq(ACL_MAP.has("private"), true);
        expect_eq(ACL_MAP.has("public-read"), true);
        expect_eq(ACL_MAP.has("bogus"), false);

        let a = ACL_MAP.get("public-read");
        expect_eq(a != null, true);
        expect_eq((a.?) as i32, ACL.PublicRead as i32);

        let m = MIME_MAP.get("application/json");
        expect_eq(m != null, true);
        expect_eq((m.?) as i32, MimeType.Json as i32);
    });
}
