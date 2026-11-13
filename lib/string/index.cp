// lib/string/index.cp
// Unified Cap String & Unicode SIMD Subsystem (cap:string)

export {
    SimdScanner,
} from "./simd_scan.cp";

export {
    REPLACEMENT_CHAR,
    DecodeResult,
    Utf8,
    Utf16,
    UnicodeWidth,
} from "./utf.cp";

export {
    hashBytes32,
    HashedString,
    SMOL_MAX_INLINE,
    SmolStr,
    StringBuilder,
} from "./builder.cp";

export {
    StringEscape,
} from "./escape.cp";
