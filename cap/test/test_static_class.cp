import { test, expect_eq } from "cap:test";

static class MathUtils {
    pub count: i32 = 0;

    pub fn square(x: i32): i32 {
        return x * x;
    }

    pub fn add(a: i32, b: i32): i32 {
        return a + b;
    }

    pub fn bump(): i32 {
        MathUtils.count = MathUtils.count + 1;
        return MathUtils.count;
    }
}

static struct StringHelper {
    tag: string = "CP_STATIC";

    fn shout(msg: string): string {
        return msg;
    }

    fn double_str(s: string): string {
        return s + s;
    }
}

fn main(): void {
    test("static class methods and fields without instance receiver", () => {
        expect_eq(MathUtils.square(6), 36);
        expect_eq(MathUtils.add(10, 25), 35);
        expect_eq(MathUtils.count, 0);

        let n1 = MathUtils.bump();
        expect_eq(n1, 1);
        let n2 = MathUtils.bump();
        expect_eq(n2, 2);
        expect_eq(MathUtils.count, 2);

        MathUtils.count = 100;
        expect_eq(MathUtils.count, 100);
    });

    test("static struct with default pub members and zero boilerplate", () => {
        expect_eq(StringHelper.tag, "CP_STATIC");
        expect_eq(StringHelper.shout("cp"), "cp");
        expect_eq(StringHelper.double_str("abc"), "abcabc");
    });
}
