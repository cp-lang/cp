// lib/dns/_cares.cp
// Low-level c-ares C-FFI bindings, NSClass/NSType & ARES status codes
// Absorbed from bun/src/cares_sys/c_ares.cp

export @opaque struct ares_channeldata {}
export @opaque struct ares_addrinfo {}
export @opaque struct ares_options {}

export const ARES_SUCCESS: i32 = 0;
export const ARES_ENODATA: i32 = 1;
export const ARES_EFORMERR: i32 = 2;
export const ARES_ESERVFAIL: i32 = 3;
export const ARES_ENOTFOUND: i32 = 4;
export const ARES_ENOTIMP: i32 = 5;
export const ARES_EREFUSED: i32 = 6;
export const ARES_EBADQUERY: i32 = 7;
export const ARES_EBADNAME: i32 = 8;
export const ARES_EBADFAMILY: i32 = 9;
export const ARES_EBADRESP: i32 = 10;
export const ARES_ECONNREFUSED: i32 = 11;
export const ARES_ETIMEOUT: i32 = 12;
export const ARES_EOF: i32 = 13;
export const ARES_EFILE: i32 = 14;
export const ARES_ENOMEM: i32 = 15;
export const ARES_EDESTRUCTION: i32 = 16;
export const ARES_EBADSTR: i32 = 17;
export const ARES_EBADFLAGS: i32 = 18;
export const ARES_ENONAME: i32 = 19;
export const ARES_EBADHINTS: i32 = 20;
export const ARES_ENOTINITIALIZED: i32 = 21;
export const ARES_ECANCELLED: i32 = 24;
export const ARES_ESERVICE: i32 = 25;
export const ARES_ENOSERVER: i32 = 26;

export enum NSClass: u16 {
    Invalid = 0,
    In = 1,
    Chaos = 3,
    Hs = 4,
    None = 254,
    Any = 255,
}

export enum NSType: u16 {
    Invalid = 0,
    A = 1,
    NS = 2,
    CNAME = 5,
    SOA = 6,
    PTR = 12,
    HINFO = 13,
    MX = 15,
    TXT = 16,
    AAAA = 28,
    SRV = 33,
    NAPTR = 35,
    OPT = 41,
    CAA = 257,
    ANY = 255,
}

export struct CaresStatus {
    pub static fn codeName(code: i32): []const u8 {
        return switch (code) {
            0 => "SUCCESS",
            1 => "ENODATA",
            2 => "EFORMERR",
            3 => "ESERVFAIL",
            4 => "ENOTFOUND",
            5 => "ENOTIMP",
            6 => "EREFUSED",
            7 => "EBADQUERY",
            8 => "EBADNAME",
            9 => "EBADFAMILY",
            10 => "EBADRESP",
            11 => "ECONNREFUSED",
            12 => "ETIMEOUT",
            13 => "EOF",
            15 => "ENOMEM",
            16 => "EDESTRUCTION",
            24 => "ECANCELLED",
            26 => "ENOSERVER",
            else => "EUNKNOWN",
        };
    }

    pub static fn isRetryable(code: i32): bool {
        return code == ARES_ESERVFAIL || code == ARES_ECONNREFUSED || code == ARES_ETIMEOUT || code == ARES_ENOSERVER;
    }
}

@c {
    pub fn ares_library_init(flags: i32): i32;
    pub fn ares_library_cleanup(): void;
    pub fn ares_init(channelptr: *(*ares_channeldata?!)): i32;
    pub fn ares_destroy(channel: *ares_channeldata!): void;
    pub fn ares_cancel(channel: *ares_channeldata!): void;
    pub fn ares_strerror(code: i32): cstr;
    pub fn ares_free_data(dataptr: *!): void;
    pub fn ares_freeaddrinfo(ai: *ares_addrinfo!): void;
}
