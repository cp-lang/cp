// lib/string/utf.cp
// High-speed UTF-8 / UTF-16 / WTF-8 codecs & East-Asian / ANSI visible display width
// Absorbed from bun/src/string/immutable/unicode.zig, wtf.zig, visible.zig & unicode/uucode/

import { SimdScanner } from "./simd_scan.cp";

export const REPLACEMENT_CHAR: u32 = 0xFFFD;

export struct DecodeResult {
    pub codepoint: u32 = 0xFFFD;
    pub bytes_consumed: u8 = 1;
    pub valid: bool = false;
}

export struct Utf8 {
    // Returns 1..4 for valid UTF-8 leading byte, or 0 for invalid / continuation byte
    pub static fn sequenceLength(lead: u8): u8 {
        if (lead < 0x80) {
            return 1;
        }
        if (lead >= 0xC2 && lead <= 0xDF) {
            return 2;
        }
        if (lead >= 0xE0 && lead <= 0xEF) {
            return 3;
        }
        if (lead >= 0xF0 && lead <= 0xF4) {
            return 4;
        }
        return 0;
    }

    // Decodes one UTF-8 sequence starting at `offset` (with 0xFFFD replacement fallback)
    pub static fn decodeNext(input: []const u8, offset: usize): DecodeResult {
        if (offset >= input.len) {
            return DecodeResult{ codepoint: 0, bytes_consumed: 0, valid: false };
        }
        const b0 = input[offset];
        if (b0 < 0x80) {
            return DecodeResult{
                codepoint: b0 as u32,
                bytes_consumed: 1,
                valid: true,
            };
        }
        const seq_len = Utf8.sequenceLength(b0);
        if (seq_len == 0 || offset + (seq_len as usize) > input.len) {
            return DecodeResult{ codepoint: REPLACEMENT_CHAR, bytes_consumed: 1, valid: false };
        }

        if (seq_len == 2) {
            const b1 = input[offset + 1];
            if ((b1 & 0xC0) != 0x80) {
                return DecodeResult{ codepoint: REPLACEMENT_CHAR, bytes_consumed: 1, valid: false };
            }
            const cp: u32 = (((b0 & 0x1F) as u32) << 6) | ((b1 & 0x3F) as u32);
            return DecodeResult{ codepoint: cp, bytes_consumed: 2, valid: true };
        }

        if (seq_len == 3) {
            const b1 = input[offset + 1];
            const b2 = input[offset + 2];
            if ((b1 & 0xC0) != 0x80 || (b2 & 0xC0) != 0x80) {
                return DecodeResult{ codepoint: REPLACEMENT_CHAR, bytes_consumed: 1, valid: false };
            }
            // Check overlong (E0 followed by < A0) and UTF-16 surrogates (ED followed by >= A0)
            if ((b0 == 0xE0 && b1 < 0xA0) || (b0 == 0xED && b1 >= 0xA0)) {
                return DecodeResult{ codepoint: REPLACEMENT_CHAR, bytes_consumed: 1, valid: false };
            }
            const cp: u32 = (((b0 & 0x0F) as u32) << 12) | (((b1 & 0x3F) as u32) << 6) | ((b2 & 0x3F) as u32);
            return DecodeResult{ codepoint: cp, bytes_consumed: 3, valid: true };
        }

        // seq_len == 4
        const b1 = input[offset + 1];
        const b2 = input[offset + 2];
        const b3 = input[offset + 3];
        if ((b1 & 0xC0) != 0x80 || (b2 & 0xC0) != 0x80 || (b3 & 0xC0) != 0x80) {
            return DecodeResult{ codepoint: REPLACEMENT_CHAR, bytes_consumed: 1, valid: false };
        }
        if ((b0 == 0xF0 && b1 < 0x90) || (b0 == 0xF4 && b1 >= 0x90)) {
            return DecodeResult{ codepoint: REPLACEMENT_CHAR, bytes_consumed: 1, valid: false };
        }
        const cp: u32 = (((b0 & 0x07) as u32) << 18)
            | (((b1 & 0x3F) as u32) << 12)
            | (((b2 & 0x3F) as u32) << 6)
            | ((b3 & 0x3F) as u32);
        return DecodeResult{ codepoint: cp, bytes_consumed: 4, valid: true };
    }

    // Fast SIMD ASCII skip + full RFC 3629 UTF-8 validation
    pub static fn isValid(input: []const u8): bool {
        if (first_non_ascii := SimdScanner.firstNonAsciiIndex(input)) {
            let pos = first_non_ascii;
            while (pos < input.len) {
                const b = input[pos];
                if (b < 0x80) {
                    pos += 1;
                } else {
                    const dec_res = Utf8.decodeNext(input, pos);
                    if (!dec_res.valid) {
                        return false;
                    }
                    pos += (dec_res.bytes_consumed as usize);
                }
            }
            return true;
        }
        return true;
    }

    // Counts Unicode scalar values in valid UTF-8 slice
    pub static fn codepointCount(input: []const u8): usize {
        let count: usize = 0;
        let i: usize = 0;
        while (i < input.len) {
            if ((input[i] & 0xC0) != 0x80) {
                count += 1;
            }
            i += 1;
        }
        return count;
    }

