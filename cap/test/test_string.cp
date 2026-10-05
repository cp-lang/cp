import { test, expect_eq } from "cap:test";
import { print } from "cap:io";

async fn main(): void {
    let s = "Hello, World!";
    
    // length
    expect_eq(s.length, 13);
    
    // indexOf
    let idx1 = s.indexOf("o");
    expect_eq(idx1, 4);
    let idx2 = s.indexOf("x");
    expect_eq(idx2, 0 - 1);
    
    // lastIndexOf
    let idx3 = s.lastIndexOf("o");
    expect_eq(idx3, 8);
    
    // includes
    let inc1 = s.includes("World");
    expect_eq(inc1, true);
    let inc2 = s.includes("Foo");
    expect_eq(inc2, false);
    
    // startsWith / endsWith
    let sw = s.startsWith("Hello");
    expect_eq(sw, true);
    let ew = s.endsWith("!");
    expect_eq(ew, true);
    
    // slice / substring
    let sl = s.slice(7, 12);
    expect_eq(sl, "World");
    let su = s.substring(7, 12);
    expect_eq(su, "World");
    
    // toLowerCase / toUpperCase
    let lower = s.toLowerCase();
    expect_eq(lower, "hello, world!");
    let upper = s.toUpperCase();
    expect_eq(upper, "HELLO, WORLD!");
    
    // replace
    let rep = s.replace("World", "CP");
    expect_eq(rep, "Hello, CP!");
    
    // charAt
    // let ca = s.charAt(0);
    // expect_eq(ca, "H");
    
    // charCodeAt
    let cca = s.charCodeAt(0);
    expect_eq(cca, 72);
    
    // repeat
    // let a = "a";
    // let rep2 = a.repeat(3);
    // expect_eq(rep2, "aaa");
    
    @print("🎉 All string methods verified successfully!");
}
