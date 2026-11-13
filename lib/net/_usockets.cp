// lib/net/_usockets.cp
// Low-level uSockets C-FFI bindings, opaque handles, and constants
// Absorbed from Bun src/uws_sys/ and src/uws/uws.cp

// ============================================================================
// 1. uSockets Core Constants & Listen Options (from Bun uws.cp)
// ============================================================================
export const LIBUS_TIMEOUT_GRANULARITY: i32 = 4;
export const LIBUS_RECV_BUFFER_PADDING: i32 = 32;
export const LIBUS_EXT_ALIGNMENT: i32 = 16;

export const LIBUS_LISTEN_DEFAULT: i32 = 0;
export const LIBUS_LISTEN_EXCLUSIVE_PORT: i32 = 1;
export const LIBUS_SOCKET_ALLOW_HALF_OPEN: i32 = 2;
export const LIBUS_LISTEN_REUSE_PORT: i32 = 4;
export const LIBUS_SOCKET_IPV6_ONLY: i32 = 8;
export const LIBUS_LISTEN_REUSE_ADDR: i32 = 16;
export const LIBUS_LISTEN_DISALLOW_REUSE_PORT_FAILURE: i32 = 32;

// ============================================================================
// 2. Socket Close Codes, Kinds & Send Status Enums
// ============================================================================
export enum SocketCloseCode: i32 {
    normal = 0,
    failure = 1,
    fast_shutdown = 2,
}

export enum SocketKind: u8 {
    closed = 0,
    established_tcp = 1,
    established_tls = 2,
    connecting_tcp = 3,
    connecting_tls = 4,
    listen_tcp = 5,
    listen_tls = 6,
    udp = 7,
}

export enum SendStatus: u8 {
    backpressure = 0,
    success = 1,
    dropped = 2,
}

// ============================================================================
// 3. Opaque C Handles & C Structs (from Bun uws_sys/)
// ============================================================================
@opaque export struct us_loop_t {}
@opaque export struct us_socket_t {}
@opaque export struct us_socket_context_t {}
@opaque export struct us_listen_socket_t {}
@opaque export struct us_connecting_socket_t {}
@opaque export struct us_udp_socket_t {}
@opaque export struct us_udp_packet_buffer_t {}
@opaque export struct us_timer_t {}

@c export struct UsVerifyError {
    pub error_no: i32 = 0;
    pub code: [*:0]const u8?! = null;
    pub reason: [*:0]const u8?! = null;
}

// ============================================================================
// 4. uSockets C-FFI Declarations (@c block)
// ============================================================================
@c {
    // Loop lifecycle
    pub fn us_create_loop(
        hint: *void?!,
        wakeup_cb: *const fn (*us_loop_t!) void,
        pre_cb: *const fn (*us_loop_t!) void,
        post_cb: *const fn (*us_loop_t!) void,
        ext_size: u32
    ): *us_loop_t?!;
    pub fn us_loop_free(loop: *us_loop_t!): void;
    pub fn us_loop_run(loop: *us_loop_t!): void;
    pub fn us_loop_pump(loop: *us_loop_t!): void;
    pub fn us_wakeup_loop(loop: *us_loop_t!): void;
    pub fn us_loop_ext(loop: *us_loop_t!): *void!;

    // TCP / TLS Socket operations
    pub fn us_socket_context(is_ssl: i32, s: *us_socket_t!): *us_socket_context_t?!;
    pub fn us_socket_ext(is_ssl: i32, s: *us_socket_t!): *void!;
    pub fn us_socket_write(is_ssl: i32, s: *us_socket_t!, data: [*]const u8?!, length: i32): i32;
    pub fn us_socket_write2(
        is_ssl: i32,
        s: *us_socket_t!,
        header: [*]const u8?!,
        header_length: i32,
        payload: [*]const u8?!,
        payload_length: i32
    ): i32;
    pub fn us_socket_raw_write(is_ssl: i32, s: *us_socket_t!, data: [*]const u8?!, length: i32): i32;
    pub fn us_socket_flush(is_ssl: i32, s: *us_socket_t!): void;
    pub fn us_socket_timeout(is_ssl: i32, s: *us_socket_t!, seconds: u32): void;
    pub fn us_socket_long_timeout(is_ssl: i32, s: *us_socket_t!, minutes: u32): void;
    pub fn us_socket_pause(is_ssl: i32, s: *us_socket_t!): void;
    pub fn us_socket_resume(is_ssl: i32, s: *us_socket_t!): void;
    pub fn us_socket_nodelay(is_ssl: i32, s: *us_socket_t!, enabled: i32): void;
    pub fn us_socket_keepalive(is_ssl: i32, s: *us_socket_t!, enabled: i32, delay: u32): void;
    pub fn us_socket_close(is_ssl: i32, s: *us_socket_t!, code: i32, reason: *void?!): *us_socket_t?!;
    pub fn us_socket_close_connecting(is_ssl: i32, s: *us_socket_t!): *us_socket_t?!;
    pub fn us_socket_shutdown(is_ssl: i32, s: *us_socket_t!): void;
    pub fn us_socket_shutdown_read(is_ssl: i32, s: *us_socket_t!): void;
    pub fn us_socket_local_port(is_ssl: i32, s: *us_socket_t!): i32;
    pub fn us_socket_remote_port(is_ssl: i32, s: *us_socket_t!): i32;
    pub fn us_socket_remote_address(is_ssl: i32, s: *us_socket_t!, buf: [*]u8!, length: *i32!): void;
    pub fn us_socket_local_address(is_ssl: i32, s: *us_socket_t!, buf: [*]u8!, length: *i32!): void;
    pub fn us_socket_get_fd(is_ssl: i32, s: *us_socket_t!): i32;
    pub fn us_socket_is_closed(is_ssl: i32, s: *us_socket_t!): i32;
    pub fn us_socket_is_shut_down(is_ssl: i32, s: *us_socket_t!): i32;
    pub fn us_socket_is_tls(is_ssl: i32, s: *us_socket_t!): bool;
    pub fn us_socket_is_established(is_ssl: i32, s: *us_socket_t!): i32;
    pub fn us_socket_get_error(is_ssl: i32, s: *us_socket_t!): i32;

    // Listen & Connecting sockets
    pub fn us_listen_socket_close(is_ssl: i32, ls: *us_listen_socket_t!): void;
    pub fn us_listen_socket_get_fd(is_ssl: i32, ls: *us_listen_socket_t!): i32;
    pub fn us_socket_local_port_listen(is_ssl: i32, ls: *us_listen_socket_t!): i32;
    pub fn us_connecting_socket_close(is_ssl: i32, c: *us_connecting_socket_t!): void;

    // UDP sockets
    pub fn us_udp_socket_close(s: *us_udp_socket_t!): void;
    pub fn us_udp_socket_bound_port(s: *us_udp_socket_t!): i32;
    pub fn us_udp_socket_user(s: *us_udp_socket_t!): *void?!;
    pub fn us_udp_socket_set_broadcast(s: *us_udp_socket_t!, enabled: i32): i32;
    pub fn us_udp_socket_set_unicast_ttl(s: *us_udp_socket_t!, ttl: i32): i32;
    pub fn us_udp_socket_set_multicast_ttl(s: *us_udp_socket_t!, ttl: i32): i32;
    pub fn us_udp_socket_set_multicast_loopback(s: *us_udp_socket_t!, enabled: i32): i32;
}
