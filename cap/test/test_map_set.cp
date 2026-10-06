import { test, expect_eq } from "cap:test";
import { print } from "cap:io";

async fn main(): void {
    let map = new Map();
    map.set("name", "CP-BEAM");
    map.set("version", 1.0);
    map.set("awesome", true);
    
    expect_eq(map.size, 3);
    
    let name_msg = map.get("name");
    expect_eq(name_msg.to_string(), "CP-BEAM");
    
    let v_msg = map.get("version");
    let awesome_msg = map.get("awesome");
    
    expect_eq(map.has("awesome"), true);
    expect_eq(map.has("nope"), false);
    
    map.delete("version");
    expect_eq(map.size, 2);
    
    map.clear();
    expect_eq(map.size, 0);
    
    let set = new Set();
    set.add(10);
    set.add(20);
    set.add(10); // Duplicate, shouldn't increase size
    
    expect_eq(set.size, 2);
    expect_eq(set.has(10), true);
    expect_eq(set.has(30), false);
    
    set.delete(10);
    expect_eq(set.size, 1);
    expect_eq(set.has(10), false);
    
    set.clear();
    expect_eq(set.size, 0);

    @print("🎉 Map and Set verified successfully!");
}
