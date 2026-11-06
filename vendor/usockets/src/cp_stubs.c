/*
 * CP Language Native Stubs for Bun's fork of uSockets
 * Allows uSockets to be compiled and linked standalone in CP without full Bun runtime.
 */

#include <stdlib.h>
#include <stdint.h>
#include <errno.h>

// ── Epoll / Kernel Detection ──────────────────────────────────────────────────
int Bun__isEpollPwait2SupportedOnLinuxKernel(void) { return 0; }
int sys_epoll_pwait2(int epfd, void *events, int maxevents, const void *timeout, const void *sigmask, size_t sigsetsize) {
    errno = ENOSYS;
    return -1;
}

// ── Event Loop & Concurrency Hooks ───────────────────────────────────────────
void Bun__internal_dispatch_ready_poll(void* loop, void* poll) {}
void Bun__JSC_onBeforeWait(void* loop) {}
void Bun__panic(const char* msg) { abort(); }
void Bun__internal_ensureDateHeaderTimerIsEnabled(void* loop) {}
void Bun__lock(void* mutex) {}
void Bun__unlock(void* mutex) {}

// ── Async DNS Stubs ──────────────────────────────────────────────────────────
void* Bun__addrinfo_get(void* req) { return NULL; }
void Bun__addrinfo_set(void* req, void* val) {}
int Bun__addrinfo_getRequestResult(void* req, void** res) { return -1; }
void Bun__addrinfo_freeRequest(void* req) {}
void Bun__addrinfo_cancel(void* req) {}

// ── Socket Event Dispatch Callbacks ───────────────────────────────────────────
void us_dispatch_connect_error(void* s, int code) {}
void us_dispatch_open(void* s, int is_client, char* ip, int ip_len) {}
void us_dispatch_connecting_error(void* s, int code) {}
void us_dispatch_close(void* s, int code, void* reason) {}
void us_dispatch_long_timeout(void* s) {}
void us_dispatch_timeout(void* s) {}
void us_dispatch_fd(void* s, int fd) {}
void us_dispatch_data(void* s, char* data, int len) {}
void us_dispatch_writable(void* s) {}
void us_dispatch_end(void* s) {}

// ── QUIC & UDP Stubs ─────────────────────────────────────────────────────────
void us_quic_loop_process(void* loop) {}
void us_udp_socket_close(void* s) {}

// ── SSL Stubs (for LIBUS_NO_SSL mode) ─────────────────────────────────────────
int us_internal_raw_root_certs(void** out) { *out = NULL; return 0; }
void us_internal_listen_socket_ssl_free(void* s) {}
void us_internal_ssl_ctx_up_ref(void* ctx) {}
void* us_internal_ssl_attach(void* s, void* ctx, int is_server) { return s; }
void us_internal_ssl_on_open(void* s) {}
int us_internal_ssl_verify_error(void* s) { return 0; }
int us_internal_ssl_is_handshake_finished(void* s) { return 1; }
int us_internal_ssl_handshake_callback_has_fired(void* s) { return 1; }
void us_internal_ssl_ctx_unref(void* ctx) {}
void us_internal_ssl_on_close(void* s) {}
void us_internal_ssl_detach(void* s) {}
void us_internal_ssl_close(void* s) {}
void* us_internal_ssl_get_native_handle(void* s) { return NULL; }
int us_internal_ssl_write(void* s, const char* data, int length, int msg_more) { return length; }
int us_internal_ssl_shutdown(void* s) { return 0; }
void us_internal_ssl_on_writable(void* s) {}
int us_internal_ssl_is_low_prio(void* s) { return 0; }
void us_internal_ssl_on_end(void* s) {}
void us_internal_ssl_on_data(void* s, char* data, int len) {}
int us_internal_ssl_is_shut_down(void* s) { return 0; }
