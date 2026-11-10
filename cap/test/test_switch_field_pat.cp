import { print } from "cap:io";

pub const Tag = enum {
    foo,
    bar,
    baz,
};

pub fn checkTag(tag: Tag): i32 {
    switch (tag) {
        @field(Tag, "foo") => {
            return 100;
        },
        @field(Tag, "bar") => {
            return 200;
        },
        else => {
            return 300;
        },
    }
}

fn main(): void {
    let r1 = checkTag(Tag.foo);
    let r2 = checkTag(Tag.bar);
    let r3 = checkTag(Tag.baz);

    if (r1 != 100) {
        @panic("checkTag(Tag.foo) failed");
    }
    if (r2 != 200) {
        @panic("checkTag(Tag.bar) failed");
    }
    if (r3 != 300) {
        @panic("checkTag(Tag.baz) failed");
    }

    print("switch @field pattern match test passed!\n");
}
