import { print } from "cap:io";

// ============================================================================
// 1. Struct Error (Value Type, Stack Alloc, Zero Heap Overhead)
// ============================================================================
struct MyStructError {
    code: i32;
    msg: string;
}

fn fail_struct(): i32 ! MyStructError {
    return err(MyStructError { code: 404, msg: "struct not found" });
}

fn fail_struct_to_unified_err(): i32 ! Err {
    return err(MyStructError { code: 403, msg: "struct forbidden" });
}

// ============================================================================
// 2. Class Error (Reference Type, Heap Allocated Pointer)
// ============================================================================
class MyClassError {
    pub code: i32;
    pub msg: string;
    pub fn init(c: i32, m: string) {
        this.code = c;
        this.msg = m;
    }
}

fn fail_class(): i32 ! MyClassError {
    let ce = new MyClassError(500, "class server crash");
    return err(ce);
}

fn fail_class_to_unified_err(): i32 ! Err {
    let ce = new MyClassError(502, "class bad gateway");
    return err(ce);
}

// ============================================================================
// 3. Enum with Payloads as Error Type
// ============================================================================
enum TaskErr {
    Idle,
    Text(string),
    Move([i32, i32]),
    Compute { batch: i32, priority: i32 }
}

fn fail_payload_text(): i32 ! TaskErr {
    return err(TaskErr.Text("text_payload_failed"));
}

fn fail_payload_move(): i32 ! TaskErr {
    return err(TaskErr.Move([10, 20]));
}

fn fail_payload_compute(): i32 ! TaskErr {
    return err(TaskErr.Compute { batch: 64, priority: 1 });
}

fn fail_payload_dot_inference(): i32 ! TaskErr {
    return err(.Compute { batch: 128, priority: 2 });
}

fn main(): void {
    // 1. Struct Error direct destructuring match
    let r1 = fail_struct();
    switch r1 {
        err(MyStructError { code, msg }) => {
            print(`1. Struct matched: ${code} ${msg}`);
        },
        val => print(val),
    }

    // 2. Struct Error auto-converted to unified Err
    let r2 = fail_struct_to_unified_err();
    switch r2 {
        err(403, msg) => {
            print(`2. Struct auto-converted to Err: ${msg}`);
        },
        err(e) => print(`fallback err: ${e.code}`),
        val => print(val),
    }

    // 3. Class Error direct matching
    let r3 = fail_class();
    switch r3 {
        err(e) => {
            print(`3. Class error matched: ${e.code} ${e.msg}`);
        },
        val => print(val),
    }

    // 4. Class Error downcast from Err (type-match syntax `e: MyClassError`)
    let r4 = fail_class_to_unified_err();
    switch r4 {
        err(e: MyClassError) => {
            print(`4. Class downcasted from Err (: syntax): ${e.code} ${e.msg}`);
        },
        err(code, msg) => {
            print(`4. Class converted to Err: ${code} ${msg}`);
        },
        val => print(val),
    }

    // 4b. Class Error downcast from Err (binding syntax `er := MyClassError`)
    let r4b = fail_class_to_unified_err();
    switch r4b {
        err(er := MyClassError) => {
            print(`4b. Class downcasted from Err (:= syntax): ${er.code} ${er.msg}`);
        },
        * => print("fallback 4b"),
    }

    // 5. Enum Text payload matching
    let r5 = fail_payload_text();
    switch r5 {
        err(.Text text) => {
            print(`5. Enum Text payload matched: ${text}`);
        },
        * => print("fallback 5"),
    }

    // 6. Enum Move tuple matching
    let r6 = fail_payload_move();
    switch r6 {
        err(.Move [x, y]) => {
            print(`6. Enum Move tuple matched: ${x} ${y}`);
        },
        * => print("fallback 6"),
    }

    // 7. Enum Compute exact fields matching
    let r7 = fail_payload_compute();
    switch r7 {
        err(.Compute { batch, priority }) => {
            print(`7. Enum Compute exact matched: ${batch} ${priority}`);
        },
        * => print("fallback 7"),
    }

    // 8. Enum Compute rest fields matching (...fields)
    let r8 = fail_payload_compute();
    switch r8 {
        err(.Compute { ...fields }) => {
            print(`8. Enum Compute rest matched: ${fields.batch} ${fields.priority}`);
        },
        * => print("fallback 8"),
    }

    // 9. Enum Compute with dot inference return
    let r9 = fail_payload_dot_inference();
    switch r9 {
        err(.Compute { batch, priority }) => {
            print(`9. Enum dot inference matched: ${batch} ${priority}`);
        },
        * => print("fallback 9"),
    }

    print("ALL_STRUCT_AND_ENUM_PAYLOAD_TESTS_PASSED");
}
