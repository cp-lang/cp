// lib/net/index.cp
// Unified Public Exports for cap:net
// Architecture:
//   - _usockets.cp  : Low-level @c uSockets FFI bindings, opaque handles, and constants
//   - address.cp    : IPv4 / IPv6 SocketAddress parsing & manipulation
//   - socket.cp     : SafeSocket with RAII, socket options, and legacy Socket compatibility
//   - tcp_stream.cp : High-throughput TCP Stream with 16KB StreamBuffer corking
//   - tcp_server.cp : TcpServer, ListenConfig, and legacy Server / listen() API
//   - udp.cp        : UdpSocket and UdpPacket datagram abstraction

export {
    LIBUS_TIMEOUT_GRANULARITY,
    LIBUS_RECV_BUFFER_PADDING,
    LIBUS_EXT_ALIGNMENT,
    LIBUS_LISTEN_DEFAULT,
    LIBUS_LISTEN_EXCLUSIVE_PORT,
    LIBUS_SOCKET_ALLOW_HALF_OPEN,
    LIBUS_LISTEN_REUSE_PORT,
    LIBUS_SOCKET_IPV6_ONLY,
    LIBUS_LISTEN_REUSE_ADDR,
    LIBUS_LISTEN_DISALLOW_REUSE_PORT_FAILURE,
    SocketCloseCode,
    SocketKind,
    SendStatus,
    us_loop_t,
    us_socket_t,
    us_socket_context_t,
    us_listen_socket_t,
    us_connecting_socket_t,
    us_udp_socket_t,
    us_udp_packet_buffer_t,
    us_timer_t,
    UsVerifyError
} from "./_usockets";

export {
    AddressFamily,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddress
} from "./address";

export {
    SocketState,
    SocketStats,
    SafeSocket,
    Socket
} from "./socket";

export {
    STREAM_CORK_BUFFER_SIZE,
    StreamBuffer,
    TcpStream
} from "./tcp_stream";

export {
    ListenConfig,
    TcpServer,
    Server,
    listen,
    connect
} from "./tcp_server";

export {
    UdpPacket,
    UdpSocket
} from "./udp";
