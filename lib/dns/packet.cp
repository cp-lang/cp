// lib/dns/packet.cp
// RFC 1035 DNS Wire Packet Header, Domain Label Codec & Pointer Decompression

import { NSClass, NSType } from "./_cares.cp";

export enum DnsRCode: u8 {
    NoError = 0,
    FormErr = 1,
    ServFail = 2,
    NXDomain = 3,
    NotImp = 4,
    Refused = 5,
}

// RFC 1035 Section 4.1.1 12-byte DNS Wire Header
export struct DnsHeader {
    pub id: u16 = 0;
    pub is_response: bool = false;
    pub opcode: u8 = 0;
    pub authoritative: bool = false;
    pub truncated: bool = false;
    pub recursion_desired: bool = true;
    pub recursion_available: bool = false;
    pub rcode: DnsRCode = .NoError;
    pub qd_count: u16 = 1;
    pub an_count: u16 = 0;
    pub ns_count: u16 = 0;
    pub ar_count: u16 = 0;

    pub fn encode(this, out: []u8): usize {
        if (out.len < 12) {
            return 0;
        }
        out[0] = (this.id >> 8) as u8;
        out[1] = (this.id & 0xFF) as u8;

        let flags_hi: u8 = (this.opcode & 0x0F) << 3;
        if (this.is_response) {
            flags_hi = flags_hi | 0x80;
        }
        if (this.authoritative) {
            flags_hi = flags_hi | 0x04;
        }
        if (this.truncated) {
            flags_hi = flags_hi | 0x02;
        }
        if (this.recursion_desired) {
            flags_hi = flags_hi | 0x01;
        }
        let flags_lo: u8 = (this.rcode as u8) & 0x0F;
        if (this.recursion_available) {
            flags_lo = flags_lo | 0x80;
        }
        out[2] = flags_hi;
        out[3] = flags_lo;

        out[4] = (this.qd_count >> 8) as u8;
        out[5] = (this.qd_count & 0xFF) as u8;
        out[6] = (this.an_count >> 8) as u8;
        out[7] = (this.an_count & 0xFF) as u8;
        out[8] = (this.ns_count >> 8) as u8;
        out[9] = (this.ns_count & 0xFF) as u8;
        out[10] = (this.ar_count >> 8) as u8;
        out[11] = (this.ar_count & 0xFF) as u8;
        return 12;
    }

    pub static fn decode(buf: []const u8): DnsHeader? {
        if (buf.len < 12) {
            return null;
        }
        const id: u16 = ((buf[0] as u16) << 8) | (buf[1] as u16);
        const f_hi = buf[2];
        const f_lo = buf[3];
        const rcode_raw = f_lo & 0x0F;
        const rcode = switch (rcode_raw) {
            0 => DnsRCode.NoError,
            1 => DnsRCode.FormErr,
            2 => DnsRCode.ServFail,
            3 => DnsRCode.NXDomain,
            4 => DnsRCode.NotImp,
            else => DnsRCode.Refused,
        };
        return DnsHeader{
            id: id,
            is_response: (f_hi & 0x80) != 0,
            opcode: (f_hi >> 3) & 0x0F,
            authoritative: (f_hi & 0x04) != 0,
            truncated: (f_hi & 0x02) != 0,
            recursion_desired: (f_hi & 0x01) != 0,
            recursion_available: (f_lo & 0x80) != 0,
            rcode: rcode,
            qd_count: ((buf[4] as u16) << 8) | (buf[5] as u16),
            an_count: ((buf[6] as u16) << 8) | (buf[7] as u16),
            ns_count: ((buf[8] as u16) << 8) | (buf[9] as u16),
            ar_count: ((buf[10] as u16) << 8) | (buf[11] as u16),
        };
    }
}

export struct DecodedDomain {
    pub name_len: usize = 0;
    pub packet_bytes_consumed: usize = 0;
}

