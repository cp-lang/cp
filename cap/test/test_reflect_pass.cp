// cap/test/test_reflect_pass.cp
// Comprehensive Phase 2 Test for Dual-State Reflection

import { expect_eq } from "cap:test";
import { print } from "cap:io";

@target(class)
meta Controller(prefix: string);

@target(method)
meta Get(path: string = "");

@target(field)
meta Column(name: string);

meta JsonIgnore;

@target(class) @once
meta Singleton;

// 1. Definition of marked class with annotations
@reflect
@Singleton
@Controller("/api/users")
@AdhocTag("v1.0")
class UserController {
    @JsonIgnore
    @Column("pwd_hash")
    secret: string;

    // Multiple annotations on the same method (automatic array aggregation)
    @Get("/list")
    @Get("/all")
    fn list(): string {
        return "user_list";
    }

    // Single annotation
    @Get("/index")
    fn index(): string {
        return "index";
    }
}

// 2. Ref class definition
@reflect
ref class RefSession {
    sid: i32;
    active: bool;
}

fn main() {
    @print("--- Starting Phase 2 Reflection Tests ---");

    // Test 1: reflect.type<UserController>()
    let info = reflect.type<UserController>();
    expect_eq(info.name, "UserController");
    expect_eq(info.isRef, false);

    // Test 2: Class metadata read via native Dict operations
    expect_eq(info.meta.has("Singleton"), true);
    expect_eq(info.meta.has("NonExistent"), false);
    expect_eq(info.meta["Controller"], "/api/users");
    expect_eq(info.meta["AdhocTag"], "v1.0");

    // Test 3: Fields reflection
    expect_eq(info.fields.len, 1);
    let f0 = info.fields[0];
    expect_eq(f0.name, "secret");
    expect_eq(f0.typeName, "string");
    expect_eq(f0.meta.has("JsonIgnore"), true);
    expect_eq(f0.meta["Column"], "pwd_hash");

    // Test 4: Methods reflection & multiple annotation array collection
    expect_eq(info.methods.len, 2);
    let m0 = info.methods[0];
    expect_eq(m0.name, "list");
    expect_eq(m0.returnTypeName, "string");
    expect_eq(m0.isAsync, false);
    expect_eq(m0.isStatic, false);
    expect_eq(m0.meta.has("Get"), true);
    // Multiple @Get annotations aggregated into array: ["/list", "/all"]
    expect_eq(m0.meta["Get"][0], "/list");
    expect_eq(m0.meta["Get"][1], "/all");

    let m1 = info.methods[1];
    expect_eq(m1.name, "index");
    expect_eq(m1.returnTypeName, "string");
    expect_eq(m1.meta["Get"], "/index");

    // Test 5: Helper getField and getMethod
    let secretField = info.getField("secret");
    if (secretField != null) {
        expect_eq(secretField.name, "secret");
    }

    let listMethod = info.getMethod("list");
    if (listMethod != null) {
        expect_eq(listMethod.name, "list");
    }

    // Test 6: reflect.of(val)
    let ctrl = new UserController();
    let infoFromInstance = reflect.of(ctrl);
    expect_eq(infoFromInstance.name, "UserController");
    expect_eq(infoFromInstance.fields.len, 1);
    expect_eq(infoFromInstance.meta["Controller"], "/api/users");

    // Test 7: Ref struct reflection
    let refInfo = reflect.type<RefSession>();
    expect_eq(refInfo.name, "RefSession");
    expect_eq(refInfo.isRef, true);
    expect_eq(refInfo.fields.len, 2);
    expect_eq(refInfo.fields[0].name, "sid");
    expect_eq(refInfo.fields[0].typeName, "i32");
    expect_eq(refInfo.fields[1].name, "active");
    expect_eq(refInfo.fields[1].typeName, "bool");

    @print("🎉 Dual-State Reflection Phase 2 verified successfully!");
}
