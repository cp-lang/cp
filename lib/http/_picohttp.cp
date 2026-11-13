// lib/http/_picohttp.cp
// PicoHTTPParser C-FFI declarations & zero-copy HTTP/1.x request/response wire parser
// Absorbed from bun/src/picohttp_sys/picohttpparser.cp & bun/src/picohttp/picohttp.cp

export @c struct phr_header {
    pub name: [*]const u8! = undefined;
    pub name_len: usize = 0;
    pub value: [*]const u8! = undefined;
    pub value_len: usize = 0;
}

export enum ChunkedEncodingState: u8 {
    CHUNKED_IN_CHUNK_SIZE = 0,
    CHUNKED_IN_CHUNK_EXT = 1,
    CHUNKED_IN_CHUNK_DATA = 2,
    CHUNKED_IN_CHUNK_CRLF = 3,
    CHUNKED_IN_TRAILERS_LINE_HEAD = 4,
    CHUNKED_IN_TRAILERS_LINE_MIDDLE = 5,
}

export @c struct phr_chunked_decoder {
    pub bytes_left_in_chunk: usize = 0;
    pub consume_trailer: u8 = 0;
    pub _hex_count: u8 = 0;
    pub _state: u8 = 0;

    pub static fn init(bytes_left: usize, consume_trailer: u8, state: ChunkedEncodingState): phr_chunked_decoder {
        const st_val = state as u8;
        zig {
            return phr_chunked_decoder{
                .bytes_left_in_chunk = bytes_left,
                .consume_trailer = consume_trailer,
                ._hex_count = 0,
                ._state = st_val,
            };
        }
    }
}

@c {
    pub fn phr_parse_request(
        buf: [*]const u8!,
        len: usize,
        method: *([*]const u8!),
        method_len: *usize!,
        path: *([*]const u8!),
        path_len: *usize!,
        minor_version: *i32!,
        headers: [*]phr_header!,
        num_headers: *usize!,
        last_len: usize
    ): i32;

    pub fn phr_parse_response(
        buf: [*]const u8!,
        len: usize,
        minor_version: *i32!,
        status: *i32!,
        msg: *([*]const u8!),
        msg_len: *usize!,
        headers: [*]phr_header!,
        num_headers: *usize!,
        last_len: usize
    ): i32;

    pub fn phr_parse_headers(
        buf: [*]const u8!,
        len: usize,
        headers: [*]phr_header!,
        num_headers: *usize!,
        last_len: usize
    ): i32;

    pub fn phr_decode_chunked(
        decoder: *phr_chunked_decoder!,
        buf: [*]u8!,
        bufsz: *usize!
    ): isize;

    pub fn phr_decode_chunked_is_in_data(
        decoder: *phr_chunked_decoder!
    ): i32;
}

export struct PicoHeader {
    pub name: []const u8 = "";
    pub value: []const u8 = "";

    pub fn isMultiline(this): bool {
        return this.name.len == 0;
    }
}

export struct PicoRequest {
    pub method: []const u8 = "";
    pub path: []const u8 = "";
    pub minor_version: i32 = 1;
    pub header_count: usize = 0;
    pub bytes_read: usize = 0;
}

export struct PicoResponse {
    pub minor_version: i32 = 1;
    pub status: i32 = 200;
    pub message: []const u8 = "";
    pub header_count: usize = 0;
    pub bytes_read: usize = 0;
}

fn trimSpaces(s: []const u8): []const u8 {
    let start: usize = 0;
    let end: usize = s.len;
    while (start < end && (s[start] == 32 || s[start] == 9)) {
        start += 1;
    }
    while (end > start && (s[end - 1] == 32 || s[end - 1] == 9)) {
        end -= 1;
    }
    return s[start..end];
}

fn eqlIgnoreCase(a: []const u8, b: []const u8): bool {
    if (a.len != b.len) {
        return false;
    }
    let i: usize = 0;
    while (i < a.len) {
        let ca = a[i];
        let cb = b[i];
        if (ca >= 65 && ca <= 90) {
            ca = ca + 32;
        }
        if (cb >= 65 && cb <= 90) {
            cb = cb + 32;
        }
        if (ca != cb) {
            return false;
        }
        i += 1;
    }
    return true;
}

