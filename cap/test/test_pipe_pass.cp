// cap/test/test_pipe_pass.cp
// Comprehensive Test for Pipeline Operator (|>) and Placeholder (.)

import { expect_eq } from "cap:test";
import { print } from "cap:io";

// Helper functions for testing
fn double_num(x: i32): i32 {
    return x * 2;
}

fn add_num(a: i32, b: i32): i32 {
    return a + b;
}

fn sub_num(a: i32, b: i32): i32 {
    return a - b;
}

class TextWrapper {
    content: string;
    fn init(c: string) {
        _.content = c;
    }
    fn prefix(p: string): string {
        return p + ":" + _.content;
    }
    fn get(): string {
        return _.content;
    }
}

class Point {
    x: i32;
    y: i32;
    fn init(x: i32, y: i32) {
        _.x = x;
        _.y = y;
    }
}

// Side-effect counter to verify single evaluation (No Double Evaluation)
class Counter {
    count: i32;
    fn init() {
        _.count = 0;
    }
    fn get_seed(mut _): i32 {
        _.count = _.count + 1;
        return 7;
    }
}

fn combine_two(a: i32, b: i32): i32 {
    return a * 10 + b;
}

// Agent pipeline stages
fn sanitize_prompt(raw: string): string {
    return "sanitized_" + raw;
}

fn retrieve_rag(kb: string, query: string, top_k: i32): string {
    return "RAG(" + kb + ",k=" + top_k + ")[" + query + "]";
}

fn build_context(header: string, body: string): string {
    return "<" + header + ">" + body + "</" + header + ">";
}

fn simulate_agent_llm(prompt: string): string {
    return "AGENT_REPLY(" + prompt + ")";
}

fn main(): void {
    @print("=== [Test 1] Rule 1: Bare function call (x |> f) ===");
    let r1 = 21 |> double_num;
    @print("21 |> double_num = {d}", r1);
    expect_eq(r1, 42);

    @print("=== [Test 2] Rule 2: Call without placeholder (default first arg) ===");
    let r2 = 100 |> add_num(50);
    @print("100 |> add_num(50) = {d}", r2);
    expect_eq(r2, 150);

    @print("=== [Test 3] Rule 3: Standalone '.' in arbitrary parameter slot ===");
    // Passed as second argument
    let r3 = 5 |> sub_num(30, .);
    @print("5 |> sub_num(30, .) = {d}", r3);
    expect_eq(r3, 25);

    // Passed as first argument explicitly
    let r4 = 30 |> sub_num(., 5);
    @print("30 |> sub_num(., 5) = {d}", r4);
    expect_eq(r4, 25);

    @print("=== [Test 4] Rule 3: Method call (obj |> .method()) ===");
    let tw = new TextWrapper("agent_core");
    let r5 = tw |> .prefix("CP");
    @print("tw |> .prefix(\"CP\") = {s}", r5);
    expect_eq(r5, "CP:agent_core");

    @print("=== [Test 5] Rule 3: Field access & arithmetic (point |> .x * .y) ===");
    let pt = new Point(6, 7);
    let r6 = pt |> .x * .y;
    @print("pt |> .x * .y = {d}", r6);
    expect_eq(r6, 42);

    @print("=== [Test 6] Multiline Chained Pipeline ===");
    let chained_res = 10
        |> double_num     // 20
        |> add_num(10)    // 30
        |> sub_num(100, .) // 100 - 30 = 70
        |> double_num;    // 140
    @print("Chained result = {d}", chained_res);
    expect_eq(chained_res, 140);

    @print("=== [Test 7] No Double Evaluation Verification ===");
    let cnt = new Counter();
    let r7 = cnt.get_seed() |> combine_two(., .);
    @print("cnt.get_seed() |> combine_two(., .) = {d}, count = {d}", r7, cnt.count);
    expect_eq(r7, 77);
    expect_eq(cnt.count, 1); // Crucial: get_seed() only evaluated ONCE!

    @print("=== [Test 8] AI Agent Pipeline Workflow ===");
    let user_msg = "find_answer";
    let agent_output = user_msg
        |> sanitize_prompt
        |> retrieve_rag("agent_kb", ., 5)
        |> build_context("PROMPT", .)
        |> simulate_agent_llm;
    @print("Agent Output = {s}", agent_output);
    expect_eq(agent_output, "AGENT_REPLY(<PROMPT>RAG(agent_kb,k=5)[sanitized_find_answer]</PROMPT>)");

    @print("=== All Pipeline (|>) and Placeholder (.) tests passed successfully! ===");
}
