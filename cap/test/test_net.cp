// cap/test/test_net.cp
// Comprehensive end-to-end test suite for cap:net (lib/net/)

import { test, expect_eq } from "cap:test";
import {
    LIBUS_TIMEOUT_GRANULARITY,
    LIBUS_RECV_BUFFER_PADDING,
    LIBUS_LISTEN_REUSE_PORT,
    LIBUS_LISTEN_REUSE_ADDR,
    LIBUS_LISTEN_EXCLUSIVE_PORT,
    SocketCloseCode,
    SocketKind,
    SendStatus,
    AddressFamily,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddress,
    SocketState,
    SafeSocket,
    STREAM_CORK_BUFFER_SIZE,
    StreamBuffer,
    TcpStream,
    ListenConfig,
    TcpServer,
    UdpSocket,
    listen
} from "cap:net";

fn main(): void {
    test("1. Backward-compatible listen() and Server API", () => {
        let server = listen(18080);
        server.close();
    });

    test("2. Ipv4Addr, Ipv6Addr & SocketAddress parsing and inspection", () => {
        const lo4 = Ipv4Addr.loopback();
        expect_eq(lo4.isLoopback(), true);
        expect_eq(lo4.isPrivate(), false);
        expect_eq(lo4.toU32(), 0x7F000001);

        const roundtrip = Ipv4Addr.fromU32(0xC0A8010A);
        expect_eq(roundtrip.octets[0] as i64, 192);
        expect_eq(roundtrip.octets[1] as i64, 168);
        expect_eq(roundtrip.octets[2] as i64, 1);
        expect_eq(roundtrip.octets[3] as i64, 10);
        expect_eq(roundtrip.isPrivate(), true);

        if (parsed := Ipv4Addr.parse("10.20.30.40")) {
            expect_eq(parsed.isPrivate(), true);
            expect_eq(parsed.octets[2] as i64, 30);
        } else {
            expect_eq(1, 0);
        }

        const mcast = Ipv4Addr.init(224, 0, 0, 251);
        expect_eq(mcast.isMulticast(), true);

        const lo6 = Ipv6Addr.loopback();
        expect_eq(lo6.isLoopback(), true);
        expect_eq(lo6.isUnspecified(), false);

        const mapped6 = Ipv6Addr.fromIpv4Mapped(lo4);
        expect_eq(mapped6.isIpv4Mapped(), true);

        const sa4 = SocketAddress.loopbackV4(3000);
        expect_eq(sa4.isIPv6(), false);
        expect_eq(sa4.port as i64, 3000);
        expect_eq(sa4.socklen() as i64, 16);

        const sa6 = SocketAddress.loopbackV6(8443);
        expect_eq(sa6.isIPv6(), true);
        expect_eq(sa6.port as i64, 8443);
        expect_eq(sa6.socklen() as i64, 28);
    });

    test("3. SafeSocket options, >240s long-timeout splitting & RAII lifecycle", () => {
        expect_eq(LIBUS_TIMEOUT_GRANULARITY as i64, 4);
        expect_eq(LIBUS_RECV_BUFFER_PADDING as i64, 32);

        let sock = SafeSocket.create(42, SocketKind.established_tcp);
        expect_eq(sock.isEstablished(), true);
        expect_eq(sock.isClosed(), false);

        sock.setNoDelay(true);
        expect_eq(sock.nodelay, true);

        const ka_ok = sock.setKeepAlive(true, 60);
        expect_eq(ka_ok, true);
        expect_eq(sock.keepalive_delay as i64, 60);

        // Short timeout <= 240s
        sock.setTimeout(120);
        expect_eq(sock.timeout_sec as i64, 120);
        expect_eq(sock.long_timeout_min as i64, 0);

        // Long timeout > 240s automatically promotes to minute-level long_timeout
        sock.setTimeout(360);
        expect_eq(sock.timeout_sec as i64, 0);
        expect_eq(sock.long_timeout_min as i64, 6);

        sock.pause();
        expect_eq(sock.paused, true);
        sock.resumeSocket();
        expect_eq(sock.paused, false);

        const status = sock.recordWrite(512);
        expect_eq(status == SendStatus.success, true);
        sock.recordRead(256);
        expect_eq(sock.stats.bytes_written as i64, 512);
        expect_eq(sock.stats.bytes_read as i64, 256);

        sock.shutdown();
        expect_eq(sock.write_shutdown, true);
        expect_eq(sock.recordWrite(64) == SendStatus.dropped, true);

        sock.close(SocketCloseCode.normal);
        expect_eq(sock.isClosed(), true);
        sock.release();
    });

    test("4. StreamBuffer & TcpStream corking / uncorking", () => {
        expect_eq(STREAM_CORK_BUFFER_SIZE as i64, 16384);

        const local_addr = SocketAddress.loopbackV4(9000);
        const remote_addr = SocketAddress.loopbackV4(51234);
        let stream = TcpStream.init(77, local_addr, remote_addr);

        stream.cork();
        expect_eq(stream.corked, true);

        const w1 = stream.write("HTTP/1.1 200 OK\r\n");
        const w2 = stream.write("Content-Length: 5\r\n\r\nhello");
        expect_eq(w1 as i64, 17);
        expect_eq(w2 as i64, 26);
        expect_eq(stream.send_buf.pendingLen() as i64, 43);
        expect_eq(stream.sock.stats.bytes_written as i64, 0);

        const flushed = stream.uncork();
        expect_eq(flushed as i64, 43);
        expect_eq(stream.corked, false);
        expect_eq(stream.send_buf.pendingLen() as i64, 0);
        expect_eq(stream.sock.stats.bytes_written as i64, 43);

        const fed = stream.feedIncoming("PING\r\n");
        expect_eq(fed as i64, 6);
        expect_eq(stream.recv_len as i64, 6);

        var out: [4]u8 = [0, 0, 0, 0];
        const read_n = stream.readAvailable(out[0..4]);
        expect_eq(read_n as i64, 4);
        expect_eq(stream.recv_len as i64, 2);

        stream.release();
    });

    test("5. ListenConfig bitmask & TcpServer connection lifecycle", () => {
        var cfg = ListenConfig.defaultForPort(8088);
        expect_eq(cfg.toLibusOptions() as i64, (LIBUS_LISTEN_REUSE_PORT | LIBUS_LISTEN_REUSE_ADDR) as i64);

        cfg.exclusive_port = true;
        expect_eq(
            cfg.toLibusOptions() as i64,
            (LIBUS_LISTEN_REUSE_PORT | LIBUS_LISTEN_REUSE_ADDR | LIBUS_LISTEN_EXCLUSIVE_PORT) as i64
        );

        const bind_addr = SocketAddress.loopbackV4(8088);
        let srv = TcpServer.bind(bind_addr, cfg);
        expect_eq(srv.is_listening, true);
        expect_eq(srv.localPort() as i64, 8088);

        const client_peer = SocketAddress.loopbackV4(60001);
        if (client_stream := srv.acceptConnection(client_peer)) {
            expect_eq(srv.active_connections as i64, 1);
            expect_eq(srv.total_accepted as i64, 1);
            let * = client_stream.write("welcome");
            expect_eq(client_stream.sock.stats.bytes_written as i64, 7);
            client_stream.release();
            srv.onClientClosed();
            expect_eq(srv.active_connections as i64, 0);
        } else {
            expect_eq(1, 0);
        }

        srv.release();
    });

    test("6. UdpSocket bind, peer connect, TTL/multicast options & datagrams", () => {
        const local_udp = SocketAddress.loopbackV4(5353);
        let udp = UdpSocket.bind(local_udp);
        expect_eq(udp.boundPort() as i64, 5353);
        expect_eq(udp.setBroadcast(true), true);
        expect_eq(udp.setUnicastTTL(128), true);
        expect_eq(udp.setMulticastTTL(4), true);
        expect_eq(udp.setMulticastLoopback(false), true);

        const peer = SocketAddress.loopbackV4(5354);
        expect_eq(udp.connectPeer(peer), true);

        const sent = udp.send("dns-query");
        expect_eq(sent as i64, 9);
        expect_eq(udp.packets_sent as i64, 1);
        expect_eq(udp.bytes_sent as i64, 9);

        const pkt = udp.simulateReceive(32, peer);
        expect_eq(pkt.payload_len as i64, 32);
        expect_eq(pkt.truncated, false);
        expect_eq(udp.packets_received as i64, 1);

        udp.disconnectPeer();
        expect_eq(udp.has_connected_peer, false);
        udp.release();
    });
}
