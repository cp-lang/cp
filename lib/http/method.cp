// lib/http/method.cp
// HTTP Method enumeration and helper functions
const std = @import("std");

export enum Method: u8 {
    ACL,
    BIND,
    CHECKOUT,
    CONNECT,
    COPY,
    DELETE,
    GET,
    HEAD,
    LINK,
    LOCK,
    MERGE,
    MKACTIVITY,
    MKCALENDAR,
    MKCOL,
    MOVE,
    NOTIFY,
    OPTIONS,
    PATCH,
    POST,
    PROPFIND,
    PROPPATCH,
    PURGE,
    PUT,
    QUERY,
    REBIND,
    REPORT,
    SEARCH,
    SOURCE,
    SUBSCRIBE,
    TRACE,
    UNBIND,
    UNLINK,
    UNLOCK,
    UNSUBSCRIBE,

    pub fn isIdempotent(this): bool {
        return switch (this) {
            .GET, .HEAD, .PUT, .DELETE, .OPTIONS, .TRACE, .QUERY => true,
            else => false,
        };
    }

    pub fn hasRequestBody(this): bool {
        return switch (this) {
            .GET, .HEAD, .OPTIONS, .TRACE => false,
            else => true,
        };
    }

    pub fn toString(this): string {
        return @tagName(this);
    }
}
