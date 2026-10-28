import { print } from "cap:io";

fn test_basic_obj_rest() {
    let user = { id: 1001, name: "Alice", role: "Admin", score: 95 };

    // 1. Extract `id`, gather remaining properties into `rest`
    let { id, ...rest } = user;
    print(id);
    print(rest.name);
    print(rest.role);
    print(rest.score);

    // 2. Extract `name`, ignore rest with bare `...`
    let { name, ... } = user;
    print(name);

    // 3. Extract `score`, ignore rest with `...*`
    let { score, ...* } = user;
    print(score);
}

fn test_switch_obj_rest() {
    let pkg = { kind: "data", code: 200, payload: "OK" };

    let msg = switch (pkg) {
        { kind, ...extra } => extra.payload,
        * => "Unknown"
    };
    print(msg);
}

fn test_for_obj_rest() {
    let items = [
        { id: 1, val: 10, tag: "A" },
        { id: 2, val: 20, tag: "B" }
    ];

    for { id, ...rest } of items {
        print(id);
        print(rest.val);
        print(rest.tag);
    }
}

fn main() {
    print("=== Testing Object Destructuring with ...rest ===");
    test_basic_obj_rest();
    test_switch_obj_rest();
    test_for_obj_rest();
    print("ALL OBJECT DESTRUCTURING REST TESTS PASSED!");
}
