// ============================================================================
// lib/sourcemap/line_offset.cp — 源码行起始字节偏移表与二分行号/列号定位器
// 吸收自 Bun src/sourcemap/LineOffsetTable.cp
// ============================================================================

export struct SourceLocation {
    line: u32 = 0,      // 0-based line index
    column: u32 = 0,    // 0-based byte column within line
}

/// 零堆分配编译期定容行偏移查找表（支持 O(log N) 二分字节偏移定位）
export struct LineOffsetTable<const MAX_LINES: usize> {
    offsets: [MAX_LINES]u32 = undefined,
    line_count: usize = 0,

    pub static fn build(contents: []const u8): LineOffsetTable<MAX_LINES> {
        let table = LineOffsetTable<MAX_LINES>{
            offsets: undefined,
            line_count: 0,
        };
        if (MAX_LINES == 0) {
            return table;
        }
        table.offsets[0] = 0;
        table.line_count = 1;

        let i: usize = 0;
        while (i < contents.len) {
            if (contents[i] == 10) { // '\n'
                if (table.line_count < MAX_LINES && i + 1 <= contents.len) {
                    table.offsets[table.line_count] = (i + 1) as u32;
                    table.line_count += 1;
                }
            }
            i += 1;
        }
        return table;
    }

    /// 二分查找给定字节偏移对应的 0-based 行号
    pub fn findLine(&this, byte_offset: u32): u32 {
        if (this.line_count == 0) { return 0; }
        let low: usize = 0;
        let count: usize = this.line_count;
        while (count > 0) {
            let step = count / 2;
            let mid = low + step;
            if (this.offsets[mid] <= byte_offset) {
                low = mid + 1;
                count = count - step - 1;
            } else {
                count = step;
            }
        }
        if (low == 0) { return 0; }
        return (low - 1) as u32;
    }

    /// 将字节偏移转换为 0-based (line, column)
    pub fn resolveLocation(&this, byte_offset: u32): SourceLocation {
        let line_idx = this.findLine(byte_offset);
        let line_start = this.offsets[line_idx as usize];
        let col: u32 = if (byte_offset >= line_start) (byte_offset - line_start) else (0 as u32);
        return SourceLocation{
            line: line_idx,
            column: col,
        };
    }
}
