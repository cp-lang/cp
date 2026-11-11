// lib/ws/opcode.cp
// RFC 6455 WebSocket Opcodes & Close Status Codes in CP

pub const Opcode = enum(u4) {
    Continue = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,

    pub fn isControl(self: Opcode): bool {
        return (@intFromEnum(self) & 0x8) != 0;
    }

    pub fn isData(self: Opcode): bool {
        return self == .Text or self == .Binary or self == .Continue;
    }
};

pub const CloseCode = enum(u16) {
    NormalClosure = 1000,
    GoingAway = 1001,
    ProtocolError = 1002,
    UnsupportedData = 1003,
    NoStatusReceived = 1005,
    AbnormalClosure = 1006,
    InvalidFramePayload = 1007,
    PolicyViolation = 1008,
    MessageTooBig = 1009,
    MissingExtension = 1010,
    InternalError = 1011,
};
