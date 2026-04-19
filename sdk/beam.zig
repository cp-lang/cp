const std = @import("std");

// --- 内部：引入 Erlang NIF C API 的纯 Zig 翻译版本 ---
// 注意：通过直接引入纯 Zig 版本，CP 编译器在编译业务代码时
// 完全不需要依赖任何 C 语言头文件 (erl_nif.h) 和 C 编译器环境！
const erl = @import("../src/erl_nif.zig");

// --- 公共 API 类型别名 ---
pub const Env = opaque {};
pub const Term = usize; 

pub const Pid = extern struct {
    pid: [erl.ERL_NIF_PID_SIZE]u8,
};

pub const BeamError = error{
    AllocationFailed,
    InvalidType,
    MessageSendFailed,
    MapPutFailed,
};

pub const Context = extern struct {
    state_id: u32,
    memory_len: usize,
};

// --- BEAM 虚拟机核心能力封装 ---
pub const Beam = struct {
    
    // ==========================================
    // 1. Atoms & Strings
    // ==========================================
    pub fn makeAtom(env: *Env, name: []const u8) Term {
        return erl.enif_make_atom_len(@ptrCast(env), name.ptr, name.len);
    }
    
    pub fn makeString(env: *Env, str: []const u8) Term {
        return erl.enif_make_string_len(@ptrCast(env), str.ptr, str.len, erl.ERL_NIF_LATIN1);
    }

    // ==========================================
    // 2. Numbers (Integer & Float)
    // ==========================================
    pub fn makeInt(env: *Env, val: i32) Term {
        return erl.enif_make_int(@ptrCast(env), val);
    }
    
    pub fn makeInt64(env: *Env, val: i64) Term {
        return erl.enif_make_int64(@ptrCast(env), val);
    }
    
    pub fn makeUint64(env: *Env, val: u64) Term {
        return erl.enif_make_uint64(@ptrCast(env), val);
    }
    
    pub fn makeDouble(env: *Env, val: f64) Term {
        return erl.enif_make_double(@ptrCast(env), val);
    }

    pub fn getInt(env: *Env, term: Term) !i32 {
        var val: c_int = 0;
        if (erl.enif_get_int(@ptrCast(env), term, &val) == 0) return BeamError.InvalidType;
        return @intCast(val);
    }

    pub fn getInt64(env: *Env, term: Term) !i64 {
        var val: i64 = 0;
        if (erl.enif_get_int64(@ptrCast(env), term, &val) == 0) return BeamError.InvalidType;
        return val;
    }

    pub fn getUint64(env: *Env, term: Term) !u64 {
        var val: u64 = 0;
        if (erl.enif_get_uint64(@ptrCast(env), term, &val) == 0) return BeamError.InvalidType;
        return val;
    }

    pub fn getDouble(env: *Env, term: Term) !f64 {
        var val: f64 = 0;
        if (erl.enif_get_double(@ptrCast(env), term, &val) == 0) return BeamError.InvalidType;
        return val;
    }

    // ==========================================
    // 3. Binaries (Zero-Copy Data)
    // ==========================================
    pub fn getBinary(env: *Env, term: Term) ![]const u8 {
        var bin: erl.ErlNifBinary = undefined;
        if (erl.enif_inspect_binary(@ptrCast(env), term, &bin) == 0) {
            return BeamError.InvalidType;
        }
        return bin.data[0..bin.size];
    }

    pub fn makeBinary(env: *Env, data: []const u8) !Term {
        var term: Term = undefined;
        const ptr = erl.enif_make_new_binary(@ptrCast(env), data.len, &term);
        if (ptr == null) return BeamError.AllocationFailed;
        @memcpy(ptr[0..data.len], data);
        return term;
    }

    // ==========================================
    // 4. Tuples & Lists
    // ==========================================
    pub fn makeTuple(env: *Env, terms: []const Term) Term {
        return erl.enif_make_tuple_from_array(@ptrCast(env), terms.ptr, @intCast(terms.len));
    }

    pub fn getTuple(env: *Env, term: Term) ![]const Term {
        var arity: c_int = 0;
        var array: [*c]const Term = undefined;
        if (erl.enif_get_tuple(@ptrCast(env), term, &arity, &array) == 0) return BeamError.InvalidType;
        return array[0..@intCast(arity)];
    }

    pub fn makeList(env: *Env, terms: []const Term) Term {
        return erl.enif_make_list_from_array(@ptrCast(env), terms.ptr, @intCast(terms.len));
    }

    pub fn makeListCell(env: *Env, head: Term, tail: Term) Term {
        return erl.enif_make_list_cell(@ptrCast(env), head, tail);
    }

    pub fn getListCell(env: *Env, term: Term) !struct { head: Term, tail: Term } {
        var head: Term = undefined;
        var tail: Term = undefined;
        if (erl.enif_get_list_cell(@ptrCast(env), term, &head, &tail) == 0) return BeamError.InvalidType;
        return .{ .head = head, .tail = tail };
    }

    pub fn isListEmpty(env: *Env, term: Term) bool {
        return erl.enif_is_empty_list(@ptrCast(env), term) != 0;
    }

    // ==========================================
    // 5. Maps
    // ==========================================
    pub fn makeMap(env: *Env) Term {
        return erl.enif_make_new_map(@ptrCast(env));
    }

    pub fn putMap(env: *Env, map_term: Term, key: Term, value: Term) !Term {
        var new_map: Term = undefined;
        if (erl.enif_make_map_put(@ptrCast(env), map_term, key, value, &new_map) == 0) {
            return BeamError.MapPutFailed;
        }
        return new_map;
    }

    pub fn getMapValue(env: *Env, map_term: Term, key: Term) !Term {
        var value: Term = undefined;
        if (erl.enif_get_map_value(@ptrCast(env), map_term, key, &value) == 0) {
            return BeamError.InvalidType;
        }
        return value;
    }

    // ==========================================
    // 6. Processes & PIDs
    // ==========================================
    pub fn getSelf(env: *Env, pid: *Pid) !void {
        if (erl.enif_self(@ptrCast(env), @ptrCast(pid)) == null) return BeamError.InvalidType;
    }

    pub fn getLocalPid(env: *Env, term: Term, pid: *Pid) !void {
        if (erl.enif_get_local_pid(@ptrCast(env), term, @ptrCast(pid)) == 0) return BeamError.InvalidType;
    }

    pub fn makePid(env: *Env, pid: *const Pid) Term {
        return erl.enif_make_pid(@ptrCast(env), @ptrCast(pid));
    }

    // ==========================================
    // 7. Messaging & Routing
    // ==========================================
    pub fn makeFastMessage(env: *Env, target_numeric_id: u32, action_id: u32, payload: []const u8) !Term {
        var bin_term: Term = undefined;
        const total_len = 8 + payload.len;
        const buf = erl.enif_make_new_binary(@ptrCast(env), total_len, &bin_term);
        if (buf == null) return BeamError.AllocationFailed;
        
        std.mem.writeInt(u32, buf[0..4], target_numeric_id, .little);
        std.mem.writeInt(u32, buf[4..8], action_id, .little);
        
        if (payload.len > 0) @memcpy(buf[8..total_len], payload);
        
        const atom_js_msg_fast = Beam.makeAtom(env, "js_msg_fast");
        return makeTuple(env, &[_]Term{ atom_js_msg_fast, bin_term });
    }

    pub fn send(env: *Env, to_pid: *Pid, msg: Term) !void {
        if (erl.enif_send(@ptrCast(env), @ptrCast(to_pid), @ptrCast(env), msg) == 0) {
            return BeamError.MessageSendFailed;
        }
    }

    // ==========================================
    // 8. Scheduling
    // ==========================================
    pub fn consumeTimeslice(env: *Env, percent: c_int) bool {
        return erl.enif_consume_timeslice(@ptrCast(env), percent) != 0;
    }
    // ==========================================
    // 9. Memory Management (NIF Safe Allocators)
    // ==========================================
    pub fn alloc(size: usize) ?*anyopaque {
        return erl.enif_alloc(size);
    }

    pub fn free(ptr: *anyopaque) void {
        erl.enif_free(ptr);
    }
};

