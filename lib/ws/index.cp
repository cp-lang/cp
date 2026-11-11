// lib/ws/index.cp
import { Opcode, CloseCode } from "./opcode";
import { Mask } from "./mask";
import { FrameHeader, FrameBuilder } from "./frame";

pub const ws = struct {
    pub const buildText = FrameBuilder.buildText;
    pub const buildBinary = FrameBuilder.buildBinary;
    pub const buildClose = FrameBuilder.buildClose;
    pub const buildPing = FrameBuilder.buildPing;
    pub const buildPong = FrameBuilder.buildPong;
    pub const applyMask = Mask.apply;
};
