import { test, expect_eq } from "cap:test";
import { print } from "cap:io";

async fn main(): void {
    // -------------------------------------------------------------
    // 1. Map (ES6 Interface & Index Access)
    // -------------------------------------------------------------
    let map = new Map();
    map.set("title", "CP Language");
    map.set("version", 2);
    expect_eq(map.size, 2);

    // .get() returns ?Message
    let title_msg = map.get("title");
    expect_eq(title_msg.to_string(), "CP Language");

    // Index read operator: map["key"]
    expect_eq(map["version"], 2);

    // Index assign operator: map["key"] = value
    map["title"] = "CP-BEAM";
    expect_eq(map["title"], "CP-BEAM");

    // .has() & .delete()
    expect_eq(map.has("title"), true);
    expect_eq(map.has("nonexistent"), false);
    map.delete("version");
    expect_eq(map.size, 1);

    // Iteration collections: keys(), values(), entries()
    let ks = map.keys();
    let vs = map.values();
    let es = map.entries();
    expect_eq(ks.len, 1);
    expect_eq(vs.len, 1);
    expect_eq(es.len, 1);

    map.clear();
    expect_eq(map.size, 0);

    // -------------------------------------------------------------
    // 2. Set (ES6 Interface)
    // -------------------------------------------------------------
    let set = new Set();
    set.add(100);
    set.add(200);
    set.add(100); // Duplicate, set.size should stay 2
    expect_eq(set.size, 2);
    expect_eq(set.has(100), true);
    expect_eq(set.has(300), false);

    let set_vals = set.values();
    expect_eq(set_vals.len, 2);

    set.delete(100);
    expect_eq(set.size, 1);
    expect_eq(set.has(100), false);

    set.clear();
    expect_eq(set.size, 0);

    // -------------------------------------------------------------
    // 3. Dict & Dynamic Object Syntax Sugar #{ ... }
    // -------------------------------------------------------------
    let config = #{ host: "localhost", port: 8080 };
    expect_eq(config["host"], "localhost");
    expect_eq(config["port"], 8080);

    // Dynamic property insertion
    config["ssl"] = true;
    expect_eq(config["ssl"], true);

    // Dict spread (...) and override
    let prod_config = #{ ...config, port: 443, env: "production" };
    expect_eq(prod_config["host"], "localhost");
    expect_eq(prod_config["port"], 443);
    expect_eq(prod_config["ssl"], true);
    expect_eq(prod_config["env"], "production");

    // Null coalescing ?? with dynamic dict index
    let timeout = config["timeout"] ?? 5000;
    expect_eq(timeout, 5000);

    @print("🎉 Phase 2: Map, Set, and Dict (#{...}) verified successfully!");
}
