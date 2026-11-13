// lib/string/escape.cp
// High-speed HTML entity escaping & RegExp pattern escaping
// Absorbed from bun/src/string/immutable/escapeHTML.zig & bun/src/string/escapeRegExp.cp

export struct StringEscape {
    pub static fn needsHtmlEscape(input: []const u8): bool {
        let i: usize = 0;
        while (i < input.len) {
            const c = input[i];
            if (c == 34 || c == 38 || c == 39 || c == 60 || c == 62) {
                return true;
            }
            i += 1;
        }
        return false;
    }

    // Escapes `"`, `&`, `'`, `<`, `>` into `out`, returning number of bytes written
    pub static fn escapeHtml(input: []const u8, out: []u8): usize {
        let i: usize = 0;
        let pos: usize = 0;
        while (i < input.len) {
            const c = input[i];
            if (c == 34) {
                // "&quot;" (6 bytes)
                if (pos + 6 > out.len) {
                    break;
                }
                const rep = "&quot;";
                let k: usize = 0;
                while (k < 6) {
                    out[pos + k] = rep[k];
                    k += 1;
                }
                pos += 6;
            } else if (c == 38) {
                // "&amp;" (5 bytes)
                if (pos + 5 > out.len) {
                    break;
                }
                const rep = "&amp;";
                let k: usize = 0;
                while (k < 5) {
                    out[pos + k] = rep[k];
                    k += 1;
                }
                pos += 5;
            } else if (c == 39) {
                // "&#x27;" (6 bytes)
                if (pos + 6 > out.len) {
                    break;
                }
                const rep = "&#x27;";
                let k: usize = 0;
                while (k < 6) {
                    out[pos + k] = rep[k];
                    k += 1;
                }
                pos += 6;
            } else if (c == 60) {
                // "&lt;" (4 bytes)
                if (pos + 4 > out.len) {
                    break;
                }
                const rep = "&lt;";
                let k: usize = 0;
                while (k < 4) {
                    out[pos + k] = rep[k];
                    k += 1;
                }
                pos += 4;
            } else if (c == 62) {
                // "&gt;" (4 bytes)
                if (pos + 4 > out.len) {
                    break;
                }
                const rep = "&gt;";
                let k: usize = 0;
                while (k < 4) {
                    out[pos + k] = rep[k];
                    k += 1;
                }
                pos += 4;
            } else {
                if (pos + 1 > out.len) {
                    break;
                }
                out[pos] = c;
                pos += 1;
            }
            i += 1;
        }
        return pos;
    }

    // Escapes RegExp metacharacters `|\{}()[]^$+*?.-` (absorbed from bun/src/string/escapeRegExp.cp)
    pub static fn escapeRegExp(input: []const u8, out: []u8): usize {
        let i: usize = 0;
        let pos: usize = 0;
        while (i < input.len) {
            const c = input[i];
            if (c == 124 || c == 92 || c == 123 || c == 125 || c == 40 || c == 41 || c == 91 || c == 93 || c == 94 || c == 36 || c == 43 || c == 42 || c == 63 || c == 46) {
                if (pos + 2 > out.len) {
                    break;
                }
                out[pos] = 92; // '\'
                out[pos + 1] = c;
                pos += 2;
            } else if (c == 45) {
                // '-' -> "\x2d"
                if (pos + 4 > out.len) {
                    break;
                }
                out[pos] = 92;
                out[pos + 1] = 120;
                out[pos + 2] = 50;
                out[pos + 3] = 100;
                pos += 4;
            } else {
                if (pos + 1 > out.len) {
                    break;
                }
                out[pos] = c;
                pos += 1;
            }
            i += 1;
        }
        return pos;
    }
}
