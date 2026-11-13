// lib/dns/index.cp
// Unified Cap DNS Resolution, Wire Codec & c-ares Subsystem (cap:dns)

export {
    ares_channeldata,
    ares_addrinfo,
    ares_options,
    ARES_SUCCESS,
    ARES_ENODATA,
    ARES_EFORMERR,
    ARES_ESERVFAIL,
    ARES_ENOTFOUND,
    ARES_ENOTIMP,
    ARES_EREFUSED,
    ARES_EBADQUERY,
    ARES_EBADNAME,
    ARES_EBADFAMILY,
    ARES_EBADRESP,
    ARES_ECONNREFUSED,
    ARES_ETIMEOUT,
    ARES_EOF,
    ARES_EFILE,
    ARES_ENOMEM,
    ARES_EDESTRUCTION,
    ARES_EBADSTR,
    ARES_EBADFLAGS,
    ARES_ENONAME,
    ARES_EBADHINTS,
    ARES_ENOTINITIALIZED,
    ARES_ECANCELLED,
    ARES_ESERVICE,
    ARES_ENOSERVER,
    NSClass,
    NSType,
    CaresStatus,
    ares_library_init,
    ares_library_cleanup,
    ares_init,
    ares_destroy,
    ares_cancel,
    ares_strerror,
    ares_free_data,
    ares_freeaddrinfo,
} from "./_cares.cp";

export {
    DnsRCode,
    DnsHeader,
    DecodedDomain,
    DnsWireCodec,
} from "./packet.cp";

export {
    DnsFamily,
    DnsBackend,
    ResolveOptions,
    DnsCacheEntry,
    DnsCache,
} from "./cache.cp";
