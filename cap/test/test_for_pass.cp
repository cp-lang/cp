import { test, expect_eq } from "cap:test";

class Player {
    id: i32;
    score: i32;
    fn init(id: i32, score: i32) {
        this.id = id;
        this.score = score;
    }
}

async fn main(): void {
    // -------------------------------------------------------------
    // 1. Range Iteration (0..N and 1..=N)
    // -------------------------------------------------------------
    let sum_range = 0;
    for i of 0..5 {
        sum_range = sum_range + i;
    }
    expect_eq(sum_range, 10);

    let sum_inclusive = 0;
    for i of 1..=5 {
        sum_inclusive = sum_inclusive + i;
    }
    expect_eq(sum_inclusive, 15);

    // Dynamic range bounds
    let start = 3;
    let end = 7;
    let sum_dynamic = 0;
    for i of start..end {
        sum_dynamic = sum_dynamic + i;
    }
    expect_eq(sum_dynamic, 18);

    // Parentheses syntax for range: for (i of 0..3)
    let sum_paren = 0;
    for (i of 0..3) {
        sum_paren = sum_paren + i;
    }
    expect_eq(sum_paren, 3);

    @print("✔ Range iteration tests passed!");

    // -------------------------------------------------------------
    // 2. Read-Only Borrow: for x of list
    // -------------------------------------------------------------
    let nums: i32[] = [10, 20, 30];
    let sum_nums = 0;
    for x of nums {
        sum_nums = sum_nums + x;
    }
    expect_eq(sum_nums, 60);

    // Verify nums is still intact and can be iterated again
    let count = 0;
    for (let x of nums) {
        count = count + 1;
    }
    expect_eq(count, 3);

    @print("✔ Read-only borrow iteration tests passed!");

    // -------------------------------------------------------------
    // 3. Mutable Borrow: for &x of list (in-place modification)
    // -------------------------------------------------------------
    let mut_list: i32[] = [1, 2, 3];
    __zig__ {
        ctx.mut_list = try ctx.arena.?.allocator().dupe(i32, ctx.mut_list);
    }
    for &x of mut_list {
        x = x * 10;
    }
    expect_eq(mut_list[0], 10);
    expect_eq(mut_list[1], 20);
    expect_eq(mut_list[2], 30);

    // Mutable with parentheses syntax: for (&x of mut_list)
    for (&x of mut_list) {
        x = x + 5;
    }
    expect_eq(mut_list[0], 15);
    expect_eq(mut_list[1], 25);
    expect_eq(mut_list[2], 35);

    @print("✔ Mutable borrow iteration tests passed!");

    // -------------------------------------------------------------
    // 4. Ownership Transfer: for move x of list
    // -------------------------------------------------------------
    let move_list: i32[] = [100, 200, 300];
    let sum_moved = 0;
    for move x of move_list {
        sum_moved = sum_moved + x;
    }
    expect_eq(sum_moved, 600);

    let move_list_paren: i32[] = [5, 15];
    let sum_moved_paren = 0;
    for (move x of move_list_paren) {
        sum_moved_paren = sum_moved_paren + x;
    }
    expect_eq(sum_moved_paren, 20);

    @print("✔ Ownership transfer (move) iteration tests passed!");

    // -------------------------------------------------------------
    // 5. Pattern Destructuring: Object ({ id, score })
    // -------------------------------------------------------------
    let players = [
        new Player(1, 100),
        new Player(2, 200)
    ];
    let total_score = 0;
    for { id, score } of players {
        total_score = total_score + score;
    }
    expect_eq(total_score, 300);

    @print("✔ Pattern destructuring iteration tests passed!");
    @print("=== All Iteration Tests Passed Successfully! ===");
}
