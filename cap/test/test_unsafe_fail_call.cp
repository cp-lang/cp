struct User {
    age: i32
}

unsafe fn setAge(p: *User!, val: i32) {
    p.age = val;
}

fn main() {
    let mut u = User{age: 20};
    unsafe {
        let p: *User! = &u as *User!;
    }
    // Calling unsafe function outside unsafe block should fail sema
    setAge(null, 30);
}
