import { test, expect_eq } from "cap:test";
import { print } from "cap:io";

async fn main(): void {
    let re = new RegExp("^hello [a-z]+$", "");
    
    let is_match = re.test("hello world");
    expect_eq(is_match, true);
    
    let is_fail = re.test("hello 123");
    expect_eq(is_fail, false);
    
    let s = "hello world";
    let m = s.match(re);
    expect_eq(m, "hello world");
    
    @print("🎉 ES6 RegExp implementation passed!");
}
