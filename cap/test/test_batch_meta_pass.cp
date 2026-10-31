// cap/test/test_batch_meta_pass.cp
// Comprehensive Test for Batch Annotations (@[...]) & Weak-typed Meta (@meta(...))

import { expect_eq } from "cap:test";

meta Author(name: string);
meta Timeout(ms: i32);
meta Singleton;
meta Column(name: string);
meta JsonIgnore;
meta Get(path: string);

dec traceDec(next) {
    @print("[Decorator] enter");
    let res = next();
    @print("[Decorator] exit");
    return res;
}

// 1. Class-level batch annotation bundling: @[...]
// Explicit @reflect combined with typed meta and weak-typed @meta(...) in single @[...] batch
@[reflect, Singleton, Author("Antigravity"), Timeout(3000), meta(author: "Antigravity-Agent", timeout: 5000, version: "2.0")]
class PaymentService {
    // 2. Field-level batch annotation
    @[JsonIgnore, Column("tx_hash")]
    txId: string;

    // 3. Method-level batch combining decorator & annotation
    @[traceDec, Get("/pay")]
    fn processPay(): string {
        @print("[Method] processPay executed");
        return "PAID_OK";
    }
}

// 4. Batch combining @reflect and ad-hoc @meta(...)
@[reflect, meta(author: "AdHocMaster", timeout: 1200)]
class AdHocService {
    @meta(role: "superadmin")
    fn deleteDb(): string {
        return "DB_DELETED";
    }
}

// 5. Pure compile-time metadata without @reflect (Zero runtime overhead, no TypeInfo generated)
@meta(author: "NoReflectAuthor", compileOnly: true)
class CompileOnlyService {
    val: i32;
    fn getVal(): i32 {
        return this.val;
    }
}

fn main() {
    @print("--- Starting Batch & Weak-typed Meta Tests ---");

    // Test 1: Decorator woven method execution
    let svc = new PaymentService();
    svc.txId = "tx_998877";
    let status = svc.processPay();
    expect_eq(status, "PAID_OK");

    // Test 2: Class reflection on PaymentService
    let info = reflect.type<PaymentService>();
    expect_eq(info.name, "PaymentService");
    expect_eq(info.meta.has("Singleton"), true);
    expect_eq(info.meta["Author"], "Antigravity");
    expect_eq(info.meta["Timeout"], 3000);

    // Weak-typed meta accessible flattened
    expect_eq(info.meta["author"], "Antigravity-Agent");
    expect_eq(info.meta["timeout"], 5000);
    expect_eq(info.meta["version"], "2.0");

    // Weak-typed meta accessible namespaced
    expect_eq(info.meta["meta"]["author"], "Antigravity-Agent");
    expect_eq(info.meta["meta"]["timeout"], 5000);
    expect_eq(info.meta["meta"]["version"], "2.0");

    // Test 3: Field annotations
    expect_eq(info.fields.len, 1);
    let f0 = info.fields[0];
    expect_eq(f0.name, "txId");
    expect_eq(f0.meta.has("JsonIgnore"), true);
    expect_eq(f0.meta["Column"], "tx_hash");

    // Test 4: Method annotations & decorator presence
    expect_eq(info.methods.len, 1);
    let m0 = info.methods[0];
    expect_eq(m0.name, "processPay");
    expect_eq(m0.meta["Get"], "/pay");

    // Test 5: AdHocService with @[reflect, meta(...)]
    let ad_info = reflect.type<AdHocService>();
    expect_eq(ad_info.name, "AdHocService");
    expect_eq(ad_info.meta["author"], "AdHocMaster");
    expect_eq(ad_info.meta["timeout"], 1200);

    expect_eq(ad_info.methods.len, 1);
    let sm0 = ad_info.methods[0];
    expect_eq(sm0.name, "deleteDb");
    expect_eq(sm0.meta["role"], "superadmin");

    // Test 6: Compile-time only class operates normally with zero reflection overhead
    let cos = new CompileOnlyService();
    cos.val = 42;
    expect_eq(cos.getVal(), 42);

    @print("🎉 Batch Annotation & Weak-typed Meta verified successfully!");
}
