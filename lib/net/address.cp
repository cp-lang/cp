// lib/net/address.cp
// High-performance IPv4 / IPv6 SocketAddress representation & parser
// Absorbed from Bun src/runtime/socket/SocketAddress.cp and src/uws/uws.cp

const std = @import("std");

export enum AddressFamily: u8 {
    INET = 4,
    INET6 = 6,
}

export struct Ipv4Addr {
    pub octets: [4]u8 = [0, 0, 0, 0];

    pub static fn init(a: u8, b: u8, c: u8, d: u8): Ipv4Addr {
        return Ipv4Addr { octets: [a, b, c, d] };
    }

    pub static fn loopback(): Ipv4Addr {
        return Ipv4Addr.init(127, 0, 0, 1);
    }

    pub static fn any(): Ipv4Addr {
        return Ipv4Addr.init(0, 0, 0, 0);
    }

    pub static fn broadcast(): Ipv4Addr {
        return Ipv4Addr.init(255, 255, 255, 255);
    }

    pub static fn parse(text: []const u8): Ipv4Addr? {
        var result: [4]u8 = [0, 0, 0, 0];
        var part_idx: usize = 0;
        var current: u32 = 0;
        var has_digit: bool = false;

        var i: usize = 0;
        while (i < text.len) {
            const ch = text[i];
            if (ch >= 48 && ch <= 57) {
                current = current * 10 + ((ch - 48) as u32);
                if (current > 255) {
                    return null;
                }
                has_digit = true;
            } else if (ch == 46) {
                if (!has_digit || part_idx >= 3) {
                    return null;
                }
                result[part_idx] = current as u8;
                part_idx += 1;
                current = 0;
                has_digit = false;
            } else {
                return null;
            }
            i += 1;
        }

        if (!has_digit || part_idx != 3) {
            return null;
        }
        result[3] = current as u8;
        return Ipv4Addr { octets: result };
    }

    pub fn isLoopback(&this): bool {
        return this.octets[0] == 127;
    }

    pub fn isUnspecified(&this): bool {
        return this.octets[0] == 0 && this.octets[1] == 0 && this.octets[2] == 0 && this.octets[3] == 0;
    }

    pub fn isPrivate(&this): bool {
        if (this.octets[0] == 10) {
            return true;
        }
        if (this.octets[0] == 172 && this.octets[1] >= 16 && this.octets[1] <= 31) {
            return true;
        }
        if (this.octets[0] == 192 && this.octets[1] == 168) {
            return true;
        }
        return false;
    }

    pub fn isMulticast(&this): bool {
        return this.octets[0] >= 224 && this.octets[0] <= 239;
    }

    pub fn toU32(&this): u32 {
        const b0 = (this.octets[0] as u32) << 24;
        const b1 = (this.octets[1] as u32) << 16;
        const b2 = (this.octets[2] as u32) << 8;
        const b3 = this.octets[3] as u32;
        return b0 | b1 | b2 | b3;
    }

    pub static fn fromU32(val: u32): Ipv4Addr {
        return Ipv4Addr {
            octets: [
                ((val >> 24) & 0xFF) as u8,
                ((val >> 16) & 0xFF) as u8,
                ((val >> 8) & 0xFF) as u8,
                (val & 0xFF) as u8,
            ],
        };
    }
}

export struct Ipv6Addr {
    pub segments: [8]u16 = [0, 0, 0, 0, 0, 0, 0, 0];

    pub static fn loopback(): Ipv6Addr {
        return Ipv6Addr { segments: [0, 0, 0, 0, 0, 0, 0, 1] };
    }

    pub static fn any(): Ipv6Addr {
        return Ipv6Addr { segments: [0, 0, 0, 0, 0, 0, 0, 0] };
    }

    pub static fn fromIpv4Mapped(v4: Ipv4Addr): Ipv6Addr {
        const hi: u16 = ((v4.octets[0] as u16) << 8) | (v4.octets[1] as u16);
        const lo: u16 = ((v4.octets[2] as u16) << 8) | (v4.octets[3] as u16);
        return Ipv6Addr { segments: [0, 0, 0, 0, 0, 0xFFFF, hi, lo] };
    }

    pub fn isLoopback(&this): bool {
        return this.segments[0] == 0 &&
            this.segments[1] == 0 &&
            this.segments[2] == 0 &&
            this.segments[3] == 0 &&
            this.segments[4] == 0 &&
            this.segments[5] == 0 &&
            this.segments[6] == 0 &&
            this.segments[7] == 1;
    }

    pub fn isUnspecified(&this): bool {
        var i: usize = 0;
        while (i < 8) {
            if (this.segments[i] != 0) {
                return false;
            }
            i += 1;
        }
        return true;
    }

    pub fn isIpv4Mapped(&this): bool {
        return this.segments[0] == 0 &&
            this.segments[1] == 0 &&
            this.segments[2] == 0 &&
            this.segments[3] == 0 &&
            this.segments[4] == 0 &&
            this.segments[5] == 0xFFFF;
    }
}

export struct SocketAddress {
    pub family: AddressFamily = AddressFamily.INET;
    pub ipv4: Ipv4Addr = Ipv4Addr {};
    pub ipv6: Ipv6Addr = Ipv6Addr {};
    pub port: u16 = 0;
    pub flowlabel: u32 = 0;
    pub scope_id: u32 = 0;

    pub static fn v4(addr: Ipv4Addr, port: u16): SocketAddress {
        return SocketAddress {
            family: AddressFamily.INET,
            ipv4: addr,
            ipv6: Ipv6Addr.any(),
            port: port,
            flowlabel: 0,
            scope_id: 0,
        };
    }

    pub static fn v6(addr: Ipv6Addr, port: u16, flowlabel: u32, scope_id: u32): SocketAddress {
        return SocketAddress {
            family: AddressFamily.INET6,
            ipv4: Ipv4Addr.any(),
            ipv6: addr,
            port: port,
            flowlabel: flowlabel,
            scope_id: scope_id,
        };
    }

    pub static fn loopbackV4(port: u16): SocketAddress {
        return SocketAddress.v4(Ipv4Addr.loopback(), port);
    }

    pub static fn loopbackV6(port: u16): SocketAddress {
        return SocketAddress.v6(Ipv6Addr.loopback(), port, 0, 0);
    }

    pub static fn parseV4(host: []const u8, port: u16): SocketAddress? {
        if (ip := Ipv4Addr.parse(host)) {
            return SocketAddress.v4(ip, port);
        }
        return null;
    }

    pub fn isIPv6(&this): bool {
        return this.family == AddressFamily.INET6;
    }

    pub fn socklen(&this): u32 {
        if (this.family == AddressFamily.INET6) {
            return 28;
        }
        return 16;
    }
}