    // Encodes a single Unicode codepoint into `out[0..4]`, returning number of bytes written (0 if invalid)
    pub static fn encodeCodepoint(cp: u32, out: []u8): usize {
        if (cp <= 0x7F) {
            if (out.len < 1) {
                return 0;
            }
            out[0] = cp as u8;
            return 1;
        }
        if (cp <= 0x7FF) {
            if (out.len < 2) {
                return 0;
            }
            out[0] = (0xC0 | (cp >> 6)) as u8;
            out[1] = (0x80 | (cp & 0x3F)) as u8;
            return 2;
        }
        if (cp >= 0xD800 && cp <= 0xDFFF) {
            return 0;
        }
        if (cp <= 0xFFFF) {
            if (out.len < 3) {
                return 0;
            }
            out[0] = (0xE0 | (cp >> 12)) as u8;
            out[1] = (0x80 | ((cp >> 6) & 0x3F)) as u8;
            out[2] = (0x80 | (cp & 0x3F)) as u8;
            return 3;
        }
        if (cp <= 0x10FFFF) {
            if (out.len < 4) {
                return 0;
            }
            out[0] = (0xF0 | (cp >> 18)) as u8;
            out[1] = (0x80 | ((cp >> 12) & 0x3F)) as u8;
            out[2] = (0x80 | ((cp >> 6) & 0x3F)) as u8;
            out[3] = (0x80 | (cp & 0x3F)) as u8;
            return 4;
        }
        return 0;
    }
}

export struct Utf16 {
    // Computes exact UTF-16 code unit length from UTF-8 input (matching Bun elementLengthUTF8IntoUTF16)
    pub static fn utf16LengthFromUtf8(input: []const u8): usize {
        let u16_len: usize = 0;
        let pos: usize = 0;
        while (pos < input.len) {
            const dec_res = Utf8.decodeNext(input, pos);
            if (dec_res.codepoint >= 0x10000) {
                u16_len += 2;
            } else {
                u16_len += 1;
            }
            pos += (dec_res.bytes_consumed as usize);
        }
        return u16_len;
    }

    // Encodes UTF-8 slice into UTF-16LE buffer `out`, returning number of u16 units written
    pub static fn encodeUtf8ToUtf16Le(input: []const u8, out: []u16): usize {
        let pos: usize = 0;
        let out_idx: usize = 0;
        while (pos < input.len) {
            const dec_res = Utf8.decodeNext(input, pos);
            const cp = dec_res.codepoint;
            if (cp < 0x10000) {
                if (out_idx >= out.len) {
                    break;
                }
                out[out_idx] = cp as u16;
                out_idx += 1;
            } else {
                if (out_idx + 1 >= out.len) {
                    break;
                }
                const adj = cp - 0x10000;
                const high: u16 = (0xD800 + (adj >> 10)) as u16;
                const low: u16 = (0xDC00 + (adj & 0x3FF)) as u16;
                out[out_idx] = high;
                out[out_idx + 1] = low;
                out_idx += 2;
            }
            pos += (dec_res.bytes_consumed as usize);
        }
        return out_idx;
    }

    // Decodes UTF-16LE slice into UTF-8 buffer `out`, returning number of bytes written
    pub static fn decodeUtf16LeToUtf8(input: []const u16, out: []u8): usize {
        let i: usize = 0;
        let out_pos: usize = 0;
        while (i < input.len) {
            const unit0 = input[i] as u32;
            let cp: u32 = unit0;
            if (unit0 >= 0xD800 && unit0 <= 0xDBFF) {
                if (i + 1 < input.len) {
                    const unit1 = input[i + 1] as u32;
                    if (unit1 >= 0xDC00 && unit1 <= 0xDFFF) {
                        cp = (0x10000 as u32) + ((unit0 - 0xD800) << 10) + (unit1 - 0xDC00);
                        i += 1;
                    } else {
                        cp = REPLACEMENT_CHAR;
                    }
                } else {
                    cp = REPLACEMENT_CHAR;
                }
            } else if (unit0 >= 0xDC00 && unit0 <= 0xDFFF) {
                cp = REPLACEMENT_CHAR;
            }
            const written = Utf8.encodeCodepoint(cp, out[out_pos..out.len]);
            if (written == 0) {
                break;
            }
            out_pos += written;
            i += 1;
        }
        return out_pos;
    }
}

// Terminal & East-Asian visible display width (absorbed from bun/src/string/immutable/visible.zig & unicode/uucode)
export struct UnicodeWidth {
    pub static fn codepointWidth(cp: u32): usize {
        if (cp == 0) {
            return 0;
        }
        // Control characters & combining marks
        if (cp < 32 || (cp >= 0x7F && cp < 0xA0)) {
            return 0;
        }
        if (cp >= 0x0300 && cp <= 0x036F) {
            return 0;
        }
        // CJK Unified Ideographs, Hangul, Kana, Fullwidth Forms & Emoji blocks
        if ((cp >= 0x1100 && cp <= 0x115F)
            || (cp >= 0x2E80 && cp <= 0xA4CF)
            || (cp >= 0xAC00 && cp <= 0xD7A3)
            || (cp >= 0xF900 && cp <= 0xFAFF)
            || (cp >= 0xFE10 && cp <= 0xFE6F)
            || (cp >= 0xFF01 && cp <= 0xFF60)
            || (cp >= 0x1F300 && cp <= 0x1FAFF))
        {
            return 2;
        }
        return 1;
    }

    // Computes visible column width, automatically stripping ANSI CSI escape codes (`\x1b[...m`)
    pub static fn visibleWidth(input: []const u8): usize {
        let width: usize = 0;
        let i: usize = 0;
        while (i < input.len) {
            // Skip ANSI escape sequence `\x1b[...m`
            if (input[i] == 0x1B && i + 1 < input.len && input[i + 1] == 91) {
                i += 2;
                while (i < input.len) {
                    const ch = input[i];
                    i += 1;
                    if ((ch >= 64 && ch <= 126)) {
                        break;
                    }
                }
            } else {
                const dec_res = Utf8.decodeNext(input, i);
                width += UnicodeWidth.codepointWidth(dec_res.codepoint);
                i += (dec_res.bytes_consumed as usize);
            }
        }
        return width;
    }
}
