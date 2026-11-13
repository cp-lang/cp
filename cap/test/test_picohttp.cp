import { test, expect_eq } from "cap:test";
import {
    phr_header,
    ChunkedEncodingState,
    phr_chunked_decoder,
    PicoHeader,
    PicoRequest,
    PicoResponse,
    PicoHttpParser,
} from "cap:http";

fn main(): void {
    test("1. PicoHttpParser.parseRequest zero-copy HTTP/1.1 request wire parsing", () => {
        const raw_req = "POST /api/v1/agents?limit=10 HTTP/1.1\r\nHost: localhost:8080\r\nContent-Type: application/json\r\nX-Agent-Id: agy-007\r\nContent-Length: 15\r\n\r\n{\"hello\":\"cap\"}";
        let headers: [8]PicoHeader = [PicoHeader{}; 8];

        if (req := PicoHttpParser.parseRequest(raw_req, headers[0..8])) {
            expect_eq(req.method, "POST");
            expect_eq(req.path, "/api/v1/agents?limit=10");
            expect_eq(req.minor_version, 1);
            expect_eq(req.header_count, 4);
            expect_eq(headers[0].name, "Host");
            expect_eq(headers[0].value, "localhost:8080");

            if (ct := PicoHttpParser.findHeader(headers[0..8], req.header_count, "content-type")) {
                expect_eq(ct, "application/json");
            } else {
                expect_eq(true, false);
            }

            if (agent := PicoHttpParser.findHeader(headers[0..8], req.header_count, "x-agent-id")) {
                expect_eq(agent, "agy-007");
            } else {
                expect_eq(true, false);
            }

            const body = raw_req[req.bytes_read .. raw_req.len];
            expect_eq(body, "{\"hello\":\"cap\"}");
        } else {
            expect_eq(true, false);
        }

        // Incomplete request should return null
        const partial = "GET /index.html HTTP/1.1\r\nHost: example.com\r\n";
        expect_eq(PicoHttpParser.parseRequest(partial, headers[0..8]) == null, true);
    });

    test("2. PicoHttpParser.parseResponse zero-copy HTTP/1.1 response wire parsing", () => {
        const raw_resp = "HTTP/1.1 201 Created\r\nServer: cap-http/1.0\r\nContent-Length: 2\r\n\r\nOK";
        let headers: [8]PicoHeader = [PicoHeader{}; 8];

        if (resp := PicoHttpParser.parseResponse(raw_resp, headers[0..8])) {
            expect_eq(resp.minor_version, 1);
            expect_eq(resp.status, 201);
            expect_eq(resp.message, "Created");
            expect_eq(resp.header_count, 2);

            if (srv := PicoHttpParser.findHeader(headers[0..8], resp.header_count, "SERVER")) {
                expect_eq(srv, "cap-http/1.0");
            } else {
                expect_eq(true, false);
            }

            const body = raw_resp[resp.bytes_read .. raw_resp.len];
            expect_eq(body, "OK");
        } else {
            expect_eq(true, false);
        }
    });

    test("3. PicoHTTPParser C-ABI struct & enum layout verification", () => {
        const decoder = phr_chunked_decoder.init(
            128,
            1,
            ChunkedEncodingState.CHUNKED_IN_CHUNK_DATA
        );
        expect_eq(decoder.bytes_left_in_chunk, 128);
        expect_eq(decoder.consume_trailer, 1);
        expect_eq(decoder._state, 2);
        expect_eq(@sizeOf(phr_header), 32);
    });
}
