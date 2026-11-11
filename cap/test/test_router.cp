// cap/test/test_router.cp
import { test, expect_eq } from "cap:test";
import { Router, ParamMap } from "cap:router";
const std = @import("std");

fn testStaticRouting(): !void {
    const allocator = std.heap.page_allocator;
    var router = Router(string).init(allocator);
    router.get("/", "home");
    router.get("/users", "users_list");
    router.post("/users", "create_user");
    router.get("/about", "about_page");

    // Match root
    const m1 = router.lookup("GET", "/");
    expect_eq(m1 != null, true);
    expect_eq(std.mem.eql(u8, m1.?.handler, "home"), true);

    // Match static GET
    const m2 = router.lookup("GET", "/users");
    expect_eq(m2 != null, true);
    expect_eq(std.mem.eql(u8, m2.?.handler, "users_list"), true);

    // Match static POST
    const m3 = router.lookup("POST", "/users");
    expect_eq(m3 != null, true);
    expect_eq(std.mem.eql(u8, m3.?.handler, "create_user"), true);

    // Method mismatch
    const m4 = router.lookup("DELETE", "/users");
    expect_eq(m4 == null, true);

    // Non-existent route
    const m5 = router.lookup("GET", "/nonexistent");
    expect_eq(m5 == null, true);
}

fn testDynamicParameters(): !void {
    const allocator = std.heap.page_allocator;
    var router = Router(string).init(allocator);
    router.get("/users/:id", "get_user");
    router.get("/users/:id/posts/:postId", "get_user_post");
    router.get("/org/[orgId]/members/[memberId]", "get_member");

    // Single param
    const m1 = router.lookup("GET", "/users/1001");
    expect_eq(m1 != null, true);
    expect_eq(std.mem.eql(u8, m1.?.handler, "get_user"), true);
    expect_eq(std.mem.eql(u8, m1.?.params.get("id").?, "1001"), true);

    // Multi params
    const m2 = router.lookup("GET", "/users/alice/posts/p99");
    expect_eq(m2 != null, true);
    expect_eq(std.mem.eql(u8, m2.?.handler, "get_user_post"), true);
    expect_eq(std.mem.eql(u8, m2.?.params.get("id").?, "alice"), true);
    expect_eq(std.mem.eql(u8, m2.?.params.get("postId").?, "p99"), true);

    // Bracket syntax
    const m3 = router.lookup("GET", "/org/deepmind/members/m42");
    expect_eq(m3 != null, true);
    expect_eq(std.mem.eql(u8, m3.?.handler, "get_member"), true);
    expect_eq(std.mem.eql(u8, m3.?.params.get("orgId").?, "deepmind"), true);
    expect_eq(std.mem.eql(u8, m3.?.params.get("memberId").?, "m42"), true);
}

fn testWildcardCatchAll(): !void {
    const allocator = std.heap.page_allocator;
    var router = Router(string).init(allocator);
    router.get("/static/*filepath", "serve_static");
    router.get("/docs/[...docpath]", "serve_doc");
    router.all("/health", "health_check");

    // Wildcard path
    const m1 = router.lookup("GET", "/static/images/avatar.png");
    expect_eq(m1 != null, true);
    expect_eq(std.mem.eql(u8, m1.?.handler, "serve_static"), true);
    expect_eq(std.mem.eql(u8, m1.?.params.get("filepath").?, "images/avatar.png"), true);

    // Bracket catch-all
    const m2 = router.lookup("GET", "/docs/api/v1/overview");
    expect_eq(m2 != null, true);
    expect_eq(std.mem.eql(u8, m2.?.handler, "serve_doc"), true);
    expect_eq(std.mem.eql(u8, m2.?.params.get("docpath").?, "api/v1/overview"), true);

    // ALL method catch-all
    const m_get = router.lookup("GET", "/health");
    expect_eq(m_get != null, true);
    expect_eq(std.mem.eql(u8, m_get.?.handler, "health_check"), true);

    const m_post = router.lookup("POST", "/health");
    expect_eq(m_post != null, true);
    expect_eq(std.mem.eql(u8, m_post.?.handler, "health_check"), true);
}

fn main(): void {
    test("Static route matching and method discrimination", () => {
        try testStaticRouting();
    });
    test("Dynamic parameter extraction (:param and [param])", () => {
        try testDynamicParameters();
    });
    test("Wildcard catch-all (*path and [...path]) and ALL method", () => {
        try testWildcardCatchAll();
    });
}
