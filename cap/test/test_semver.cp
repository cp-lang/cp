// cap/test/test_semver.cp
const std = @import("std");
import { test, expect_eq } from "cap:test";
import { Semver, Version } from "cap:semver";

fn testVersionParsing(): !void {
    const v1 = try Version.parse("1.2.3");
    expect_eq(v1.major, 1);
    expect_eq(v1.minor, 2);
    expect_eq(v1.patch, 3);
    expect_eq(v1.prerelease.len, 0);
    expect_eq(v1.build.len, 0);

    const v2 = try Version.parse("v2.10.5-beta.1+sha.abcdef");
    expect_eq(v2.major, 2);
    expect_eq(v2.minor, 10);
    expect_eq(v2.patch, 5);
    expect_eq(std.mem.eql(u8, v2.prerelease, "beta.1"), true);
    expect_eq(std.mem.eql(u8, v2.build, "sha.abcdef"), true);

    const v3 = try Version.parse("=0.4.0");
    expect_eq(v3.major, 0);
    expect_eq(v3.minor, 4);
    expect_eq(v3.patch, 0);
}

fn testVersionComparison(): !void {
    const v1 = try Version.parse("1.0.0");
    const v2 = try Version.parse("2.0.0");
    const v3 = try Version.parse("1.1.0");
    const v4 = try Version.parse("1.0.1");

    expect_eq(v1.lt(v2), true);
    expect_eq(v2.gt(v1), true);
    expect_eq(v3.gt(v1), true);
    expect_eq(v4.gt(v1), true);
    expect_eq(v1.eql(v1), true);

    // Prerelease comparisons (Semver 2.0 spec)
    const v_release = try Version.parse("1.0.0");
    const v_alpha = try Version.parse("1.0.0-alpha");
    const v_alpha1 = try Version.parse("1.0.0-alpha.1");
    const v_beta = try Version.parse("1.0.0-beta");
    const v_beta2 = try Version.parse("1.0.0-beta.2");
    const v_beta11 = try Version.parse("1.0.0-beta.11");

    // normal release > prerelease
    expect_eq(v_release.gt(v_alpha), true);
    expect_eq(v_alpha.lt(v_release), true);

    // alpha < beta
    expect_eq(v_alpha.lt(v_beta), true);

    // alpha < alpha.1
    expect_eq(v_alpha.lt(v_alpha1), true);

    // numeric component comparison: 2 < 11
    expect_eq(v_beta2.lt(v_beta11), true);
    expect_eq(v_beta11.gt(v_beta2), true);
}

fn testVersionFormatting(): !void {
    const v = try Version.parse("1.5.12-rc.1+arm64");
    var buf: [64]u8 = undefined;
    const formatted = v.format(&buf);
    expect_eq(std.mem.eql(u8, formatted, "1.5.12-rc.1+arm64"), true);
}

fn testSemverRanges(): !void {
    // 1. Caret ^
    expect_eq(Semver.satisfiesStr("1.2.3", "^1.2.3"), true);
    expect_eq(Semver.satisfiesStr("1.5.0", "^1.2.3"), true);
    expect_eq(Semver.satisfiesStr("2.0.0", "^1.2.3"), false);
    expect_eq(Semver.satisfiesStr("1.2.2", "^1.2.3"), false);

    // Caret with 0.x
    expect_eq(Semver.satisfiesStr("0.2.3", "^0.2.0"), true);
    expect_eq(Semver.satisfiesStr("0.2.9", "^0.2.0"), true);
    expect_eq(Semver.satisfiesStr("0.3.0", "^0.2.0"), false);

    // Caret with 0.0.x
    expect_eq(Semver.satisfiesStr("0.0.3", "^0.0.3"), true);
    expect_eq(Semver.satisfiesStr("0.0.4", "^0.0.3"), false);

    // 2. Tilde ~
    expect_eq(Semver.satisfiesStr("1.2.3", "~1.2.0"), true);
    expect_eq(Semver.satisfiesStr("1.2.9", "~1.2.0"), true);
    expect_eq(Semver.satisfiesStr("1.3.0", "~1.2.0"), false);

    // 3. Simple inequalities and bounds
    expect_eq(Semver.satisfiesStr("1.5.0", ">=1.0.0 <2.0.0"), true);
    expect_eq(Semver.satisfiesStr("2.0.0", ">=1.0.0 <2.0.0"), false);
    expect_eq(Semver.satisfiesStr("0.9.9", ">=1.0.0 <2.0.0"), false);

    // 4. Hyphen range
    expect_eq(Semver.satisfiesStr("1.5.0", "1.0.0 - 2.0.0"), true);
    expect_eq(Semver.satisfiesStr("2.0.0", "1.0.0 - 2.0.0"), true);
    expect_eq(Semver.satisfiesStr("2.0.1", "1.0.0 - 2.0.0"), false);

    // 5. Logical OR (||)
    expect_eq(Semver.satisfiesStr("1.5.0", "^1.0.0 || ^3.0.0"), true);
    expect_eq(Semver.satisfiesStr("3.2.0", "^1.0.0 || ^3.0.0"), true);
    expect_eq(Semver.satisfiesStr("2.1.0", "^1.0.0 || ^3.0.0"), false);

    // 6. Wildcard
    expect_eq(Semver.satisfiesStr("9.9.9", "*"), true);
}

fn main(): void {
    test("Semver 2.0 version string parsing", () => {
        try testVersionParsing();
    });
    test("Semver 2.0 precedence and comparison", () => {
        try testVersionComparison();
    });
    test("Semver formatting", () => {
        try testVersionFormatting();
    });
    test("Semver range matching (^, ~, >=, <=, -, ||, *)", () => {
        try testSemverRanges();
    });
}
