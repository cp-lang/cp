// cap/test/test_http_full.cp
// Comprehensive End-to-End Test Suite for Industrial HTTP Subsystem (cap:http)
// Validating Step 1, Step 2 (H2/H3), and Step 3 (Core Client Engine & Lifecycle)

import { test, expect_eq } from "cap:test";
import {
    Method,
    Headers,
    Encoding,
    FetchCacheMode,
    FetchRedirect,
    FetchRequestMode,
    URLPath,
    SignalStore,
    HeaderValueIterator,
    HTTPRequestBody,
    InternalState,
    FrameType,
    ErrorCode,
    FrameHeader,
    H2ClientSession,
    H2StreamState,
    AltSvc,
    H3ClientSession,
    ProxyTunnel,
    SendFile,
    HTTPContext,
    HTTPClient,
    HTTPClientResult,
    Decompressor
} from "cap:http";

const std = @import("std");

fn testStep1_TypesAndInfrastructure(): !void {
    // 1. Encoding
    const enc_gzip = Encoding.Gzip;
    expect_eq(enc_gzip.canUseLibDeflate(), true);
    expect_eq(enc_gzip.isCompressed(), true);

    const enc_identity = Encoding.Identity;
    expect_eq(enc_identity.canUseLibDeflate(), false);
    expect_eq(enc_identity.isCompressed(), false);

    // 2. Fetch Modes
    const cache_mode = FetchCacheMode.fromString("no-store");
    expect_eq(cache_mode != null, true);
    expect_eq(cache_mode.? == .NoStore, true);

    const redirect_mode = FetchRedirect.fromString("follow");
    expect_eq(redirect_mode != null, true);
    expect_eq(redirect_mode.? == .Follow, true);

    const req_mode = FetchRequestMode.fromString("cors");
    expect_eq(req_mode != null, true);
    expect_eq(req_mode.? == .Cors, true);

    // 3. URLPath Parsing
    const path_info = URLPath.parse("/api/v1/users/profile.json?tab=activity&view=compact");
    expect_eq(std.mem.eql(u8, path_info.path, "/api/v1/users/profile.json"), true);
    expect_eq(std.mem.eql(u8, path_info.extname, "json"), true);
    expect_eq(std.mem.eql(u8, path_info.first_segment, "api"), true);
    expect_eq(std.mem.eql(u8, path_info.query_string, "?tab=activity&view=compact"), true);
    expect_eq(path_info.is_source_map, false);

    const root_path = URLPath.parse("/");
    expect_eq(root_path.isRoot(), true);

    // 4. Signals
    var store = SignalStore{};
    const sigs = store.toSignals();
    expect_eq(sigs.isAborted(), false);
    sigs.abort();
    expect_eq(sigs.isAborted(), true);

    // 5. HeaderValueIterator
    var iter = HeaderValueIterator.init("gzip, deflate, br;q=1.0, zstd");
    expect_eq(std.mem.eql(u8, iter.next().?, "gzip"), true);
    expect_eq(std.mem.eql(u8, iter.next().?, "deflate"), true);
    expect_eq(std.mem.eql(u8, iter.next().?, "br;q=1.0"), true);
    expect_eq(std.mem.eql(u8, iter.next().?, "zstd"), true);
    expect_eq(iter.next() == null, true);

    // 6. HTTPRequestBody
    const body = HTTPRequestBody.fromBytes("Hello CP Request Body");
    expect_eq(body.len(), 21);
    expect_eq(body.isStream(), false);
}

fn testStep2_H2AndH3Subsystems(): !void {
    // 1. HTTP/2 Frame Header Wire Encoding & Decoding
    var orig_header = FrameHeader{};
    orig_header.length = 1024;
    orig_header.type = FrameType.Data;
    orig_header.flags = 0x01; // EndStream
    orig_header.stream_id = 7;

    var wire_buf: [9]u8 = undefined;
    orig_header.encode(&wire_buf);

    const decoded_opt = FrameHeader.decode(&wire_buf);
    expect_eq(decoded_opt != null, true);
    const decoded = decoded_opt.?;
    expect_eq(decoded.length, 1024);
    expect_eq(decoded.type == FrameType.Data, true);
    expect_eq(decoded.flags, 0x01);
    expect_eq(decoded.stream_id, 7);

    // 2. HTTP/2 Client Session & Stream Multiplexing
    var h2_session = H2ClientSession{};
    expect_eq(h2_session.canCreateStream(), true);

    var s1 = h2_session.newStream();
    expect_eq(s1.id, 1);
    expect_eq(s1.isOpen(), true);

    var s2 = h2_session.newStream();
    expect_eq(s2.id, 3);

    s1.close();
    expect_eq(s1.isOpen(), false);

    h2_session.onGoaway();
    expect_eq(h2_session.canCreateStream(), false);

    // 3. HTTP/3 Alt-Svc Parsing
    const alt_opt = AltSvc.parse("h3=\":8443\"; ma=3600");
    expect_eq(alt_opt != null, true);
    const alt = alt_opt.?;
    expect_eq(alt.port, 8443);
    expect_eq(alt.max_age, 3600);
    expect_eq(std.mem.eql(u8, alt.protocol, "h3"), true);

    // 4. HTTP/3 Client Session & Bidi Stream Allocation
    var h3_session = H3ClientSession{};
    var h3_s1 = h3_session.newRequestStream();
    expect_eq(h3_s1.id, 0);
    expect_eq(h3_s1.isOpen(), true);

    var h3_s2 = h3_session.newRequestStream();
    expect_eq(h3_s2.id, 4);

    h3_s1.close();
    expect_eq(h3_s1.isOpen(), false);
}

