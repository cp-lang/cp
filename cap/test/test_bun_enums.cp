import { print } from "cap:io";

export enum FetchRequestMode: u2 {
    @"same-origin",
    @"no-cors",
    cors,
    navigate,

    fn isSameOrigin(self): bool {
        return self == FetchRequestMode.@"same-origin";
    }

    zig {
        pub fn getCustomTag(mode: FetchRequestMode) u8 {
            return @intFromEnum(mode);
        }
    }
}

fn checkMode(mode: FetchRequestMode): i32 {
    return switch (mode) {
        .@"same-origin", .@"no-cors" => 1,
        .cors => 2,
        .navigate => 3,
        * => 0
    };
}

fn main(): void {
    let mode: FetchRequestMode = FetchRequestMode.@"same-origin";
    if (!mode.isSameOrigin()) {
        print("Expected mode to be same-origin\n");
        return;
    }

    let code1 = checkMode(mode);
    let code2 = checkMode(FetchRequestMode.cors);
    if (code1 != 1 || code2 != 2) {
        print("Switch multi-match failed\n");
        return;
    }

    print("Bun enum and multi-match features verified successfully!\n");
}
