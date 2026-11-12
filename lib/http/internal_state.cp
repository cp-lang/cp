/**
 * CP HTTP Internal State & Protocols (cap:http)
 * Ported from Bun.js http/InternalState.cp & http/http.cp
 */

import { Encoding } from "./encoding";
import { CertificateInfo } from "./cert";

export enum HTTPVerboseLevel {
    None,
    Headers,
    Curl,
}

export enum HTTPUpgradeState: u2 {
    None = 0,
    Pending = 1,
    Upgraded = 2,
}

export enum Protocol: u2 {
    Http1_1 = 0,
    Http2 = 1,
    Http3 = 2,
}

export enum ShouldContinue {
    ContinueStreaming,
    Finished,
}

export enum HTTPStage {
    Pending,
    Opened,
    Headers,
    Body,
    BodyChunk,
    Fail,
    Done,
    ProxyHandshake,
    ProxyHeaders,
    ProxyBody,
}

export enum Stage: u8 {
    Pending,
    Connect,
    Done,
    Fail,
}

export const Flags = struct {
    disable_timeout: bool = false,
    disable_keepalive: bool = false,
    disable_decompression: bool = false,
    did_have_handshaking_error: bool = false,
    force_last_modified: bool = false,
    redirected: bool = false,
    proxy_tunneling: bool = false,
    reject_unauthorized: bool = true,
    is_preconnect_only: bool = false,
    is_streaming_request_body: bool = false,
    defer_fail_until_connecting_is_complete: bool = false,
    upgrade_state: HTTPUpgradeState = HTTPUpgradeState.None,
    protocol: Protocol = Protocol.Http1_1,
    force_http2: bool = false,
    force_http1: bool = false,
    force_http3: bool = false,
    h3_retried: bool = false,
};

export const InternalStateFlags = struct {
    allow_keepalive: bool = true,
    received_last_chunk: bool = false,
    did_set_content_encoding: bool = false,
    is_redirect_pending: bool = false,
    is_libdeflate_fast_path_disabled: bool = false,
    resend_request_body_on_redirect: bool = false,
};

export const InternalState = struct {
    flags: InternalStateFlags = InternalStateFlags{},
    transfer_encoding: Encoding = Encoding.Identity,
    encoding: Encoding = Encoding.Identity,
    stage: Stage = Stage.Pending,
    request_stage: HTTPStage = HTTPStage.Pending,
    response_stage: HTTPStage = HTTPStage.Pending,
    content_length: ?usize = null,
    total_body_received: usize = 0,
    request_body: []const u8 = "",
    request_sent_len: usize = 0,
    certificate_info: ?CertificateInfo = null,

    pub fn isChunkedEncoding(self: InternalState) bool {
        return self.transfer_encoding == Encoding.Chunked;
    }

    pub fn isDone(self: InternalState) bool {
        if (self.isChunkedEncoding()) {
            return self.flags.received_last_chunk;
        }
        if (self.content_length) |cl| {
            return self.total_body_received >= cl;
        }
        return self.flags.received_last_chunk;
    }
};