// Zero-copy HTTP/1.x request & response header wire parser
export struct PicoHttpParser {
    // Returns PicoRequest on complete headers, or null if incomplete/malformed
    pub static fn parseRequest(buf: []const u8, out_headers: []PicoHeader): PicoRequest? {
        if (buf.len < 14) {
            return null;
        }
        // Find first CRLF
        let line_end: usize = 0;
        let found_line = false;
        while (line_end + 1 < buf.len) {
            if (buf[line_end] == 13 && buf[line_end + 1] == 10) {
                found_line = true;
                break;
            }
            line_end += 1;
        }
        if (!found_line) {
            return null;
        }

        const req_line = buf[0..line_end];
        let sp1: usize = 0;
        let found_sp1 = false;
        while (sp1 < req_line.len) {
            if (req_line[sp1] == 32) {
                found_sp1 = true;
                break;
            }
            sp1 += 1;
        }
        if (!found_sp1 || sp1 == 0) {
            return null;
        }

        let sp2: usize = sp1 + 1;
        let found_sp2 = false;
        while (sp2 < req_line.len) {
            if (req_line[sp2] == 32) {
                found_sp2 = true;
                break;
            }
            sp2 += 1;
        }
        if (!found_sp2 || sp2 == sp1 + 1) {
            return null;
        }

        const method = req_line[0..sp1];
        const path = req_line[sp1 + 1 .. sp2];
        const ver_str = req_line[sp2 + 1 .. req_line.len];
        if (ver_str.len != 8 || ver_str[0] != 72 || ver_str[1] != 84 || ver_str[2] != 84 || ver_str[3] != 80 || ver_str[4] != 47 || ver_str[5] != 49 || ver_str[6] != 46) {
            return null;
        }
        const minor_ch = ver_str[7];
        if (minor_ch != 48 && minor_ch != 49) {
            return null;
        }
        const minor_version: i32 = (minor_ch - 48) as i32;

        // Parse headers until empty CRLF
        let pos: usize = line_end + 2;
        let h_idx: usize = 0;
        while (pos + 1 < buf.len) {
            if (buf[pos] == 13 && buf[pos + 1] == 10) {
                // End of headers!
                return PicoRequest{
                    method: method,
                    path: path,
                    minor_version: minor_version,
                    header_count: h_idx,
                    bytes_read: pos + 2,
                };
            }
            let h_end = pos;
            let found_h_end = false;
            while (h_end + 1 < buf.len) {
                if (buf[h_end] == 13 && buf[h_end + 1] == 10) {
                    found_h_end = true;
                    break;
                }
                h_end += 1;
            }
            if (!found_h_end) {
                return null;
            }
            const h_line = buf[pos..h_end];
            let colon: usize = 0;
            let found_colon = false;
            while (colon < h_line.len) {
                if (h_line[colon] == 58) {
                    found_colon = true;
                    break;
                }
                colon += 1;
            }
            if (!found_colon || colon == 0) {
                return null;
            }
            if (h_idx < out_headers.len) {
                out_headers[h_idx] = PicoHeader{
                    name: trimSpaces(h_line[0..colon]),
                    value: trimSpaces(h_line[colon + 1 .. h_line.len]),
                };
                h_idx += 1;
            }
            pos = h_end + 2;
        }
        return null;
    }

    // Returns PicoResponse on complete headers, or null if incomplete/malformed
    pub static fn parseResponse(buf: []const u8, out_headers: []PicoHeader): PicoResponse? {
        if (buf.len < 14) {
            return null;
        }
        let line_end: usize = 0;
        let found_line = false;
        while (line_end + 1 < buf.len) {
            if (buf[line_end] == 13 && buf[line_end + 1] == 10) {
                found_line = true;
                break;
            }
            line_end += 1;
        }
        if (!found_line) {
            return null;
        }

        const status_line = buf[0..line_end];
        if (status_line.len < 12 || status_line[0] != 72 || status_line[1] != 84 || status_line[2] != 84 || status_line[3] != 80 || status_line[4] != 47 || status_line[5] != 49 || status_line[6] != 46) {
            return null;
        }
        const minor_ch = status_line[7];
        if (minor_ch != 48 && minor_ch != 49) {
            return null;
        }
        const minor_version: i32 = (minor_ch - 48) as i32;
        if (status_line[8] != 32) {
            return null;
        }
        let status: i32 = 0;
        let k: usize = 9;
        while (k < 12) {
            const d = status_line[k];
            if (d < 48 || d > 57) {
                return null;
            }
            status = status * 10 + ((d - 48) as i32);
            k += 1;
        }
        let msg: []const u8 = "";
        if (status_line.len > 13 && status_line[12] == 32) {
            msg = status_line[13..status_line.len];
        }

        let pos: usize = line_end + 2;
        let h_idx: usize = 0;
        while (pos + 1 < buf.len) {
            if (buf[pos] == 13 && buf[pos + 1] == 10) {
                return PicoResponse{
                    minor_version: minor_version,
                    status: status,
                    message: msg,
                    header_count: h_idx,
                    bytes_read: pos + 2,
                };
            }
            let h_end = pos;
            let found_h_end = false;
            while (h_end + 1 < buf.len) {
                if (buf[h_end] == 13 && buf[h_end + 1] == 10) {
                    found_h_end = true;
                    break;
                }
                h_end += 1;
            }
            if (!found_h_end) {
                return null;
            }
            const h_line = buf[pos..h_end];
            let colon: usize = 0;
            let found_colon = false;
            while (colon < h_line.len) {
                if (h_line[colon] == 58) {
                    found_colon = true;
                    break;
                }
                colon += 1;
            }
            if (!found_colon || colon == 0) {
                return null;
            }
            if (h_idx < out_headers.len) {
                out_headers[h_idx] = PicoHeader{
                    name: trimSpaces(h_line[0..colon]),
                    value: trimSpaces(h_line[colon + 1 .. h_line.len]),
                };
                h_idx += 1;
            }
            pos = h_end + 2;
        }
        return null;
    }

    pub static fn findHeader(headers: []const PicoHeader, count: usize, target_name: []const u8): ([]const u8)? {
        const limit = if (count < headers.len) count else headers.len;
        let i: usize = 0;
        while (i < limit) {
            if (eqlIgnoreCase(headers[i].name, target_name)) {
                return headers[i].value;
            }
            i += 1;
        }
        return null;
    }
}
