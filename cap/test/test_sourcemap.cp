import { test, expect_eq } from "cap:test";
import {
    VLQ,
    LineOffsetTable,
    ParsedSourceMap
} from "cap:sourcemap";

fn main(): void {
    test("1. Base64 VLQ encode & decode signed integers roundtrip", () => {
        let v0 = VLQ.encode(0);
        expect_eq(v0.slice(), "A");

        let v1 = VLQ.encode(1);
        expect_eq(v1.slice(), "C");

        let vm1 = VLQ.encode(-1);
        expect_eq(vm1.slice(), "D");

        let v16 = VLQ.encode(16);
        expect_eq(v16.slice(), "gB");

        let r0 = VLQ.decode("A", 0);
        expect_eq(r0.valid, true);
        expect_eq(r0.value, 0);
        expect_eq(r0.next_offset, 1 as usize);

        let r_neg = VLQ.decode("D", 0);
        expect_eq(r_neg.valid, true);
        expect_eq(r_neg.value, -1);

        let r_big = VLQ.encode(-12345);
        let d_big = VLQ.decode(r_big.slice(), 0);
        expect_eq(d_big.valid, true);
        expect_eq(d_big.value, -12345);
    });

    test("2. LineOffsetTable O(log N) binary search line & column resolution", () => {
        let src = "fn add(a: i32, b: i32): i32 {\n    return a + b;\n}\n";
        let table = LineOffsetTable<32>.build(src);
        expect_eq(table.line_count, 4 as usize);

        let loc0 = table.resolveLocation(3);
        expect_eq(loc0.line, 0 as u32);
        expect_eq(loc0.column, 3 as u32);

        // 第二行起始于字节偏移 30 ("    return a + b;\n")
        let loc1 = table.resolveLocation(34);
        expect_eq(loc1.line, 1 as u32);
        expect_eq(loc1.column, 4 as u32);
    });

    test("3. ParsedSourceMap v3 mappings string parser & position lookup", () => {
        // "AAAA,IAAIA;ECAA"
        // Segment 1 (line 0): AAAA -> gen_col=0, src=0, orig_line=0, orig_col=0
        // Segment 2 (line 0): IAAIA -> gen_col+=4(4), src+=0(0), orig_line+=0(0), orig_col+=4(4), name+=0(0)
        // ';' -> line 1, gen_col=0
        // Segment 3 (line 1): ECAA -> gen_col+=2(2), src+=1(1), orig_line+=0(0), orig_col+=0(0)
        let sm = ParsedSourceMap<16>.parseMappings("AAAA,IAAIA;ECAA");
        expect_eq(sm.count, 3 as usize);

        let m0_opt = sm.findMapping(0, 2);
        expect_eq(m0_opt != null, true);
        if (m0_opt) |m0| {
            expect_eq(m0.generated_column, 0);
            expect_eq(m0.source_index, 0);
            expect_eq(m0.original_line, 0);
            expect_eq(m0.original_column, 0);
        }

        let m1_opt = sm.findMapping(0, 6);
        expect_eq(m1_opt != null, true);
        if (m1_opt) |m1| {
            expect_eq(m1.generated_column, 4);
            expect_eq(m1.original_column, 4);
            expect_eq(m1.name_index, 0);
        }

        let m2_opt = sm.findMapping(1, 5);
        expect_eq(m2_opt != null, true);
        if (m2_opt) |m2| {
            expect_eq(m2.generated_line, 1);
            expect_eq(m2.generated_column, 2);
            expect_eq(m2.source_index, 1);
            expect_eq(m2.original_line, 0);
            expect_eq(m2.original_column, 4);
        }
    });
}
