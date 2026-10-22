// cap/test/test_dec_pass.cp
// Comprehensive Phase 3 Test for Decorators & AOP Synthesis

import { expect_eq } from "cap:test";
import { print } from "cap:io";

// 1. Unparameterized Decorators (AOP Around: result doubling and offset)
dec doubleResult(next) {
    let res = next();
    return res * 2;
}

dec addTen(next) {
    let res = next();
    return res + 10;
}

// 2. Parametric Decorator (Direct Style)
dec tag(next, prefix: string) {
    let res = next();
    return "[" + prefix + "] " + res;
}

// 3. Higher-Order Parametric Decorator (TS / Python closure factory style)
dec prefixFactory(prefix: string) {
    return fn(next) {
        let res = next();
        return prefix + ":" + res;
    };
}

// 4. Onion Model: Outer and Inner wrappers
dec outerWrap(next) {
    let res = next();
    return "OUTER(" + res + ")";
}

dec innerWrap(next) {
    let res = next();
    return "INNER(" + res + ")";
}

// 5. Coexistence with Meta & Reflection
@target(method)
meta Route(path: string);

@reflect
class Calculator {
    base: i32;

    // Decorated class method: result doubling
    @doubleResult
    fn add(a: i32, b: i32): i32 {
        return this.base + a + b;
    }

    // Direct parametric decorator
    @tag("SERVICE")
    fn formatMsg(msg: string): string {
        return "msg=" + msg;
    }

    // Stacked decorators (Onion Model: outerWrap -> innerWrap -> target)
    @outerWrap
    @innerWrap
    @Route("/compute")
    fn computeName(id: string): string {
        return "item-" + id;
    }
}

// 6. Free/Top-level function decoration
@prefixFactory("GLOBAL")
fn greet(user: string): string {
    return "Hello " + user;
}

// Stacked on top-level function: (x * 2) + 10
@addTen
@doubleResult
fn calcNumber(x: i32): i32 {
    return x;
}

fn main() {
    @print("--- Starting Phase 3 Decorators & AOP Tests ---");

    // Test 1: Class method with @doubleResult
    let calc = new Calculator();
    calc.base = 100;
    let sum = calc.add(10, 20); // (100 + 10 + 20) * 2 = 260
    @print("calc.add(10, 20) = {d}", sum);
    expect_eq(sum, 260);

    // Test 2: Class method with direct parametric decorator @tag("SERVICE")
    let tagged = calc.formatMsg("ok");
    @print("calc.formatMsg: {s}", tagged);
    expect_eq(tagged, "[SERVICE] msg=ok");

    // Test 3: Onion Model stacked decorators: outerWrap(innerWrap(target))
    let layered = calc.computeName("99");
    @print("calc.computeName: {s}", layered);
    expect_eq(layered, "OUTER(INNER(item-99))");

    // Test 4: Higher-order factory decorator on free function
    let g = greet("CP");
    @print("greet: {s}", g);
    expect_eq(g, "GLOBAL:Hello CP");

    // Test 5: Stacked decorators on free function: addTen(doubleResult(x))
    // x = 5 -> doubleResult(5) = 10 -> addTen(10) = 20
    let num = calcNumber(5);
    @print("calcNumber(5) = {d}", num);
    expect_eq(num, 20);

    // Test 6: Verify reflection coexistence: Calculator still reflectable with methods
    let info = reflect.type<Calculator>();
    expect_eq(info.name, "Calculator");
    let m = info.getMethod("computeName");
    expect_eq(m != null, true);

    @print("🎉 Phase 3 Decorators & AOP verified successfully!");
}
