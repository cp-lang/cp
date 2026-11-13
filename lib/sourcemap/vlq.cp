// ============================================================================
// lib/sourcemap/vlq.cp — Base64 VLQ (Variable-Length Quantity) 编解码器
// 吸收自 Bun src/sourcemap/VLQ.cp
// ============================================================================

const BASE64_CHARS: []const u8 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Base64 VLQ 编码结果（最多 7 字节栈内联，零堆分配）
export struct VLQ {
    bytes: [7]u8 = [0, 0, 0, 0, 0, 0, 0],
    len: u8 = 0,

    pub fn slice(*this): []const u8 {
        let n = this.len as usize;
        return this.bytes[0..n];
    }

    pub static fn encode(value: i32): VLQ {
        let out = VLQ{
            bytes: [0, 0, 0, 0, 0, 0, 0],
            len: 0,
        };
        let vlq: u32 = if (value >= 0)
            ((value as u32) << 1)
        else
            ((((-value) as u32) << 1) | (1 as u32));

        let step: usize = 0;
        while (step < 7) {
            let digit: u32 = vlq & 31;
            vlq = vlq >> 5;
            if (vlq != 0) {
                digit = digit | 32;
            }
            let idx = out.len as usize;
            out.bytes[idx] = BASE64_CHARS[digit as usize];
            out.len += 1;
            if (vlq == 0) {
                return out;
            }
            step += 1;
        }
        return out;
    }

    pub static fn decode(encoded: []const u8, start: usize): VLQResult {
        let shift: u5 = 0;
        let vlq: u32 = 0;
        let pos = start;
        let step: usize = 0;

        while (pos < encoded.len && step < 7) {
            let b = encoded[pos];
            let sextet_opt = decodeBase64Sextet(b);
            if (sextet_opt == null) {
                break;
            }
            let digit = (sextet_opt ?? 0) as u32;
            vlq = vlq | ((digit & 31) << shift);
            pos += 1;
            step += 1;

            if ((digit & 32) == 0) {
                let raw = (vlq >> 1) as i32;
                let signed_val: i32 = if ((vlq & 1) == 0) raw else (-raw);
                return VLQResult{
                    value: signed_val,
                    next_offset: pos,
                    valid: true,
                };
            }
            if (shift <= 25) {
                shift += 5;
            }
        }

        return VLQResult{
            value: 0,
            next_offset: pos,
            valid: false,
        };
    }
}

export struct VLQResult {
    value: i32 = 0,
    next_offset: usize = 0,
    valid: bool = false,
}

fn decodeBase64Sextet(b: u8): u8? {
    if (b >= 65 && b <= 90) { return b - 65; }         // 'A'..'Z' -> 0..25
    if (b >= 97 && b <= 122) { return b - 97 + 26; }   // 'a'..'z' -> 26..51
    if (b >= 48 && b <= 57) { return b - 48 + 52; }    // '0'..'9' -> 52..61
    if (b == 43) { return 62; }                        // '+' -> 62
    if (b == 47) { return 63; }                        // '/' -> 63
    return null;
}
