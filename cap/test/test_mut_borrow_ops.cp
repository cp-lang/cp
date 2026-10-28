import { print } from "cap:io";

fn mutate_int(*x: i32, val: i32): void {
    x.* = val;
}

fn mutate_arr(*arr: i32[], val: i32): void {
    arr.push(val);
    arr[0] = 99;
}

fn mutate_map(*m: Map<string, i32>): void {
    m.set("k2", 200);
}

fn mutate_set(*s: Set<string>): void {
    s.add("item2");
}

fn mutate_dict(*d: Dict<string, i32>): void {
    d.set("d2", 88);
    d["d1"] = 77;
}

fn main(): void {
    print("=== 1. Primitive Mutable Borrow (*v & p.* = 20) ===");
    let v = 10;
    let p = *v;
    p.* = 20;
    print(`v after p.* = 20: ${v}`);
    mutate_int(*v, 35);
    print(`v after mutate_int(*v, 35): ${v}`);

    print("=== 2. Object Literal Mutable Borrow (*v & p.a = 10) ===");
    let v = {a: 10, b: 20};
    let p = *v;
    p.a = 100;
    p.b = 200;
    print(`p.a=${p.a}, v.a=${v.a}, v.b=${v.b}`);

    print("=== 3. Array / Vec Mutable Borrow (*v & p.push(30)) ===");
    let v = [10, 20];
    let p = *v;
    p.push(30);
    p[1] = 25;
    let popped = p.pop();
    print(`popped=${popped}, v.length=${v.length}, v[0]=${v[0]}, v[1]=${v[1]}`);
    mutate_arr(*v, 40);
    print(`v after mutate_arr: len=${v.length}, v[0]=${v[0]}, v[2]=${v[2]}`);

    print("=== 4. Mutable For Iteration (*item of v) ===");
    for *item of v {
        item = item + 1;
    }
    print(`v after for *item: v[0]=${v[0]}, v[1]=${v[1]}, v[2]=${v[2]}`);

    print("=== 5. Map / Set / Dict Mutable Borrow Operations ===");
    let m = new Map<string, i32>();
    let pm = *m;
    pm.set("k1", 100);
    pm["k3"] = 300;
    pm.delete("k1");
    mutate_map(*m);
    print(`Map size=${m.size}, k2=${m.get("k2")}, k3=${m["k3"]}`);

    let s = new Set<string>();
    let ps = *s;
    ps.add("item1");
    mutate_set(*s);
    print(`Set size=${s.size}, has item2=${s.has("item2")}`);
    s.clear();
    print(`Set size after clear=${s.size}`);

    let d = #{ d1: 11 };
    let pd = *d;
    pd.set("d3", 33);
    mutate_dict(*d);
    print(`Dict size=${d.size}, d1=${d["d1"]}, d2=${d.get("d2")}`);
    d.clear();
    print(`Dict size after clear=${d.size}`);

    print("ALL MUTABLE BORROW CONTAINER TESTS PASSED!");
}
