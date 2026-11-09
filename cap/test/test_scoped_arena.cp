import { print } from "cap:io";

class BufferNode {
    val: i32;
    tag: string;

    fn get_val(): i32 {
        return this.val;
    }
}

pub fn main(): void {
    print("Testing Scoped Arena (P0 Item 3)...");

    // 1. Basic arena statement block
    arena {
        let msg = `Scoped arena allocation inside block: ${100 + 23}`;
        print(msg);
        let node = new BufferNode(42, "arena_node");
        @expect_eq(node.get_val(), 42);
    }
    print("Test 1 Passed: Basic arena statement block works");

    // 2. Expression arena block returning a value
    let computed_val: i32 = arena {
        let a = 10;
        let b = 20;
        let temp_node = new BufferNode(a + b, "temp");
        temp_node.val * 2
    };
    @expect_eq(computed_val, 60);
    print("Test 2 Passed: Expression arena block returning scalar value works");

    // 3. Nested arena blocks
    let nested_res: i32 = arena {
        let outer_val = 5;
        let inner_val = arena {
            let inner_node = new BufferNode(outer_val * 10, "inner");
            inner_node.val + 7
        };
        inner_val * 2
    };
    @expect_eq(nested_res, 114);
    print("Test 3 Passed: Nested arena blocks work");

    // 4. Raw dynamic allocation with @alloc in arena
    arena {
        let raw_ptr = @alloc(64);
        raw_ptr[0] = 77;
        raw_ptr[63] = 88;
        @expect_eq(raw_ptr[0], 77);
        @expect_eq(raw_ptr[63], 88);
    }
    print("Test 4 Passed: Raw allocation @alloc inside arena works");

    // 5. Dict creation inside arena
    arena {
        let d = new Dict();
        d.set("alpha", 123);
        d.set("beta", 456);
        @expect_eq(d.get("alpha").?.to_int(), 123);
        @expect_eq(d.get("beta").?.to_int(), 456);
    }
    print("Test 5 Passed: Dict container allocation inside arena works");

    print("All Scoped Arena tests passed successfully!");
}
