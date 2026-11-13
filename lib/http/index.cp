// lib/http/index.cp
// CP Standard HTTP Library (cap:http) - Industrial-grade unified HTTP subsystem

export { Method } from "./method";
export { Headers, HeaderEntry } from "./headers";
export { ETag, ParsedETag } from "./etag";
export { MimeType } from "./mime";
export { Request } from "./request";
export { Response } from "./response";
export { Server, fetch, json, serve } from "./server";

// --- Step 1: HTTP Infrastructure Types & Enums ---
export { Encoding } from "./encoding";
export { FetchCacheMode, FetchRedirect, FetchRequestMode } from "./fetch_modes";
export { HTTPCertError, CertificateInfo, InitError } from "./cert";
export { URLPath } from "./url_path";
export { SignalStore, Signals } from "./signals";
export { HeaderValueIterator } from "./header_iterator";
export { HTTPRequestBody, HTTPRequestBodyTag } from "./request_body";
export {
    HTTPVerboseLevel,
    HTTPUpgradeState,
    Protocol,
    ShouldContinue,
    HTTPStage,
    Stage,
    Flags,
    InternalStateFlags,
    InternalState
} from "./internal_state";

// --- Step 2: HTTP/2 & HTTP/3 Subsystems ---
export {
    CLIENT_PREFACE,
    FrameType,
    ErrorCode,
    SettingsType,
    FrameHeader,
    H2StreamState,
    H2Stream,
    H2ClientSession
} from "./h2/index";

export {
    AltSvcEntry,
    AltSvc,
    H3StreamType,
    H3Stream,
    H3ClientSession
} from "./h3/index";

// --- Step 3: Industrial Client Engine & Lifecycle ---
export {
    DecompressorType,
    Decompressor
} from "./decompressor";

export {
    ProxyTunnelSocketType,
    ProxyTunnel
} from "./proxy";

export {
    SendFileStatus,
    SendFile
} from "./sendfile";

export {
    MAX_POOLED_SOCKETS,
    PooledSocket,
    HTTPContext
} from "./context";

export {
    HTTPClientResult,
    HTTPClient
} from "./client";

// --- Step 4: Low-level PicoHTTPParser C-FFI & Zero-Copy Wire Parser ---
export {
    phr_header,
    ChunkedEncodingState,
    phr_chunked_decoder,
    phr_parse_request,
    phr_parse_response,
    phr_parse_headers,
    phr_decode_chunked,
    phr_decode_chunked_is_in_data,
    PicoHeader,
    PicoRequest,
    PicoResponse,
    PicoHttpParser
} from "./_picohttp";
