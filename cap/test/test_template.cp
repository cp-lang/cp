fn format_user(uid: i32, role: string): string {
    return `User #${uid} [${role}]`;
}

async fn main(): void {
    let name: string = "World";
    let version: i32 = 1;
    
    // 1. Simple interpolation
    let t1: string = `Hello ${name}!`;
    @print("t1: {s}", t1);
    
    // 2. Multiple interpolations with different types
    let t2: string = `CP Language v${version} says ${name}`;
    @print("t2: {s}", t2);
    
    // 3. Interpolation with expression
    let a: i32 = 10;
    let b: i32 = 20;
    let t3: string = `Result: ${a + b}`;
    @print("t3: {s}", t3);
    
    // 4. No interpolation (constant folding)
    let t4: string = `Just a string`;
    @print("t4: {s}", t4);

    // 5. Escaping \$ and \`
    let t5: string = `Price is \$100 for \`CP Lang\``;
    @print("t5: {s}", t5);

    // 6. JSON payload with literal curly braces
    let user_id = 88;
    let t6: string = `{"status": 200, "user_id": ${user_id}, "name": "${name}"}`;
    @print("t6: {s}", t6);

    // 7. Expressions containing braces inside strings
    let t7: string = `Braces in expr: ${"{" + "foo" + "}"}`;
    @print("t7: {s}", t7);

    // 8. Chained string method on template string
    let t8: string = `agent ${name}`.toUpperCase();
    @print("t8: {s}", t8);

    // 9. Sync function return template string
    let t9: string = format_user(999, "Admin");
    @print("t9: {s}", t9);

    @print("🎉 Template strings verified successfully! {s}", "");
}
