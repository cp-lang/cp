import { print } from "cap:io";

zig {
    pub const MyConfig = struct {
        id: u32,
    };
    pub fn getMagicNumber(cfg: MyConfig) u32 {
        return cfg.id * 2;
    }
}

fn computeWithEscape(cfg: MyConfig, unused_flag: bool): u32 {
    let mut res: u32 = 0;
    zig {
        res = getMagicNumber(cfg);
    }
    return res;
}

fn main(): void {
    let cfg = MyConfig{ id: 21 };
    let result = computeWithEscape(cfg, false);
    if (result != 42) {
        print("Expected 42\n");
        return;
    }
    print("Escape type and unused param optimization passed successfully!\n");
}
