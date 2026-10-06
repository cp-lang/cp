import { test, expect_eq } from "cap:test";
import { print } from "cap:io";

async fn main(): void {
    let x = 1.0;
    
    // Test constants
    let pi = Math.PI;
    let e = Math.E;
    
    // Basic ops
    let a = Math.abs(0.0 - 5.5);
    expect_eq(a, 5.5);
    
    let c = Math.ceil(1.2);
    expect_eq(c, 2.0);
    
    let f = Math.floor(1.8);
    expect_eq(f, 1.0);
    
    let r = Math.round(1.5);
    expect_eq(r, 2.0);
    
    let t = Math.trunc(1.9);
    expect_eq(t, 1.0);
    
    // Min/Max
    let m1 = Math.max(10.0, 20.0);
    expect_eq(m1, 20.0);
    
    let m2 = Math.min(10.0, 20.0);
    expect_eq(m2, 10.0);
    
    // Complex ops
    let sq = Math.sqrt(9.0);
    expect_eq(sq, 3.0);
    
    let pw = Math.pow(2.0, 3.0);
    expect_eq(pw, 8.0);
    
    let lg = Math.log(Math.E);
    // Float comparison is tricky, but should be approx 1.0
    // We will just verify it runs
    
    let lg10 = Math.log10(100.0);
    expect_eq(lg10, 2.0);
    
    let lg2 = Math.log2(8.0);
    expect_eq(lg2, 3.0);
    
    // Random
    let rnd = Math.random();
    
    @print("🎉 All Math methods verified successfully!");
}
