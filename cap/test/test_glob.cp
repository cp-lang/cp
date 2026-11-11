// cap/test/test_glob.cp
import { test, expect_eq } from "cap:test";
import { Glob } from "cap:glob";

fn testGlobBasics(): !void {
    // 1. Single wildcard *
    expect_eq(Glob.match("*.cp", "main.cp"), true);
    expect_eq(Glob.match("*.cp", "test.zig"), false);
    expect_eq(Glob.match("*.cp", "src/main.cp"), false); // * does not span directories

    // 2. Single char ?
    expect_eq(Glob.match("test?.cp", "test1.cp"), true);
    expect_eq(Glob.match("test?.cp", "testA.cp"), true);
    expect_eq(Glob.match("test?.cp", "test12.cp"), false);

    // 3. Exact literal matching
    expect_eq(Glob.match("src/index.cp", "src/index.cp"), true);
    expect_eq(Glob.match("src/index.cp", "src/other.cp"), false);
}

fn testGlobStar(): !void {
    // ** matches arbitrary directory depth
    expect_eq(Glob.match("**/*.cp", "main.cp"), true);
    expect_eq(Glob.match("**/*.cp", "src/main.cp"), true);
    expect_eq(Glob.match("**/*.cp", "src/core/parser/expr.cp"), true);
    expect_eq(Glob.match("lib/**/test_*.cp", "lib/sync/test_futex.cp"), true);
    expect_eq(Glob.match("lib/**/test_*.cp", "lib/a/b/c/test_deep.cp"), true);
    expect_eq(Glob.match("lib/**/test_*.cp", "src/test_futex.cp"), false);
}

fn testCharacterClassAndNegation(): !void {
    // [0-9] range
    expect_eq(Glob.match("v[0-9].txt", "v1.txt"), true);
    expect_eq(Glob.match("v[0-9].txt", "v9.txt"), true);
    expect_eq(Glob.match("v[0-9].txt", "va.txt"), false);

    // [a-z] range
    expect_eq(Glob.match("log_[a-z].txt", "log_d.txt"), true);
    expect_eq(Glob.match("log_[a-z].txt", "log_1.txt"), false);

    // ! Negation
    expect_eq(Glob.match("!*.tmp", "data.json"), true);
    expect_eq(Glob.match("!*.tmp", "temp.tmp"), false);
}

fn testHasMagic(): !void {
    expect_eq(Glob.hasMagic("src/**/*.cp"), true);
    expect_eq(Glob.hasMagic("test?.cp"), true);
    expect_eq(Glob.hasMagic("file[0-9].txt"), true);
    expect_eq(Glob.hasMagic("plain_filename.cp"), false);
}

fn main(): void {
    test("Glob basic wildcards (* and ?)", () => {
        try testGlobBasics();
    });
    test("Glob recursive globstar (**)", () => {
        try testGlobStar();
    });
    test("Glob character classes [a-z] and negation (!)", () => {
        try testCharacterClassAndNegation();
    });
    test("Glob syntax magic detection hasMagic()", () => {
        try testHasMagic();
    });
}