fn testStep3_CoreClientEngine(): !void {
    // 1. Proxy Tunnel Request Formatting & Validation
    const tunnel = ProxyTunnel.init("127.0.0.1", 8080, "api.example.com", 443);
    var connect_buf: [256]u8 = undefined;
    const connect_req = tunnel.formatConnectRequest(&connect_buf)?;
    expect_eq(std.mem.startsWith(u8, connect_req, "CONNECT api.example.com:443 HTTP/1.1\r\n"), true);
    expect_eq(ProxyTunnel.verifyConnectResponse("HTTP/1.1 200 Connection Established\r\n\r\n"), true);
    expect_eq(ProxyTunnel.verifyConnectResponse("HTTP/1.1 502 Bad Gateway\r\n\r\n"), false);

    // 2. SendFile Kernel Zero-Copy State Machine
    var sf = SendFile.init(3, 4096);
    expect_eq(sf.isDone(), false);
    expect_eq(sf.remain, 4096);

    // 3. HTTPContext Keep-Alive Connection Pooling
    var ctx = HTTPContext.init();
    defer ctx.deinit();

    // Release mock socket fd 101 into pool
    const released = ctx.releaseSocket(101, "api.example.com", 443, true);
    expect_eq(released, true);

    // Acquire socket from pool
    const acquired = ctx.acquireSocket("api.example.com", 443, true);
    expect_eq(acquired != null, true);
    expect_eq(acquired.?, 101);

    // Socket is now in use, acquiring again returns null
    const acquired2 = ctx.acquireSocket("api.example.com", 443, true);
    expect_eq(acquired2 == null, true);

    // Release it back
    expect_eq(ctx.releaseSocket(101, "api.example.com", 443, true), true);

    // Evict dead socket
    ctx.closeDeadSocket(101);
    expect_eq(ctx.acquireSocket("api.example.com", 443, true) == null, true);

    // 4. HTTPClient Request Serialization (RFC 7230)
    var client = HTTPClient.init(Method.POST, "http://api.myserver.org:9000/v1/chat/completions");
    expect_eq(std.mem.eql(u8, client.hostname, "api.myserver.org"), true);
    expect_eq(client.port, 9000);
    expect_eq(std.mem.eql(u8, client.path, "/v1/chat/completions"), true);

    client.body = "{\"model\":\"cp-gemini\",\"stream\":false}";

    var req_buf: [1024]u8 = undefined;
    const req_str = client.formatRequest(&req_buf)?;
    expect_eq(std.mem.startsWith(u8, req_str, "POST /v1/chat/completions HTTP/1.1\r\n"), true);
    expect_eq(std.mem.indexOf(u8, req_str, "Host: api.myserver.org:9000\r\n") != null, true);
    expect_eq(std.mem.indexOf(u8, req_str, "Content-Length: 36\r\n") != null, true);
    expect_eq(std.mem.indexOf(u8, req_str, "{\"model\":\"cp-gemini\",\"stream\":false}") != null, true);

    // 5. Response Status and Body Parsing
    const mock_response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"healthy\"}";
    const status_code = HTTPClient.parseResponseStatus(mock_response);
    expect_eq(status_code != null, true);
    expect_eq(status_code.?, 200);

    const resp_body = HTTPClient.parseResponseBody(mock_response);
    expect_eq(std.mem.eql(u8, resp_body, "{\"status\":\"healthy\"}"), true);

    // 6. Chunked Transfer Encoding Decoding
    const chunked_payload = "5\r\nHello\r\n8\r\n CP-Lang\r\n1\r\n!\r\n0\r\n\r\n";
    var decoded_buf: [128]u8 = undefined;
    const decoded_len = HTTPClient.decodeChunkedBody(chunked_payload, &decoded_buf)?;
    expect_eq(decoded_len, 14);
    expect_eq(std.mem.eql(u8, decoded_buf[0..decoded_len], "Hello CP-Lang!"), true);

    // 7. Decompressor Lifecycle
    var decomp = Decompressor.init(Encoding.Gzip);
    expect_eq(decomp.is_done, false);
    decomp.release();
    expect_eq(decomp.is_done, true);
}

fn main(): void {
    test("HTTP Step 1: Types, Enums & Path/Header Infrastructure", () => {
        testStep1_TypesAndInfrastructure()?;
    });

    test("HTTP Step 2: HTTP/2 Framing & HTTP/3 Client Subsystems", () => {
        testStep2_H2AndH3Subsystems()?;
    });

    test("HTTP Step 3: Industrial Client Engine, Keep-Alive Pooling & Chunked Decoding", () => {
        testStep3_CoreClientEngine()?;
    });
}
