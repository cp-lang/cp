import { print } from "cap:io";

// 省略参数名、省略 ': void'、使用极简 '*!?' 可选通用指针
pub type ares_host_callback = @c fn (*!?, i32, i32, *!?);

pub struct MyContext {
    pub received_status: i32,
    pub received_timeouts: i32,
}

fn my_user_callback(ctx: *MyContext, status: i32, timeouts: i32, hostent: *!?): void {
    ctx.received_status = status;
    ctx.received_timeouts = timeouts;
}

// 使用统一的 @comptime 元编程注解修饰编译期参数
pub fn hostCallbackWrapper(@comptime Type: type, @comptime callback: anytype): ares_host_callback {
    @c fn handle(ctx: *!?, status: i32, timeouts: i32, hostent: *!?): void {
        unsafe {
            // 使用 CP 规范的 'as' 显式强转，编译器自动安全注入底层 @ptrCast(@alignCast(...))
            let this_ptr = ctx.? as *Type;
            callback(this_ptr, status, timeouts, hostent);
        }
    }
    // &handle 获取静态 C-ABI 函数指针
    return &handle;
}

fn main(): void {
    // 遵循 CP 哲学：'let' 变量默认可变，废除冗余的 'mut'
    let my_ctx = MyContext {
        received_status: 0,
        received_timeouts: 0,
    };

    // 自动类型推断，无需冗余类型注解
    let cb = hostCallbackWrapper(MyContext, my_user_callback);

    unsafe {
        // 验证 1: 使用对称取裸指针语法 '*my_ctx!?'（可变 C 裸指针）
        let raw_ctx1 = *my_ctx!?;
        cb(raw_ctx1, 200, 15, null);

        // 验证 2: 使用只读对称取裸指针语法 '&my_ctx!?'
        let raw_ctx2 = &my_ctx!?;
        cb(raw_ctx2, 201, 16, null);

        // 验证 3: 使用显式 'as *!?' 转换
        let raw_ctx3 = *my_ctx as *!?;
        cb(raw_ctx3, 202, 17, null);

        // 验证 4: 使用 '&my_ctx as *!?' 转换
        let raw_ctx4 = &my_ctx as *!?;
        cb(raw_ctx4, 200, 15, null);
    }

    print(`Status: ${my_ctx.received_status}\n`);
    print(`Timeouts: ${my_ctx.received_timeouts}\n`);

    if (my_ctx.received_status == 200 && my_ctx.received_timeouts == 15) {
        print("C-Trampoline test passed!\n");
    } else {
        print("C-Trampoline test failed!\n");
    }
}
