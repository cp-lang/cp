import { test, expect_eq } from "cap:test";
import { JSON, JsonValue, JsonTag, json } from "cap:json";
const std = @import("std");

fn test_primitive_json(): !void {
    const allocator = std.heap.page_allocator;

    const v_null = try JSON.parse(allocator, "null");
    expect_eq(v_null.isNull(), true);

    const v_bool1 = try JSON.parse(allocator, "true");
    expect_eq(v_bool1.asBool(), true);

    const v_bool2 = try JSON.parse(allocator, "false");
    expect_eq(v_bool2.asBool(), false);

    const v_num = try JSON.parse(allocator, "123.45");
    expect_eq(v_num.tag == JsonTag.Number, true);
    if (v_num.asNumber()) |n| {
        expect_eq(n > 123.0 and n < 124.0, true);
    }
}

fn test_zero_copy_string_borrow(): !void {
    const allocator = std.heap.page_allocator;
    const raw_input = "  \"hello_zero_copy_world\"  ";
    const v_str = try JSON.parse(allocator, raw_input);

    expect_eq(v_str.tag == JsonTag.String, true);
    if (v_str.asString()) |s| {
        expect_eq(s.len, 21);
        // Verify that the slice points directly into raw_input (zero-copy borrow!)
        const input_ptr = @intFromPtr(raw_input.ptr);
        const slice_ptr = @intFromPtr(s.ptr);
        expect_eq(slice_ptr >= input_ptr, true);
        expect_eq(slice_ptr < input_ptr + raw_input.len, true);
    }
}

fn test_complex_object_and_array(): !void {
    const allocator = std.heap.page_allocator;
    const json_str = "{\"name\": \"agent\", \"active\": true, \"scores\": [10, 20, 30]}";
    const root = try JSON.parse(allocator, json_str);

    expect_eq(root.tag == JsonTag.Object, true);
    expect_eq(root.len(), 3);

    const name_val = root.get("name");
    expect_eq(name_val != null, true);
    if (name_val) |nv| {
        expect_eq(nv.tag == JsonTag.String, true);
        if (nv.asString()) |s| {
            expect_eq(std.mem.eql(u8, s, "agent"), true);
        }
    }

    const active_val = root.get("active");
    expect_eq(active_val != null, true);
    if (active_val) |av| {
        expect_eq(av.asBool(), true);
    }

    const scores_val = root.get("scores");
    expect_eq(scores_val != null, true);
    if (scores_val) |sv| {
        expect_eq(sv.tag == JsonTag.Array, true);
        expect_eq(sv.len(), 3);
        const item0 = sv.getIndex(0);
        expect_eq(item0 != null, true);
    }
}

fn main(): void {
    test("Primitive JSON tokens null, bool, number", () => {
        try test_primitive_json();
    });

    test("Zero-copy string slice borrows directly from source buffer", () => {
        try test_zero_copy_string_borrow();
    });

    test("Complex nested JSON object and array parsing with lookup", () => {
        try test_complex_object_and_array();
    });
}
