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

    pub fn release(this): void {
        this.is_released = true;
    }
}

fn consume(own res: Resource): i32 {
    return res.id * 2;
}

pub fn main(): void {
    let r1 = Resource.create(42);
    // 使用 own 关键字进行独占所有权转移
    let val = consume(own r1);
    
    let a = Resource.create(100);
    // 表达式一元 own 转移所有权到新变量
    let b = own a;
    
    if (val == 84 && b.id == 100) {
        print("own keyword test passed: val=84, id=100\n");
    }
}
