/**
 * CP Web Fetch Request Enums & Modes (cap:http)
 * Ported from Bun.js http_types (FetchCacheMode, FetchRedirect, FetchRequestMode)
 */

const std = @import("std");

/**
 * https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
 */
export const FetchCacheMode = enum(u3) {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,

    pub fn fromString(str: []const u8) ?FetchCacheMode {
        if (std.mem.eql(u8, str, "default")) return .Default;
        if (std.mem.eql(u8, str, "no-store")) return .NoStore;
        if (std.mem.eql(u8, str, "reload")) return .Reload;
        if (std.mem.eql(u8, str, "no-cache")) return .NoCache;
        if (std.mem.eql(u8, str, "force-cache")) return .ForceCache;
        if (std.mem.eql(u8, str, "only-if-cached")) return .OnlyIfCached;
        return null;
    }

    pub fn get(str: []const u8) ?FetchCacheMode {
        return fromString(str);
    }
};

/**
 * https://developer.mozilla.org/en-US/docs/Web/API/Request/redirect
 */
export const FetchRedirect = enum(u2) {
    Follow,
    Manual,
    Error,

    pub fn fromString(str: []const u8) ?FetchRedirect {
        if (std.mem.eql(u8, str, "follow")) return .Follow;
        if (std.mem.eql(u8, str, "manual")) return .Manual;
        if (std.mem.eql(u8, str, "error")) return .Error;
        return null;
    }

    pub fn get(str: []const u8) ?FetchRedirect {
        return fromString(str);
    }
};

/**
 * https://developer.mozilla.org/en-US/docs/Web/API/Request/mode
 */
export const FetchRequestMode = enum(u2) {
    SameOrigin,
    NoCors,
    Cors,
    Navigate,

    pub fn fromString(str: []const u8) ?FetchRequestMode {
        if (std.mem.eql(u8, str, "same-origin")) return .SameOrigin;
        if (std.mem.eql(u8, str, "no-cors")) return .NoCors;
        if (std.mem.eql(u8, str, "cors")) return .Cors;
        if (std.mem.eql(u8, str, "navigate")) return .Navigate;
        return null;
    }

    pub fn get(str: []const u8) ?FetchRequestMode {
        return fromString(str);
    }
};
