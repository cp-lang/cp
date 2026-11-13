// ============================================================================
// lib/compress/checksum.cp — IEEE 802.3 CRC32 与 RFC 1950 Adler32 校验和引擎
// 用于 Gzip / Zlib / ZIP 帧完整性校验与流式增量哈希
// ============================================================================

/// IEEE 802.3 标准多项式 (0xEDB88320) CRC-32 增量与单次校验器
export struct Crc32 {
    state: u32 = 0xFFFFFFFF,

    pub static fn init(): Crc32 {
        return Crc32{ state: 0xFFFFFFFF };
    }

    pub fn update(*this, data: []const u8): void {
        let crc = this.state;
        let i: usize = 0;
        while (i < data.len) {
            crc = crc ^ (data[i] as u32);
            let bit: usize = 0;
            while (bit < 8) {
                let mask: u32 = if ((crc & 1) != 0) (0xEDB88320 as u32) else (0 as u32);
                crc = (crc >> 1) ^ mask;
                bit += 1;
            }
            i += 1;
        }
        this.state = crc;
    }

    pub fn final(&this): u32 {
        return this.state ^ 0xFFFFFFFF;
    }

    pub fn reset(*this): void {
        this.state = 0xFFFFFFFF;
    }

    /// 单次计算字节切片的 IEEE CRC-32 值
    pub static fn checksum(data: []const u8): u32 {
        let hasher = Crc32.init();
        hasher.update(data);
        return hasher.final();
    }
}

/// RFC 1950 Zlib 标准 Adler-32 (MOD_ADLER = 65521) 增量与单次校验器
export struct Adler32 {
    s1: u32 = 1,
    s2: u32 = 0,

    pub static fn init(): Adler32 {
        return Adler32{ s1: 1, s2: 0 };
    }

    pub fn update(*this, data: []const u8): void {
        let a = this.s1;
        let b = this.s2;
        let i: usize = 0;
        while (i < data.len) {
            a = (a + (data[i] as u32)) % 65521;
            b = (b + a) % 65521;
            i += 1;
        }
        this.s1 = a;
        this.s2 = b;
    }

    pub fn final(&this): u32 {
        return (this.s2 << 16) | this.s1;
    }

    pub fn reset(*this): void {
        this.s1 = 1;
        this.s2 = 0;
    }

    /// 单次计算字节切片的 RFC 1950 Adler-32 值
    pub static fn checksum(data: []const u8): u32 {
        let hasher = Adler32.init();
        hasher.update(data);
        return hasher.final();
    }
}
