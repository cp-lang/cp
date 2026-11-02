struct Base64 {
    pad_char: u8?

    // 1. 省略只读借用参数 `_`；使用极简推导错误联合 `!usize`
    fn calcSizeUpperBound(len: usize): !usize {
        return len * 4 / 3;
    }

    // 2. 省略只读借用参数 `_`；省略 `_.` 直接访问内部属性与方法；使用默认可变的 `let result`（无需 mut）
    fn calcSizeForSlice(source: string): !usize {
        let source_len = source.len;
        let result = calcSizeUpperBound(source_len)?;

        if (pad_char != null) {
            let pad = pad_char ?? 0;
            if (source_len >= 1 && source[source_len - 1] == pad) {
                result -= 1;
            }
            if (source_len >= 2 && source[source_len - 2] == pad) {
                result -= 1;
            }
        }
        return result;
    }

    // 3. 静态函数明确标记 static，与实例方法彻底正交隔离
    static fn defaultPadChar(): u8 {
        return 61; // '='
    }
}

fn main() {
    let default_pad = Base64.defaultPadChar();
    let b64 = Base64{ pad_char: default_pad };
    let s = "YWJj=";
    let size = b64.calcSizeForSlice(s) catch 0;

    if (size != 5) {
        @print("Test failed: expected 5, got {d}", size);
    } else {
        @print("Base64 slice calculation tests passed successfully!");
    }
}
