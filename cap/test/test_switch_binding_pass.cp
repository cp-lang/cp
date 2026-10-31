import { expect_eq } from "cap:test";

enum Task {
    Idle,
    Text: string,
    Move: [i32, i32],
    Compute: { batch: usize, priority: i32 }
}

fn test_enum_matching(t: Task): i32 {
    return switch (t) {
        .Idle => 1,
        .Text text => text.len as i32,
        .Move [x, y] => x + y,
        .Compute { batch, priority } => (batch as i32) + priority,
    };
}

fn test_enum_rest_capture(t: Task): usize {
    return switch (t) {
        .Compute { ...fields } => fields.batch,
        * => 0 as usize,
    };
}

fn test_walrus_range_binding(score: i32): i32 {
    return switch (score) {
        s := 90..=100 => s * 2,
        s := 60..=89 => s + 10,
        * => 0,
    };
}

fn main(): ?void {
    // 1. 无载荷变体构造与匹配
    let t_idle: Task = .Idle;
    expect_eq(test_enum_matching(t_idle), 1);

    // 2. 单类型载荷构造与匹配
    let t_text: Task = .Text("hello");
    expect_eq(test_enum_matching(t_text), 5);

    // 3. 元组载荷构造与方括号解构匹配 (.Move [x, y])
    let t_move: Task = .Move [15, 25];
    expect_eq(test_enum_matching(t_move), 40);

    // 4. 结构体载荷构造与花括号解构匹配 (.Compute { batch, priority })
    let t_comp: Task = .Compute { batch: 200, priority: 7 };
    expect_eq(test_enum_matching(t_comp), 207);

    // 5. 结构体载荷全捕获 (.Compute { ...fields })
    expect_eq(test_enum_rest_capture(t_comp), 200);

    // 6. 区间模式 Walrus 变量绑定 (s := 90..=100)
    expect_eq(test_walrus_range_binding(95), 190);
    expect_eq(test_walrus_range_binding(70), 80);
    expect_eq(test_walrus_range_binding(50), 0);

    @print("All switch and tagged union tests passed successfully!");
}
