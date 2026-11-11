//
// Wyhash port to CP language for Bun.js
// Based on Wyhash from the zig standard library, version v0.11.0-dev.2609+5e19250a1
//

const primes: [u64; 5] = [
    0xa0761d6478bd642f as u64,
    0xe7037ed1a0b428db as u64,
    0x8ebc6af09c88c6e3 as u64,
    0x589965cc75374cc3 as u64,
    0x1d8e4e27c47d124f as u64,
];

fn read_bytes1(data: []const u8): u64 {
    return data[0] as u64;
}

fn read_bytes2(data: []const u8): u64 {
    return (data[0] as u64) | ((data[1] as u64) << 8);
}

fn read_bytes4(data: []const u8): u64 {
    return (data[0] as u64)
        | ((data[1] as u64) << 8)
        | ((data[2] as u64) << 16)
        | ((data[3] as u64) << 24);
}

fn read_bytes8(data: []const u8): u64 {
    return (data[0] as u64)
        | ((data[1] as u64) << 8)
        | ((data[2] as u64) << 16)
        | ((data[3] as u64) << 24)
        | ((data[4] as u64) << 32)
        | ((data[5] as u64) << 40)
        | ((data[6] as u64) << 48)
        | ((data[7] as u64) << 56);
}

fn read_8bytes_swapped(data: []const u8): u64 {
    return (read_bytes4(data) << 32) | read_bytes4(data[4..]);
}

fn mum(a: u64, b: u64): u64 {
    let r: u128 = (a as u128) * (b as u128);
    r = (r >> 64) ^ r;
    return r as u64;
}

fn mix0(a: u64, b: u64, seed: u64): u64 {
    return mum(a ^ seed ^ primes[0], b ^ seed ^ primes[1]);
}

fn mix1(a: u64, b: u64, seed: u64): u64 {
    return mum(a ^ seed ^ primes[2], b ^ seed ^ primes[3]);
}

// Wyhash version which does not store internal state for handling partial buffers.
// This is needed so that we can maximize the speed for the short key case, which will
// use the non-iterative api which the public Wyhash exposes.
export struct WyhashStateless {
    seed: u64,
    msg_len: usize,

    pub static fn init(seed: u64): WyhashStateless {
        return WyhashStateless {
            seed: seed,
            msg_len: 0,
        };
    }

    pub fn round(this: *WyhashStateless, b: []const u8): void {
        this.seed = mix0(
            read_bytes8(b[0..]),
            read_bytes8(b[8..]),
            this.seed,
        ) ^ mix1(
            read_bytes8(b[16..]),
            read_bytes8(b[24..]),
            this.seed,
        );
    }

    pub fn update(this: *WyhashStateless, b: []const u8): void {
        var off: usize = 0;
        while (off + 32 <= b.len) {
            this.round(b[off..off + 32]);
            off += 32;
        }
        this.msg_len += b.len;
    }

