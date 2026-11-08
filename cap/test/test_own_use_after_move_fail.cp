import { print } from "cap:io";

struct Resource {
    pub id: i32,
    pub is_released: bool,

    pub static fn create(id: i32): Resource {
        return Resource {
            id: id,
            is_released: false,
        };
    }
}

fn consume(own res: Resource): i32 {
    return res.id;
}

pub fn main(): void {
    let r1 = Resource.create(42);
    let val = consume(own r1);
    // ❌ 错误：r1 已被 own 移交，再次读取必须被编译器拦截！
    print("{}", r1.id);
}
