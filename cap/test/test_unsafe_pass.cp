import { print } from "cap:io";

struct User {
    age: i32
}

error AllocError {
    OutOfMemory
}

// 1. Unsafe function accepting raw pointer
unsafe fn setAge(p: *User!, val: i32) {
    p.age = val;
}

// 2. Unsafe function returning raw pointer
unsafe fn getPtr(): *User! {
    let u = User{age: 10};
    return &u as *User!;
}

// 3. Fallible function returning raw pointer (no confusion with ! and AllocError!)
unsafe fn getFalliblePtr(fail: bool): *User! ! AllocError {
    if (fail) {
        return Err(AllocError.OutOfMemory);
    }
    let u = User{age: 88};
    return &u as *User!;
}

// 4. Double raw pointer function
unsafe fn modifyViaDoublePtr(pp: **User!, new_val: i32) {
    pp.*.age = new_val;
}

// 5. Scalar raw pointer function
unsafe fn setScalar(p: *i32!, val: i32) {
    p.* = val;
}

fn main() {
    let mut user = User{age: 20};

    unsafe {
        // 1. Explicit cast to 1-level raw pointer
        let p: *User! = &user as *User!;
        setAge(p, 30);
        @print("User age after setAge: {d}", user.age);

        // 2. Double raw pointer (**User!)
        let mut p_copy: *User! = p;
        let pp: **User! = &p_copy as **User!;
        modifyViaDoublePtr(pp, 45);
        @print("User age after modifyViaDoublePtr: {d}", user.age);

        // 3. Scalar raw pointer (*i32!)
        let mut num: i32 = 100;
        let pnum: *i32! = &num as *i32!;
        setScalar(pnum, 200);
        @print("Scalar num after setScalar: {d}", num);

        // 4. Fallible raw pointer return
        let res = getFalliblePtr(false);
        let out = switch res {
            raw_p => "ok",
            Err(AllocError.OutOfMemory) => "oom",
            Err(*) => "err"
        };
        @print("Fallible ptr: {s}", out);
    }
}
