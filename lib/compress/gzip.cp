// ============================================================================
// lib/compress/gzip.cp — RFC 1952 Gzip 帧头/帧尾编解码与高速 RLE 字节压缩器
// ============================================================================

import { Crc32 } from "./checksum.cp";

export const GZIP_MAGIC_ID1: u8 = 0x1F;
export const GZIP_MAGIC_ID2: u8 = 0x8B;
export const GZIP_METHOD_DEFLATE: u8 = 8;

export const GZIP_FTEXT: u8 = 0x01;
export const GZIP_FHCRC: u8 = 0x02;
export const GZIP_FEXTRA: u8 = 0x04;
export const GZIP_FNAME: u8 = 0x08;
export const GZIP_FCOMMENT: u8 = 0x10;

/// RFC 1952 10 字节 Gzip 帧头与 8 字节帧尾 (CRC32 + ISIZE)
export struct GzipHeader {
    flags: u8 = 0,
    mtime: u32 = 0,
    xfl: u8 = 0,
    os: u8 = 3, // 3 = Unix

    /// 将 10 字节标准 Gzip Header 写入 out[0..10]
    pub fn write(&this, out: []u8): usize {
        if (out.len < 10) { return 0; }
        out[0] = GZIP_MAGIC_ID1;
        out[1] = GZIP_MAGIC_ID2;
        out[2] = GZIP_METHOD_DEFLATE;
        out[3] = this.flags;
        out[4] = (this.mtime & 0xFF) as u8;
        out[5] = ((this.mtime >> 8) & 0xFF) as u8;
        out[6] = ((this.mtime >> 16) & 0xFF) as u8;
        out[7] = ((this.mtime >> 24) & 0xFF) as u8;
        out[8] = this.xfl;
        out[9] = this.os;
        return 10;
    }

    /// 解析 10 字节标准 Gzip Header
    pub static fn parse(buf: []const u8): GzipHeader? {
        if (buf.len < 10) { return null; }
        if (buf[0] != GZIP_MAGIC_ID1 || buf[1] != GZIP_MAGIC_ID2 || buf[2] != GZIP_METHOD_DEFLATE) {
            return null;
        }
        let mtime: u32 = (buf[4] as u32)
            | ((buf[5] as u32) << 8)
            | ((buf[6] as u32) << 16)
            | ((buf[7] as u32) << 24);
        return GzipHeader{
            flags: buf[3],
            mtime: mtime,
            xfl: buf[8],
            os: buf[9],
        };
    }

    /// 写入 8 字节小端序 Gzip Trailer (CRC32 + Uncompressed ISIZE mod 2^32)
    pub static fn writeTrailer(out: []u8, crc32: u32, isize_val: u32): usize {
        if (out.len < 8) { return 0; }
        out[0] = (crc32 & 0xFF) as u8;
        out[1] = ((crc32 >> 8) & 0xFF) as u8;
        out[2] = ((crc32 >> 16) & 0xFF) as u8;
        out[3] = ((crc32 >> 24) & 0xFF) as u8;
        out[4] = (isize_val & 0xFF) as u8;
        out[5] = ((isize_val >> 8) & 0xFF) as u8;
        out[6] = ((isize_val >> 16) & 0xFF) as u8;
        out[7] = ((isize_val >> 24) & 0xFF) as u8;
        return 8;
    }

    /// 校验 8 字节 Gzip Trailer 是否与预期数据匹配
    pub static fn verifyTrailer(trailer: []const u8, payload: []const u8): bool {
        if (trailer.len < 8) { return false; }
        let expected_crc: u32 = (trailer[0] as u32)
            | ((trailer[1] as u32) << 8)
            | ((trailer[2] as u32) << 16)
            | ((trailer[3] as u32) << 24);
        let expected_size: u32 = (trailer[4] as u32)
            | ((trailer[5] as u32) << 8)
            | ((trailer[6] as u32) << 16)
            | ((trailer[7] as u32) << 24);
        let actual_crc = Crc32.checksum(payload);
        let actual_size = (payload.len & 0xFFFFFFFF) as u32;
        return expected_crc == actual_crc && expected_size == actual_size;
    }
}

/// 零依赖 PackBits / Run-Length Encoding (RLE) 字节流无损压缩与解压器
/// 控制字节约定：
/// - `0xFF, count, byte`：连续重复 `count` 次 `byte`（当连续重复 >= 4 次或原字节本身为 `0xFF` 时触发）
/// - 其他字节：原样直通输出
export struct RleCodec {
    pub static fn compress(src: []const u8, out: []u8): usize {
        let in_pos: usize = 0;
        let out_pos: usize = 0;
        while (in_pos < src.len) {
            let b = src[in_pos];
            let run_len: usize = 1;
            while (in_pos + run_len < src.len && src[in_pos + run_len] == b && run_len < 255) {
                run_len += 1;
            }
            if (run_len >= 4 || b == 0xFF) {
                if (out_pos + 3 > out.len) { return 0; }
                out[out_pos] = 0xFF;
                out[out_pos + 1] = run_len as u8;
                out[out_pos + 2] = b;
                out_pos += 3;
                in_pos += run_len;
            } else {
                if (out_pos + 1 > out.len) { return 0; }
                out[out_pos] = b;
                out_pos += 1;
                in_pos += 1;
            }
        }
        return out_pos;
    }

    pub static fn decompress(src: []const u8, out: []u8): usize {
        let in_pos: usize = 0;
        let out_pos: usize = 0;
        while (in_pos < src.len) {
            let b = src[in_pos];
            if (b == 0xFF) {
                if (in_pos + 3 > src.len) { return 0; }
                let count = src[in_pos + 1] as usize;
                let val = src[in_pos + 2];
                if (out_pos + count > out.len) { return 0; }
                let k: usize = 0;
                while (k < count) {
                    out[out_pos + k] = val;
                    k += 1;
                }
                out_pos += count;
                in_pos += 3;
            } else {
                if (out_pos + 1 > out.len) { return 0; }
                out[out_pos] = b;
                out_pos += 1;
                in_pos += 1;
            }
        }
        return out_pos;
    }
}
