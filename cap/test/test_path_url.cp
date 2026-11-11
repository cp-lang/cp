// cap/test/test_path_url.cp
import { test, expect_eq } from "cap:test";
import { Path, PosixPath, WindowsPath } from "cap:path";
import { URL, URLSearchParams } from "cap:url";
const std = @import("std");

fn testPosixPath(): !void {
    var buf: [1024]u8 = undefined;

    // Basename
    expect_eq(std.mem.eql(u8, PosixPath.basename("/var/log/syslog", null), "syslog"), true);
    expect_eq(std.mem.eql(u8, PosixPath.basename("/app/src/index.ts", ".ts"), "index"), true);
    expect_eq(std.mem.eql(u8, PosixPath.basename("/app/src/index.ts", null), "index.ts"), true);

    // Dirname
    expect_eq(std.mem.eql(u8, PosixPath.dirname("/var/log/syslog"), "/var/log"), true);
    expect_eq(std.mem.eql(u8, PosixPath.dirname("/app"), "/"), true);
    expect_eq(std.mem.eql(u8, PosixPath.dirname("file.txt"), "."), true);

    // Extname
    expect_eq(std.mem.eql(u8, PosixPath.extname("index.html"), ".html"), true);
    expect_eq(std.mem.eql(u8, PosixPath.extname("archive.tar.gz"), ".gz"), true);
    expect_eq(std.mem.eql(u8, PosixPath.extname(".gitignore"), ""), true);
    expect_eq(std.mem.eql(u8, PosixPath.extname("Makefile"), ""), true);

    // isAbsolute
    expect_eq(PosixPath.isAbsolute("/usr/local/bin"), true);
    expect_eq(PosixPath.isAbsolute("relative/path"), false);

    // Normalize
    const n1 = PosixPath.normalize("/foo/bar//baz/asdf/quux/..", &buf);
    expect_eq(std.mem.eql(u8, n1, "/foo/bar/baz/asdf"), true);

    const n2 = PosixPath.normalize("a/b/../c/./d", &buf);
    expect_eq(std.mem.eql(u8, n2, "a/c/d"), true);

    // Join
    const paths = ["/usr", "local", "bin/cp"];
    const j1 = PosixPath.join(paths[..], &buf);
    expect_eq(std.mem.eql(u8, j1, "/usr/local/bin/cp"), true);
}

fn testWindowsPath(): !void {
    expect_eq(WindowsPath.isAbsolute("C:\\Users\\Admin"), true);
    expect_eq(WindowsPath.isAbsolute("\\\\server\\share\\file"), true);
    expect_eq(WindowsPath.isAbsolute("foo\\bar"), false);

    expect_eq(std.mem.eql(u8, WindowsPath.basename("C:\\dir\\file.exe", null), "file.exe"), true);
    expect_eq(std.mem.eql(u8, WindowsPath.basename("C:\\dir\\file.exe", ".exe"), "file"), true);
    expect_eq(std.mem.eql(u8, WindowsPath.dirname("C:\\dir\\file.exe"), "C:\\dir"), true);
    expect_eq(std.mem.eql(u8, WindowsPath.extname("C:\\dir\\file.exe"), ".exe"), true);
}

fn testURLSearchParams(): !void {
    const allocator = std.heap.page_allocator;
    var params = URLSearchParams.parse("?name=alice&role=engineer&team=deepmind", allocator);
    defer params.deinit();

    expect_eq(params.count(), 3);
    expect_eq(params.has("name"), true);
    expect_eq(std.mem.eql(u8, params.get("name").?, "alice"), true);
    expect_eq(std.mem.eql(u8, params.get("role").?, "engineer"), true);
    expect_eq(std.mem.eql(u8, params.get("team").?, "deepmind"), true);

    params.set("role", "architect");
    expect_eq(std.mem.eql(u8, params.get("role").?, "architect"), true);

    params.delete("team");
    expect_eq(params.count(), 2);
    expect_eq(params.has("team"), false);

    var buf: [1024]u8 = undefined;
    const qs = params.toString(&buf);
    expect_eq(std.mem.eql(u8, qs, "name=alice&role=architect"), true);
}

fn testURLParsing(): !void {
    const allocator = std.heap.page_allocator;
    var u = try URL.parse("https://agent:secret@example.com:8080/api/v1/query?sort=desc#results", allocator);
    defer u.deinit();

    expect_eq(std.mem.eql(u8, u.protocol, "https:"), true);
    expect_eq(std.mem.eql(u8, u.username, "agent"), true);
    expect_eq(std.mem.eql(u8, u.password, "secret"), true);
    expect_eq(std.mem.eql(u8, u.host, "example.com:8080"), true);
    expect_eq(std.mem.eql(u8, u.hostname, "example.com"), true);
    expect_eq(std.mem.eql(u8, u.port, "8080"), true);
    expect_eq(std.mem.eql(u8, u.pathname, "/api/v1/query"), true);
    expect_eq(std.mem.eql(u8, u.search, "?sort=desc"), true);
    expect_eq(std.mem.eql(u8, u.hash, "#results"), true);
    expect_eq(std.mem.eql(u8, u.origin, "https://example.com:8080"), true);

    expect_eq(u.searchParams.has("sort"), true);
    expect_eq(std.mem.eql(u8, u.searchParams.get("sort").?, "desc"), true);
}

fn main(): void {
    test("POSIX path operations (basename, dirname, extname, isAbsolute, normalize, join)", () => {
        try testPosixPath();
    });
    test("Windows path operations (drive letter, UNC, basename, dirname)", () => {
        try testWindowsPath();
    });
    test("URLSearchParams parsing, manipulation and serialization", () => {
        try testURLSearchParams();
    });
    test("WHATWG URL parsing and component decomposition", () => {
        try testURLParsing();
    });
}
