// lib/string/simd_scan.cp
// 16-byte SIMD (@Vector(16, u8)) high-speed string scanner
// Absorbed from bun/src/string/immutable.zig & bun/src/string/immutable/unicode.zig

type Vec16u8 = @Vector(16, u8);

export struct SimdScanner {
    // Returns the index of the first non-ASCII byte (>= 0x80), or null if 100% ASCII
    pub static fn firstNonAsciiIndex(input: []const u8): usize? {
        let i: usize = 0;
        const n = input.len;
        const high_bit: Vec16u8 = @splat(0x80);
        const zero_vec: Vec16u8 = @splat(0);

        while (i + 16 <= n) {
            let chunk: Vec16u8 = @splat(0);
            zig {
                chunk = input[i..][0..16].*;
            }
            const masked = chunk & high_bit;
            const cmp = masked != zero_vec;
            const mask: u16 = @bitCast(cmp);
            if (mask != 0) {
                const bit_idx = @ctz(mask) as usize;
                return i + bit_idx;
            }
            i += 16;
        }

        while (i < n) {
            if (input[i] >= 0x80) {
                return i;
            }
            i += 1;
        }
        return null;
    }

    pub static fn isAscii(input: []const u8): bool {
        return SimdScanner.firstNonAsciiIndex(input) == null;
    }

    // 16-byte SIMD search for single byte `needle`
    pub static fn indexOfByte(input: []const u8, needle: u8): usize? {
        let i: usize = 0;
        const n = input.len;
        const target: Vec16u8 = @splat(needle);

        while (i + 16 <= n) {
            let chunk: Vec16u8 = @splat(0);
            zig {
                chunk = input[i..][0..16].*;
            }
            const cmp = chunk == target;
            const mask: u16 = @bitCast(cmp);
            if (mask != 0) {
                const bit_idx = @ctz(mask) as usize;
                return i + bit_idx;
            }
            i += 16;
        }

        while (i < n) {
            if (input[i] == needle) {
                return i;
            }
            i += 1;
        }
        return null;
    }

    // 16-byte SIMD search for either of two target bytes `b1` or `b2`
    pub static fn indexOfAny2(input: []const u8, b1: u8, b2: u8): usize? {
        let i: usize = 0;
        const n = input.len;
        const t1: Vec16u8 = @splat(b1);
        const t2: Vec16u8 = @splat(b2);

        while (i + 16 <= n) {
            let chunk: Vec16u8 = @splat(0);
            zig {
                chunk = input[i..][0..16].*;
            }
            const m1: u16 = @bitCast(chunk == t1);
            const m2: u16 = @bitCast(chunk == t2);
            const mask: u16 = m1 | m2;
            if (mask != 0) {
                const bit_idx = @ctz(mask) as usize;
                return i + bit_idx;
            }
            i += 16;
        }

        while (i < n) {
            const c = input[i];
            if (c == b1 || c == b2) {
                return i;
            }
            i += 1;
        }
        return null;
    }

    // 16-byte SIMD byte frequency count using @bitCast + @popCount
    pub static fn countByte(input: []const u8, needle: u8): usize {
        let i: usize = 0;
        let total: usize = 0;
        const n = input.len;
        const target: Vec16u8 = @splat(needle);

        while (i + 16 <= n) {
            let chunk: Vec16u8 = @splat(0);
            zig {
                chunk = input[i..][0..16].*;
            }
            const cmp = chunk == target;
            const mask: u16 = @bitCast(cmp);
            total += (@popCount(mask) as usize);
            i += 16;
        }

        while (i < n) {
            if (input[i] == needle) {
                total += 1;
            }
            i += 1;
        }
        return total;
    }

    pub static fn eqlCaseInsensitiveAscii(a: []const u8, b: []const u8): bool {
        if (a.len != b.len) {
            return false;
        }
        let i: usize = 0;
        while (i < a.len) {
            let ca = a[i];
            let cb = b[i];
            if (ca >= 65 && ca <= 90) {
                ca += 32;
            }
            if (cb >= 65 && cb <= 90) {
                cb += 32;
            }
            if (ca != cb) {
                return false;
            }
            i += 1;
        }
        return true;
    }
}
