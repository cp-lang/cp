import { test, expect_eq } from "cap:test";
import { print } from "cap:io";

async fn main(): void {
    // 1. Static anonymous object creation & field access
    let user = { id: 101, name: "Alice", active: true };
    expect_eq(user.id, 101);
    expect_eq(user.name, "Alice");
    expect_eq(user.active, true);

    // 2. Destructuring assignment
    let { id, name } = user;
    expect_eq(id, 101);
    expect_eq(name, "Alice");

    // 3. Object spread (...) and property override
    let updated = { ...user, active: false, role: "admin" };
    expect_eq(updated.id, 101);
    expect_eq(updated.name, "Alice");
    expect_eq(updated.active, false);
    expect_eq(updated.role, "admin");

    // 4. Multiple spreads & overrides
    let extra = { score: 99, level: 5 };
    let combined = { ...user, ...extra, id: 202 };
    expect_eq(combined.id, 202);
    expect_eq(combined.name, "Alice");
    expect_eq(combined.score, 99);
    expect_eq(combined.level, 5);

    @print("🎉 Phase 1: Static anonymous objects & spread verified successfully!");
}
