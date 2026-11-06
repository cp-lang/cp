// cap/test/test_usockets_picohttp.cp
// 100% 纯正 CP 语法端到端验证 @c / @c("alias") 与 unsafe 裸指针互操作体系
// 无任何内联 zig {} 代码，直接在纯 CP 中调用 uSockets 与 picohttpparser

// 1. C 内存布局结构体（支持 @c struct 或 @fix struct，亦兼容 @extern struct）与不透明类型声明
@c struct phr_header {
    name: cstr,
    name_len: usize,
    value: cstr,
    value_len: usize,
}

@fix struct phr_fix_sample {
    dummy: i32,
}

@opaque struct us_loop_t {}

// 2. 通过 @c("alias") 导入并重命名 C 函数（C 符号名为 phr_parse_request，在 CP 中重命名为 phr_parse_req）
@c("phr_parse_request")
fn phr_parse_req(
    buf: cstr,
    len: usize,
    method: *cstr!,
    method_len: *usize!,
    path: *cstr!,
    path_len: *usize!,
    minor_version: *i32!,
    headers: [*]phr_header!,
    num_headers: *usize!,
    last_len: usize,
): i32;

// 3. 通过 @c { ... } 批量导入 C 函数，使用 unsafe 裸指针与后缀可选指针 (*T!?)
@c {
    fn us_create_loop(
        hint: *any!?,
        wakeup_cb: *any!?,
        pre_cb: *any!?,
        post_cb: *any!?,
        ext_size: u32,
    ): *us_loop_t!?;

    fn us_loop_free(loop: *us_loop_t!?): void;
}

fn test_pico_http(): void {
    @print("Testing picohttpparser HTTP parsing in pure CP via @c(\"phr_parse_request\")...");
    let req: cstr = "POST /api/v1/agent/chat HTTP/1.1\r\nHost: 127.0.0.1:8080\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"message\":\"ping\"}";
    let req_len: usize = req.len;

    unsafe {
        let method: cstr = "";
        let method_len: usize = 0;
        let path: cstr = "";
        let path_len: usize = 0;
        let minor_version: i32 = 0;
        let header_buf: [16]phr_header = undefined;
        let num_headers: usize = 16;

        let consumed = phr_parse_req(
            req,
            req_len,
            *method!,
            *method_len!,
            *path!,
            *path_len!,
            *minor_version!,
            header_buf as [*]phr_header!,
            *num_headers!,
            0,
        );

        if (consumed > 0) {
            @print("  ✓ Parse success! Consumed: {d} bytes", consumed);
            @print("  ✓ Method: {s}", method[0..method_len]);
            @print("  ✓ Path: {s}", path[0..path_len]);
            @print("  ✓ Version: HTTP/1.{d}", minor_version);
            @print("  ✓ Headers parsed: {d}", num_headers);
            let h0 = header_buf[0];
            @print("  ✓ Header[0]: {s}: {s}", h0.name[0..h0.name_len], h0.value[0..h0.value_len]);
            let h1 = header_buf[1];
            @print("  ✓ Header[1]: {s}: {s}", h1.name[0..h1.name_len], h1.value[0..h1.value_len]);
            let h2 = header_buf[2];
            @print("  ✓ Header[2]: {s}: {s}", h2.name[0..h2.name_len], h2.value[0..h2.value_len]);
        } else {
            @panic("phr_parse_req failed");
        }
    }
}

fn test_usockets_event_loop(): void {
    @print("Testing uSockets Event Loop in pure CP via @c block...");
    let loop = us_create_loop(null, null, null, null, 0);
    if (loop != null) {
        @print("  ✓ Native uSockets event loop created successfully!");
        us_loop_free(loop);
        @print("  ✓ Native uSockets event loop freed successfully!");
    } else {
        @panic("us_create_loop returned null");
    }
}

export fn main(): void {
    test_pico_http();
    test_usockets_event_loop();
    @print("ALL PURE CP USOCKETS & PICOHTTPPARSER TESTS PASSED!");
}
