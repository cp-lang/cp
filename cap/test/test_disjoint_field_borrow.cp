import { print } from "cap:io";

class Header {
    content_type: string;
    content_len: i32;
}

class HTTPContext {
    header: Header;
    status_code: i32;
    body_read: i32;
    active_timer: i32;

    fn update_fields(*this): void {
        // Disjoint borrows on `this` fields
        let status_ref = &mut this.status_code;
        let timer_ref = &mut this.active_timer;
        status_ref.* = 200;
        timer_ref.* = 999;
    }
}

class InternalLoopData {
    q_len: i32;
    active_timer: i32;
    max_batch: i32;
}

fn process_internal_loop(*data: InternalLoopData): void {
    // 1. Simultaneous mutable borrows to disjoint fields
    let q = &mut data.q_len;
    let t = &mut data.active_timer;
    let b = &mut data.max_batch;

    q.* = 42;
    t.* = 100;
    b.* = 1024;
}

fn process_nested(*ctx: HTTPContext): void {
    // 2. Disjoint borrows on nested struct fields
    let h_len = &mut ctx.header.content_len;
    let b_read = &mut ctx.body_read;

    h_len.* = 4096;
    b_read.* = 128;
}

pub fn main(): void {
    let loop_data = new InternalLoopData(0, 0, 0);
    process_internal_loop(&loop_data);
    if (loop_data.q_len != 42) {
        @panic("q_len mismatch");
    }
    if (loop_data.active_timer != 100) {
        @panic("active_timer mismatch");
    }
    if (loop_data.max_batch != 1024) {
        @panic("max_batch mismatch");
    }

    let hdr = new Header("application/json", 0);
    let ctx = new HTTPContext(hdr, 0, 0, 0);
    ctx.update_fields();
    if (ctx.status_code != 200 || ctx.active_timer != 999) {
        @panic("ctx field update failed");
    }

    process_nested(&ctx);
    if (ctx.header.content_len != 4096 || ctx.body_read != 128) {
        @panic("nested field update failed");
    }

    print("Disjoint field borrowing test passed! ✓");
}
