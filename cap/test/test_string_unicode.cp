import { test, expect_eq } from "cap:test";
import {
    SimdScanner,
    REPLACEMENT_CHAR,
    Utf8,
    Utf16,
    UnicodeWidth,
    HashedString,
    SmolStr,
    StringBuilder,
    StringEscape,
} from "cap:string";

fn main(): void {
    test("1. SimdScanner 16-byte @Vector(16, u8) ASCII & byte scanning", () => {
        const ascii_text = "0123456789abcdef_hello_simd_world!";
        expect_eq(SimdScanner.isAscii(ascii_text), true);
        expect_eq(SimdScanner.firstNonAsciiIndex(ascii_text) == null, true);

        const mixed_text = "0123456789abcdef0123_中文_end";
        expect_eq(SimdScanner.isAscii(mixed_text), false);
        if (non_ascii := SimdScanner.firstNonAsciiIndex(mixed_text)) {
            expect_eq(non_ascii, 21);
        } else {
            expect_eq(true, false);
        }

        if (idx := SimdScanner.indexOfByte(ascii_text, 95)) { // '_' = 95
            expect_eq(idx, 16);
        } else {
            expect_eq(true, false);
        }

        if (idx2 := SimdScanner.indexOfAny2(ascii_text, 33, 95)) { // '!' = 33, '_' = 95
            expect_eq(idx2, 16);
        } else {
            expect_eq(true, false);
        }

        expect_eq(SimdScanner.countByte(ascii_text, 95), 3);
        expect_eq(SimdScanner.eqlCaseInsensitiveAscii("Content-Type", "content-type"), true);
    });

    test("2. Utf8 & Utf16 validation, transcoding, and surrogate pairs", () => {
        const valid_utf8 = "Hello, 世界! 🚀";
        expect_eq(Utf8.isValid(valid_utf8), true);
        expect_eq(Utf8.codepointCount(valid_utf8), 12);

        // "Hello, " (7) + "世界" (2 BMP chars = 2 u16) + "! " (2) + "🚀" (U+1F680 = 2 surrogate u16s) = 13 u16 units
        expect_eq(Utf16.utf16LengthFromUtf8(valid_utf8), 13);

        let u16_buf: [32]u16 = [0; 32];
        const u16_written = Utf16.encodeUtf8ToUtf16Le(valid_utf8, u16_buf[0..32]);
        expect_eq(u16_written, 13);
        // Check 🚀 surrogate pair at indices 11 and 12 (0xD83D, 0xDE80)
        expect_eq(u16_buf[11], 0xD83D);
        expect_eq(u16_buf[12], 0xDE80);

        let roundtrip_buf: [64]u8 = [0; 64];
        const utf8_written = Utf16.decodeUtf16LeToUtf8(u16_buf[0..u16_written], roundtrip_buf[0..64]);
        expect_eq(roundtrip_buf[0..utf8_written], valid_utf8);

        // Malformed UTF-8 (invalid lead byte 0xFF)
        const bad_bytes: [3]u8 = [65, 0xFF, 66];
        expect_eq(Utf8.isValid(bad_bytes[0..3]), false);
        const dec_bad = Utf8.decodeNext(bad_bytes[0..3], 1);
        expect_eq(dec_bad.valid, false);
        expect_eq(dec_bad.codepoint, REPLACEMENT_CHAR);
    });

    test("3. UnicodeWidth East-Asian Wide / Emoji & ANSI escape stripping", () => {
        expect_eq(UnicodeWidth.visibleWidth("hello"), 5);
        // "你好" = 2 CJK ideographs * 2 columns = 4
        expect_eq(UnicodeWidth.visibleWidth("你好"), 4);
        // ANSI colored "\x1b[32mPASS\x1b[0m" should have visible width 4
        expect_eq(UnicodeWidth.visibleWidth("\x1b[32mPASS\x1b[0m"), 4);
    });

    test("4. HashedString, SmolStr (15-byte inline), and two-pass StringBuilder", () => {
        const h1 = HashedString.init("authorization");
        const h2 = HashedString.init("authorization");
        const h3 = HashedString.init("content-length");
        expect_eq(h1.eql(h2), true);
        expect_eq(h1.eql(h3), false);

        let inline_str = SmolStr.init("short-stack-str");
        expect_eq(inline_str.isInlined(), true);
        expect_eq(inline_str.len(), 15);
        expect_eq(inline_str.slice(), "short-stack-str");
        inline_str.release();

        let heap_str = SmolStr.init("this-string-exceeds-fifteen-bytes");
        expect_eq(heap_str.isInlined(), false);
        expect_eq(heap_str.len(), 33);
        expect_eq(heap_str.slice(), "this-string-exceeds-fifteen-bytes");
        heap_str.release();

        let sb = StringBuilder.init();
        sb.count("https://");
        sb.count("cap.dev");
        sb.count("/v1/string");
        sb.allocate();
        _ = sb.append("https://");
        _ = sb.append("cap.dev");
        _ = sb.append("/v1/string");
        expect_eq(sb.slice(), "https://cap.dev/v1/string");
        sb.release();
    });

    test("5. StringEscape HTML entities & RegExp metacharacter escaping", () => {
        const raw_html = "<div class=\"box\">A & B</div>";
        expect_eq(StringEscape.needsHtmlEscape(raw_html), true);
        let html_out: [128]u8 = [0; 128];
        const html_len = StringEscape.escapeHtml(raw_html, html_out[0..128]);
        expect_eq(html_out[0..html_len], "&lt;div class=&quot;box&quot;&gt;A &amp; B&lt;/div&gt;");

        const raw_re = "a+b*c-d.e";
        let re_out: [64]u8 = [0; 64];
        const re_len = StringEscape.escapeRegExp(raw_re, re_out[0..64]);
        expect_eq(re_out[0..re_len], "a\\+b\\*c\\x2dd\\.e");
    });
}
