/**
 * CP HTTP/3 Alt-Svc Parser (cap:http)
 * RFC 7838 Alt-Svc header handling for HTTP/3 client upgrade
 * Ported from Bun.js http/h3_client/AltSvc.cp
 */

const std = @import("std");

export const AltSvcEntry = struct {
    protocol: []const u8 = "h3",
    port: u16 = 443,
    max_age: u32 = 86400,
};

export const AltSvc = struct {
    pub fn parse(field_value: []const u8) ?AltSvcEntry {
        const trimmed = std.mem.trim(u8, field_value, " \t");
        if (trimmed.len == 0 or std.mem.eql(u8, trimmed, "clear")) {
            return null;
        }

        // Example: h3=":443"; ma=86400, h3-29=":443"
        var entries = std.mem.splitScalar(u8, trimmed, ',');
        while (entries.next()) |raw_entry| {
            const entry = std.mem.trim(u8, raw_entry, " \t");
            if (entry.len == 0) continue;

            var params = std.mem.splitScalar(u8, entry, ';');
            const alt = std.mem.trim(u8, params.first(), " \t");

            if (std.mem.indexOfScalar(u8, alt, '=')) |eq| {
                const proto = alt[0..eq];
                if (!std.mem.eql(u8, proto, "h3")) continue;

                var auth = std.mem.trim(u8, alt[eq + 1..], " \t");
                if (auth.len >= 2 and auth[0] == '"' and auth[auth.len - 1] == '"') {
                    auth = auth[1..auth.len - 1];
                }
                if (std.mem.lastIndexOfScalar(u8, auth, ':')) |colon| {
                    const port_str = auth[colon + 1..];
                    const port = std.fmt.parseInt(u16, port_str, 10) catch 0;
                    if (port == 0) continue;

                    var ma: u32 = 86400;
                    while (params.next()) |raw_p| {
                        const p = std.mem.trim(u8, raw_p, " \t");
                        if (std.mem.indexOfScalar(u8, p, '=')) |peq| {
                            const k = p[0..peq];
                            const v = p[peq + 1..];
                            if (std.mem.eql(u8, k, "ma")) {
                                ma = std.fmt.parseInt(u32, v, 10) catch 86400;
                            }
                        }
                    }

                    var res = AltSvcEntry{};
                    res.protocol = "h3";
                    res.port = port;
                    res.max_age = ma;
                    return res;
                }
            }
        }
        return null;
    }
};
