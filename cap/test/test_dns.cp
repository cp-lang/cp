import { test, expect_eq } from "cap:test";
import {
    ARES_SUCCESS,
    ARES_ETIMEOUT,
    ARES_ENOTFOUND,
    NSClass,
    NSType,
    CaresStatus,
    DnsRCode,
    DnsHeader,
    DnsWireCodec,
    DnsFamily,
    DnsCache,
} from "cap:dns";

fn main(): void {
    test("1. c-ares status codes, retryability & NSClass/NSType constants", () => {
        expect_eq(CaresStatus.codeName(ARES_SUCCESS), "SUCCESS");
        expect_eq(CaresStatus.codeName(ARES_ETIMEOUT), "ETIMEOUT");
        expect_eq(CaresStatus.codeName(ARES_ENOTFOUND), "ENOTFOUND");
        expect_eq(CaresStatus.isRetryable(ARES_ETIMEOUT), true);
        expect_eq(CaresStatus.isRetryable(ARES_ENOTFOUND), false);
        expect_eq(NSClass.In as u16, 1);
        expect_eq(NSType.A as u16, 1);
        expect_eq(NSType.AAAA as u16, 28);
        expect_eq(NSType.SRV as u16, 33);
    });

    test("2. RFC 1035 12-byte DNS Wire Header encode & decode roundtrip", () => {
        const hdr = DnsHeader{
            id: 0x1337,
            is_response: true,
            opcode: 0,
            authoritative: true,
            truncated: false,
            recursion_desired: true,
            recursion_available: true,
            rcode: DnsRCode.NoError,
            qd_count: 1,
            an_count: 2,
            ns_count: 0,
            ar_count: 1,
        };
        let wire: [12]u8 = [0; 12];
        expect_eq(hdr.encode(wire[0..12]), 12);

        if (dec_hdr := DnsHeader.decode(wire[0..12])) {
            expect_eq(dec_hdr.id, 0x1337);
            expect_eq(dec_hdr.is_response, true);
            expect_eq(dec_hdr.authoritative, true);
            expect_eq(dec_hdr.recursion_desired, true);
            expect_eq(dec_hdr.recursion_available, true);
            expect_eq(dec_hdr.rcode == DnsRCode.NoError, true);
            expect_eq(dec_hdr.qd_count, 1);
            expect_eq(dec_hdr.an_count, 2);
            expect_eq(dec_hdr.ar_count, 1);
        } else {
            expect_eq(true, false);
        }
    });

    test("3. RFC 1035 domain label encoding, query packet & 0xC0 pointer decompression", () => {
        let pkt: [64]u8 = [0; 64];
        const pkt_len = DnsWireCodec.buildQueryPacket(0x4242, "api.cap.dev", NSType.A, pkt[0..64]);
        // 12 (header) + 13 (\x03api\x03cap\x03dev\x00) + 4 (QTYPE + QCLASS) = 29 bytes
        expect_eq(pkt_len, 29);

        let name_out: [64]u8 = [0; 64];
        if (d1 := DnsWireCodec.decodeDomainName(pkt[0..pkt_len], 12, name_out[0..64])) {
            expect_eq(name_out[0..d1.name_len], "api.cap.dev");
            expect_eq(d1.packet_bytes_consumed, 13);
        } else {
            expect_eq(true, false);
        }

        // Append a compressed domain name "\x04edge\xC0\x10" at offset 29 (pointing to offset 16 = "\x03cap\x03dev\x00")
        pkt[29] = 4;
        pkt[30] = 101; // 'e'
        pkt[31] = 100; // 'd'
        pkt[32] = 103; // 'g'
        pkt[33] = 101; // 'e'
        pkt[34] = 0xC0;
        pkt[35] = 16;

        let comp_out: [64]u8 = [0; 64];
        if (d2 := DnsWireCodec.decodeDomainName(pkt[0..36], 29, comp_out[0..64])) {
            expect_eq(comp_out[0..d2.name_len], "edge.cap.dev");
            expect_eq(d2.packet_bytes_consumed, 7);
        } else {
            expect_eq(true, false);
        }
    });

    test("4. DnsCache<CAP> TTL lookup, hit/miss tracking & expiration eviction", () => {
        expect_eq(DnsFamily.fromString("IPv4") == DnsFamily.IPv4, true);
        expect_eq(DnsFamily.fromString("ipv6") == DnsFamily.IPv6, true);

        let cache = DnsCache<16>.init();
        expect_eq(cache.activeCount(), 0);

        // Insert entry at t=1000ms with TTL=5000ms (expires at 6000ms)
        expect_eq(cache.put("cap.dev", 0x7F000001, 443, 1000, 5000), true);
        expect_eq(cache.activeCount(), 1);

        // Hit at t=3000ms
        if (ip := cache.lookup("cap.dev", 443, 3000)) {
            expect_eq(ip, 0x7F000001);
        } else {
            expect_eq(true, false);
        }
        expect_eq(cache.total_hits, 1);

        // Expired at t=6000ms -> returns null and evicts
        const expired = cache.lookup("cap.dev", 443, 6000);
        expect_eq(expired == null, true);
        expect_eq(cache.activeCount(), 0);
        expect_eq(cache.total_misses, 1);
    });
}