    pub fn final(this: *WyhashStateless, b: []const u8): u64 {
        const s = this.seed;
        const rem_len = b.len;
        const rem_key = b[0..rem_len];

        this.seed = switch (rem_len) {
            0 => s,
            1 => mix0(read_bytes1(rem_key), primes[4], s),
            2 => mix0(read_bytes2(rem_key), primes[4], s),
            3 => mix0((read_bytes2(rem_key) << 8) | read_bytes1(rem_key[2..]), primes[4], s),
            4 => mix0(read_bytes4(rem_key), primes[4], s),
            5 => mix0((read_bytes4(rem_key) << 8) | read_bytes1(rem_key[4..]), primes[4], s),
            6 => mix0((read_bytes4(rem_key) << 16) | read_bytes2(rem_key[4..]), primes[4], s),
            7 => mix0((read_bytes4(rem_key) << 24) | (read_bytes2(rem_key[4..]) << 8) | read_bytes1(rem_key[6..]), primes[4], s),
            8 => mix0(read_8bytes_swapped(rem_key), primes[4], s),
            9 => mix0(read_8bytes_swapped(rem_key), read_bytes1(rem_key[8..]), s),
            10 => mix0(read_8bytes_swapped(rem_key), read_bytes2(rem_key[8..]), s),
            11 => mix0(read_8bytes_swapped(rem_key), (read_bytes2(rem_key[8..]) << 8) | read_bytes1(rem_key[10..]), s),
            12 => mix0(read_8bytes_swapped(rem_key), read_bytes4(rem_key[8..]), s),
            13 => mix0(read_8bytes_swapped(rem_key), (read_bytes4(rem_key[8..]) << 8) | read_bytes1(rem_key[12..]), s),
            14 => mix0(read_8bytes_swapped(rem_key), (read_bytes4(rem_key[8..]) << 16) | read_bytes2(rem_key[12..]), s),
            15 => mix0(read_8bytes_swapped(rem_key), (read_bytes4(rem_key[8..]) << 24) | (read_bytes2(rem_key[12..]) << 8) | read_bytes1(rem_key[14..]), s),
            16 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s),
            17 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_bytes1(rem_key[16..]), primes[4], s),
            18 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_bytes2(rem_key[16..]), primes[4], s),
            19 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1((read_bytes2(rem_key[16..]) << 8) | read_bytes1(rem_key[18..]), primes[4], s),
            20 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_bytes4(rem_key[16..]), primes[4], s),
            21 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1((read_bytes4(rem_key[16..]) << 8) | read_bytes1(rem_key[20..]), primes[4], s),
            22 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1((read_bytes4(rem_key[16..]) << 16) | read_bytes2(rem_key[20..]), primes[4], s),
            23 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1((read_bytes4(rem_key[16..]) << 24) | (read_bytes2(rem_key[20..]) << 8) | read_bytes1(rem_key[22..]), primes[4], s),
            24 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), primes[4], s),
            25 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), read_bytes1(rem_key[24..]), s),
            26 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), read_bytes2(rem_key[24..]), s),
            27 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), (read_bytes2(rem_key[24..]) << 8) | read_bytes1(rem_key[26..]), s),
            28 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), read_bytes4(rem_key[24..]), s),
            29 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), (read_bytes4(rem_key[24..]) << 8) | read_bytes1(rem_key[28..]), s),
            30 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), (read_bytes4(rem_key[24..]) << 16) | read_bytes2(rem_key[28..]), s),
            31 => mix0(read_8bytes_swapped(rem_key), read_8bytes_swapped(rem_key[8..]), s) ^ mix1(read_8bytes_swapped(rem_key[16..]), (read_bytes4(rem_key[24..]) << 24) | (read_bytes2(rem_key[28..]) << 8) | read_bytes1(rem_key[30..]), s),
            else => s,
        };

        this.msg_len += b.len;
        return mum(this.seed ^ (this.msg_len as u64), primes[4]);
    }

    pub static fn hash(seed: u64, input: []const u8): u64 {
        const aligned_len = input.len - (input.len % 32);
        var c = WyhashStateless.init(seed);
        c.update(input[0..aligned_len]);
        return c.final(input[aligned_len..]);
    }
}

export struct Wyhash {
    state: WyhashStateless,
    buf: [32]u8,
    buf_len: usize,

    pub static fn init(seed: u64): Wyhash {
        return Wyhash {
            state: WyhashStateless.init(seed),
            buf: [0 as u8; 32],
            buf_len: 0,
        };
    }

    pub fn update(this: *Wyhash, b: []const u8): void {
        var off: usize = 0;

        if (this.buf_len != 0 and this.buf_len + b.len >= 32) {
            off += 32 - this.buf_len;
            var i: usize = 0;
            while (i < off) {
                this.buf[this.buf_len + i] = b[i];
                i += 1;
            }
            this.state.update(this.buf[0..32]);
            this.buf_len = 0;
        }

        const remain_len = b.len - off;
        const aligned_len = remain_len - (remain_len % 32);
        this.state.update(b[off..off + aligned_len]);

        const rem_slice = b[off + aligned_len..b.len];
        var j: usize = 0;
        while (j < rem_slice.len) {
            this.buf[this.buf_len + j] = rem_slice[j];
            j += 1;
        }
        this.buf_len += rem_slice.len;
    }

    pub fn final(this: *Wyhash): u64 {
        const rem_key = this.buf[0..this.buf_len];
        return this.state.final(rem_key);
    }

    pub static fn hash(seed: u64, input: []const u8): u64 {
        return WyhashStateless.hash(seed, input);
    }
}
