// lib/semver/range.cp
// Semver Range matching (^, ~, >=, <=, >, <, =, *, ||) in CP
const std = @import("std");
import { Version } from "./version";

pub const Op = enum {
    any,
    eq,
    lt,
    lte,
    gt,
    gte,
};

pub const Comparator = struct {
    op: Op = .any,
    version: Version = .{},

    pub fn satisfies(self: Comparator, v: Version): bool {
        return switch (self.op) {
            .any => true,
            .eq => v.eql(self.version),
            .lt => v.lt(self.version),
            .lte => v.lte(self.version),
            .gt => v.gt(self.version),
            .gte => v.gte(self.version),
        };
    }
};

pub const SemverRange = struct {
    pub fn satisfiesStr(version_str: []const u8, range_str: []const u8): bool {
        const v = Version.parse(version_str) catch return false;
        return satisfies(v, range_str);
    }

    pub fn satisfies(v: Version, range_str: []const u8): bool {
        const trimmed = std.mem.trim(u8, range_str, " \t\r\n");
        if (trimmed.len == 0 or std.mem.eql(u8, trimmed, "*")) return true;

        // Split by logical OR: "||"
        var or_iter = std.mem.splitSequence(u8, trimmed, "||");
        while (or_iter.next()) |clause| {
            const trimmed_clause = std.mem.trim(u8, clause, " \t\r\n");
            if (trimmed_clause.len == 0) continue;

            if (matchesClause(v, trimmed_clause)) {
                return true;
            }
        }
        return false;
    }

    fn matchesClause(v: Version, clause: []const u8): bool {
        // Check for hyphen range: "1.2.3 - 2.3.4"
        if (std.mem.indexOf(u8, clause, " - ")) |sep_idx| {
            const left = std.mem.trim(u8, clause[0..sep_idx], " ");
            const right = std.mem.trim(u8, clause[sep_idx + 3 ..], " ");
            const v_left = Version.parse(left) catch return false;
            const v_right = Version.parse(right) catch return false;
            return v.gte(v_left) and v.lte(v_right);
        }

        // Split by whitespace: all conditions in clause must be satisfied (AND)
        var iter = std.mem.tokenizeAny(u8, clause, " \t\r\n");
        while (iter.next()) |token| {
            if (!matchesToken(v, token)) {
                return false;
            }
        }
        return true;
    }

    fn matchesToken(v: Version, tok: []const u8): bool {
        if (tok.len == 0 or std.mem.eql(u8, tok, "*") or std.mem.eql(u8, tok, "x") or std.mem.eql(u8, tok, "X")) {
            return true;
        }

        // Caret range ^1.2.3
        if (tok[0] == '^') {
            const target = Version.parse(tok[1..]) catch return false;
            // Lower bound: >= target
            if (v.lt(target)) return false;

            // Upper bound depends on major/minor/patch
            if (target.major > 0) {
                const upper = Version{ .major = target.major + 1, .minor = 0, .patch = 0 };
                return v.lt(upper);
            } else if (target.minor > 0) {
                const upper = Version{ .major = 0, .minor = target.minor + 1, .patch = 0 };
                return v.lt(upper);
            } else {
                const upper = Version{ .major = 0, .minor = 0, .patch = target.patch + 1 };
                return v.lt(upper);
            }
        }

        // Tilde range ~1.2.3
        if (tok[0] == '~') {
            const target = Version.parse(tok[1..]) catch return false;
            // Lower bound: >= target
            if (v.lt(target)) return false;

            // Upper bound: minor increments, patch reset
            const upper = Version{ .major = target.major, .minor = target.minor + 1, .patch = 0 };
            return v.lt(upper);
        }

        // >=
        if (std.mem.startsWith(u8, tok, ">=")) {
            const target = Version.parse(tok[2..]) catch return false;
            return v.gte(target);
        }

        // <=
        if (std.mem.startsWith(u8, tok, "<=")) {
            const target = Version.parse(tok[2..]) catch return false;
            return v.lte(target);
        }

        // >
        if (std.mem.startsWith(u8, tok, ">")) {
            const target = Version.parse(tok[1..]) catch return false;
            return v.gt(target);
        }

        // <
        if (std.mem.startsWith(u8, tok, "<")) {
            const target = Version.parse(tok[1..]) catch return false;
            return v.lt(target);
        }

        // =
        if (std.mem.startsWith(u8, tok, "=")) {
            const target = Version.parse(tok[1..]) catch return false;
            return v.eql(target);
        }

        // Plain version: exact match
        const target = Version.parse(tok) catch return false;
        return v.eql(target);
    }
};
