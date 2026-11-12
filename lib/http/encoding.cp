/**
 * CP HTTP Content & Transfer Encoding (cap:http)
 * Ported from Bun.js http_types/Encoding.cp
 */

export enum Encoding {
    Identity,
    Gzip,
    Deflate,
    Brotli,
    Zstd,
    Chunked,

    pub fn canUseLibDeflate(self: Encoding) bool {
        return switch (self) {
            .Gzip, .Deflate => true,
            else => false,
        };
    }

    pub fn isCompressed(self: Encoding) bool {
        return switch (self) {
            .Gzip, .Deflate, .Brotli, .Zstd => true,
            else => false,
        };
    }
}
