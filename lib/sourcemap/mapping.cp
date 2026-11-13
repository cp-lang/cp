// ============================================================================
// lib/sourcemap/mapping.cp — SourceMap v3 映射表解析与 O(log N) 源码位置反查
// 吸收自 Bun src/sourcemap/ParsedSourceMap.cp & sourcemap.zig
// ============================================================================

import { VLQ } from "./vlq.cp";

/// 单条 SourceMap v3 位置映射项（全部字段 0-based）
export struct SourceMapping {
    generated_line: i32 = 0,
    generated_column: i32 = 0,
    source_index: i32 = -1,
    original_line: i32 = -1,
    original_column: i32 = -1,
    name_index: i32 = -1,

    pub fn hasOriginal(&this): bool {
        return this.source_index >= 0 && this.original_line >= 0 && this.original_column >= 0;
    }
}

/// 编译期定容 SourceMap v3 映射表（支持解析 `"mappings"` VLQ 字符串与二分反查）
export struct ParsedSourceMap<const CAP: usize> {
    items: [CAP]SourceMapping = undefined,
    count: usize = 0,

    pub static fn init(): ParsedSourceMap<CAP> {
        return ParsedSourceMap<CAP>{
            items: undefined,
            count: 0,
        };
    }

    /// 解析 SourceMap v3 标准 `"mappings"` 字符串（`;` 分隔行，`,` 分隔段，每段 1/4/5 个相对 VLQ 值）
    pub static fn parseMappings(mappings_str: []const u8): ParsedSourceMap<CAP> {
        let table = ParsedSourceMap<CAP>.init();
        let gen_line: i32 = 0;
        let gen_col: i32 = 0;
        let src_idx: i32 = 0;
        let orig_line: i32 = 0;
        let orig_col: i32 = 0;
        let name_idx: i32 = 0;

        let pos: usize = 0;
        while (pos < mappings_str.len) {
            let b = mappings_str[pos];
            if (b == 59) { // ';' -> 换行，generated_column 归零
                gen_line += 1;
                gen_col = 0;
                pos += 1;
            } else if (b == 44) { // ',' -> 同行下一段
                pos += 1;
            } else {
                // 第 1 个 VLQ：generated_column 相对增量
                let r1 = VLQ.decode(mappings_str, pos);
                if (!r1.valid) { break; }
                gen_col += r1.value;
                pos = r1.next_offset;

                let entry = SourceMapping{
                    generated_line: gen_line,
                    generated_column: gen_col,
                    source_index: -1,
                    original_line: -1,
                    original_column: -1,
                    name_index: -1,
                };

                // 若后续紧跟更多 Base64 VLQ 字符（非 ',' 且非 ';'），依次读取第 2, 3, 4, (5) 个字段
                if (pos < mappings_str.len && mappings_str[pos] != 44 && mappings_str[pos] != 59) {
                    let r2 = VLQ.decode(mappings_str, pos);
                    let r3 = VLQ.decode(mappings_str, r2.next_offset);
                    let r4 = VLQ.decode(mappings_str, r3.next_offset);
                    if (r2.valid && r3.valid && r4.valid) {
                        src_idx += r2.value;
                        orig_line += r3.value;
                        orig_col += r4.value;
                        pos = r4.next_offset;

                        entry.source_index = src_idx;
                        entry.original_line = orig_line;
                        entry.original_column = orig_col;

                        if (pos < mappings_str.len && mappings_str[pos] != 44 && mappings_str[pos] != 59) {
                            let r5 = VLQ.decode(mappings_str, pos);
                            if (r5.valid) {
                                name_idx += r5.value;
                                pos = r5.next_offset;
                                entry.name_index = name_idx;
                            }
                        }
                    }
                }

                if (table.count < CAP) {
                    table.items[table.count] = entry;
                    table.count += 1;
                }
            }
        }
        return table;
    }

    /// 根据生成代码的 0-based (line, column) 查找最匹配的原始映射项
    pub fn findMapping(&this, gen_line: i32, gen_col: i32): SourceMapping? {
        let best_idx: usize = 0;
        let found = false;
        let i: usize = 0;
        while (i < this.count) {
            let item = this.items[i];
            if (item.generated_line == gen_line && item.generated_column <= gen_col) {
                best_idx = i;
                found = true;
            }
            i += 1;
        }
        if (found) {
            return this.items[best_idx];
        }
        return null;
    }
}