export struct DnsWireCodec {
    // Encodes "api.cap.dev" -> "\x03api\x03cap\x03dev\x00"
    pub static fn encodeDomainName(domain: []const u8, out: []u8): usize {
        if (domain.len == 0 || domain.len > 253) {
            return 0;
        }
        let pos: usize = 0;
        let label_start: usize = 0;
        let i: usize = 0;
        while (i <= domain.len) {
            if (i == domain.len || domain[i] == 46) { // '.' = 46
                const label_len = i - label_start;
                if (label_len == 0 || label_len > 63 || pos + 1 + label_len + 1 > out.len) {
                    return 0;
                }
                out[pos] = label_len as u8;
                pos += 1;
                let k: usize = 0;
                while (k < label_len) {
                    out[pos + k] = domain[label_start + k];
                    k += 1;
                }
                pos += label_len;
                label_start = i + 1;
            }
            i += 1;
        }
        out[pos] = 0;
        return pos + 1;
    }

    // Decodes wire domain name (supporting RFC 1035 Section 4.1.4 `0xC0` pointer compression!)
    pub static fn decodeDomainName(packet: []const u8, start_offset: usize, out_name: []u8): DecodedDomain? {
        let cur = start_offset;
        let out_pos: usize = 0;
        let consumed: usize = 0;
        let jumped = false;
        let hops: usize = 0;

        while (cur < packet.len && hops < 16) {
            const len_byte = packet[cur];
            if (len_byte == 0) {
                if (!jumped) {
                    consumed = (cur - start_offset) + 1;
                }
                return DecodedDomain{
                    name_len: out_pos,
                    packet_bytes_consumed: consumed,
                };
            }
            // Check pointer compression (top 2 bits == 11 -> 0xC0)
            if ((len_byte & 0xC0) == 0xC0) {
                if (cur + 1 >= packet.len) {
                    return null;
                }
                if (!jumped) {
                    consumed = (cur - start_offset) + 2;
                    jumped = true;
                }
                const target_offset: usize = (((len_byte & 0x3F) as usize) << 8) | (packet[cur + 1] as usize);
                if (target_offset >= packet.len) {
                    return null;
                }
                cur = target_offset;
                hops += 1;
            } else {
                const label_len = len_byte as usize;
                cur += 1;
                if (cur + label_len > packet.len) {
                    return null;
                }
                if (out_pos > 0) {
                    if (out_pos + 1 > out_name.len) {
                        return null;
                    }
                    out_name[out_pos] = 46; // '.'
                    out_pos += 1;
                }
                if (out_pos + label_len > out_name.len) {
                    return null;
                }
                let k: usize = 0;
                while (k < label_len) {
                    out_name[out_pos + k] = packet[cur + k];
                    k += 1;
                }
                out_pos += label_len;
                cur += label_len;
            }
        }
        return null;
    }

    // Builds a complete DNS query packet (12-byte header + encoded QNAME + QTYPE + QCLASS)
    pub static fn buildQueryPacket(id: u16, domain: []const u8, qtype: NSType, out: []u8): usize {
        const hdr = DnsHeader{
            id: id,
            is_response: false,
            opcode: 0,
            authoritative: false,
            truncated: false,
            recursion_desired: true,
            recursion_available: false,
            rcode: .NoError,
            qd_count: 1,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        };
        const hdr_len = hdr.encode(out);
        if (hdr_len == 0) {
            return 0;
        }
        const qname_len = DnsWireCodec.encodeDomainName(domain, out[12..out.len]);
        if (qname_len == 0) {
            return 0;
        }
        const pos = 12 + qname_len;
        if (pos + 4 > out.len) {
            return 0;
        }
        const qt: u16 = qtype as u16;
        const qc: u16 = NSClass.In as u16;
        out[pos] = (qt >> 8) as u8;
        out[pos + 1] = (qt & 0xFF) as u8;
        out[pos + 2] = (qc >> 8) as u8;
        out[pos + 3] = (qc & 0xFF) as u8;
        return pos + 4;
    }
}