// --- NIF 生命周期与初始化 (Load/Upgrade/Unload) ---
pub const NifFunc = erl.ErlNifFunc;
pub const NifEntry = erl.ErlNifEntry;

pub const ERL_NIF_MAJOR_VERSION = erl.ERL_NIF_MAJOR_VERSION;
pub const ERL_NIF_MINOR_VERSION = erl.ERL_NIF_MINOR_VERSION;

/// CP Native Agent 的标准系统行为动作枚举 (System Action)
/// 业务代码 (CP 语言) 可以自定义其他任何 u32 值作为业务 Action
pub const SystemAction = enum(u32) {
    NULL_ACTION = 0, // 防御性编程：预留 0 以捕获空指针或未初始化的内存错误
    WAKE = 1,
    SLEEP = 2,
    STOP = 3,

    // 网络 I/O (10-19)
    NET_CONNECT_REQ = 10,
    NET_CONNECT_RES = 11,
    NET_WRITE_REQ = 12,
    NET_READ_REQ = 13,
    NET_READ_RES = 14,

    // 文件 I/O (20-29)
    FS_READ_REQ = 20,
    FS_READ_RES = 21,
    FS_WRITE_REQ = 22,
    FS_WRITE_RES = 23,

    // 高层 HTTP (30-39)
    HTTP_FETCH_REQ = 30,
    HTTP_FETCH_RES = 31,

    // 系统级错误响应 (99)
    // Payload 约定：前 4 字节为 ErrorCode (参考 HTTP, 如 404, 403, 500)
    IO_ERROR = 99,
};

/// 暴露一个规范定义的统一 Native NIF 入口函数类型
pub const NativeDispatchFunc = fn (
    env: ?*erl.ErlNifEnv, 
    argc: c_int, 
    argv: [*c]const erl.ERL_NIF_TERM
) callconv(.C) erl.ERL_NIF_TERM;

/// 辅助 CP 编译器一键生成供 BEAM 加载的 NifEntry 结构
pub fn defineNifEntry(
    comptime name: []const u8,
    comptime funcs: []const NifFunc,
    comptime load: ?*const fn (?*erl.ErlNifEnv, [*c]?*anyopaque, erl.ERL_NIF_TERM) callconv(.C) c_int,
    comptime upgrade: ?*const fn (?*erl.ErlNifEnv, [*c]?*anyopaque, [*c]?*anyopaque, erl.ERL_NIF_TERM) callconv(.C) c_int,
    comptime unload: ?*const fn (?*erl.ErlNifEnv, ?*anyopaque) callconv(.C) void,
) NifEntry {
    return .{
        .major = ERL_NIF_MAJOR_VERSION,
        .minor = ERL_NIF_MINOR_VERSION,
        .name = name.ptr,
        .num_of_funcs = funcs.len,
        .funcs = @constCast(funcs.ptr),
        .load = load,
        .reload = null,
        .upgrade = upgrade,
        .unload = unload,
        .vm_variant = "beam.vanilla",
        .options = 1, // ERL_NIF_OPT_DELAY_HALT
        .sizeof_ErlNifResourceTypeInit = @sizeOf(erl.ErlNifResourceTypeInit),
        .min_erts = "erts-10.0",
    };
}