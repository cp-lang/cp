import { test, expect_eq } from "cap:test";
import {
    Z_MIN_WBITS,
    Z_MAX_WBITS,
    Z_BEST_COMPRESSION,
    ZlibFlush,
    ZlibStatus,
    CompressionFormat,
    CompressBoundEstimator,
    Crc32,
    Adler32,
    GzipHeader,
    GZIP_FNAME,
    RleCodec
} from "cap:compress";

fn main(): void {
    test("1. CompressionFormat negotiation & CompressBoundEstimator", () => {
        expect_eq(Z_MIN_WBITS, 8);
        expect_eq(Z_MAX_WBITS, 15);
        expect_eq(Z_BEST_COMPRESSION, 9);
        expect_eq(ZlibStatus.isSuccess(ZlibStatus.Ok as i32), true);
        expect_eq(ZlibStatus.isSuccess(ZlibStatus.StreamEnd as i32), true);
        expect_eq(ZlibStatus.isSuccess(ZlibStatus.DataError as i32), false);
        expect_eq(ZlibFlush.Finish as i32, 4);

        let fmt1 = CompressionFormat.fromAcceptEncoding("gzip, deflate, br, zstd");
        expect_eq(fmt1 == CompressionFormat.Zstd, true);
        expect_eq(CompressionFormat.contentEncodingName(fmt1), "zstd");

        let fmt2 = CompressionFormat.fromAcceptEncoding("gzip, deflate, br");
        expect_eq(fmt2 == CompressionFormat.Brotli, true);
        expect_eq(CompressionFormat.contentEncodingName(fmt2), "br");

        expect_eq(CompressBoundEstimator.zlibBound(1024) > 1024, true);
        expect_eq(CompressBoundEstimator.zstdBound(1024) > 1024, true);
    });

    test("2. Crc32 (IEEE 802.3) & Adler32 (RFC 1950) standard vectors", () => {
        // 标准测试向量 "123456789" -> IEEE CRC-32 = 0xCBF43926
        let crc = Crc32.checksum("123456789");
        expect_eq(crc, 0xCBF43926 as u32);

        // 流式增量计算验证
        let stream_crc = Crc32.init();
        stream_crc.update("1234");
        stream_crc.update("56789");
        expect_eq(stream_crc.final(), 0xCBF43926 as u32);

        // 标准测试向量 "Wikipedia" -> Adler-32 = 0x11E60398 (300286872)
        let adler = Adler32.checksum("Wikipedia");
        expect_eq(adler, 0x11E60398 as u32);
    });

    test("3. GzipHeader (RFC 1952) 10-byte header & 8-byte trailer verification", () => {
        let hdr = GzipHeader{
            flags: GZIP_FNAME,
            mtime: 1700000000,
            xfl: 2,
            os: 3,
        };
        let hdr_buf: [10]u8 = undefined;
        let written = hdr.write(hdr_buf[0..10]);
        expect_eq(written, 10 as usize);

        let parsed_opt = GzipHeader.parse(hdr_buf[0..10]);
        expect_eq(parsed_opt != null, true);
        if (parsed_opt) |parsed| {
            expect_eq(parsed.flags, GZIP_FNAME);
            expect_eq(parsed.mtime, 1700000000 as u32);
            expect_eq(parsed.xfl, 2 as u8);
            expect_eq(parsed.os, 3 as u8);
        }

        let payload = "Hello CP Gzip Frame Verification!";
        let crc = Crc32.checksum(payload);
        let trailer_buf: [8]u8 = undefined;
        let t_written = GzipHeader.writeTrailer(trailer_buf[0..8], crc, payload.len as u32);
        expect_eq(t_written, 8 as usize);
        expect_eq(GzipHeader.verifyTrailer(trailer_buf[0..8], payload), true);
        expect_eq(GzipHeader.verifyTrailer(trailer_buf[0..8], "Tampered"), false);
    });

    test("4. RleCodec lossless compression and decompression roundtrip", () => {
        let src = "AAAAAAAAAAAAABBBBBBBBBCCCCCDDDDXYZ";
        let compressed: [64]u8 = undefined;
        let comp_len = RleCodec.compress(src, compressed[0..64]);
        expect_eq(comp_len > 0, true);
        expect_eq(comp_len < src.len, true);

        let decompressed: [64]u8 = undefined;
        let decomp_len = RleCodec.decompress(compressed[0..comp_len], decompressed[0..64]);
        expect_eq(decomp_len, src.len);
        expect_eq(decompressed[0..decomp_len] == src, true);
    });
}
