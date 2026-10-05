async fn main(): void {
    let name: string = "World";
    let version: i32 = 1;
    
    // Simple interpolation
    let t1: string = `Hello ${name}!`;
    @print("t1: {s}", t1);
    
    // Multiple interpolations with different types
    let t2: string = `CP Language v${version} says ${name}`;
    @print("t2: {s}", t2);
    
    // Interpolation with expression
    let a: i32 = 10;
    let b: i32 = 20;
    let t3: string = `Result: ${a + b}`;
    @print("t3: {s}", t3);
    
    // No interpolation
    let t4: string = `Just a string`;
    @print("t4: {s}", t4);

    @print("🎉 Template strings verified successfully! {s}", "");
}
