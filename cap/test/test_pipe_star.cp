import { print } from "cap:io";

fn add(a: i32, b: i32): i32 {
    return a + b;
}

fn mul(a: i32, b: i32): i32 {
    return a * b;
}

fn main() {
    let x = 10;
    // Test * as pipeline placeholder in second position:
    let res1 = x |> add(5, *);
    print(res1);

    // Test * as pipeline placeholder in first position:
    let res2 = res1 |> mul(*, 2);
    print(res2);

    // Test . as pipeline placeholder:
    let res3 = res2 |> add(10, .);
    print(res3);
}
