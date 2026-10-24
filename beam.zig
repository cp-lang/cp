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
    OpenFailed,
    PutFailed,
    GetFailed,
    ProcessDead,
};

/// Native Agent 的物理运行上下文
/// 这个结构体由底层编译器 in CP language 在生成状态机时使用。
pub const Context = extern struct {
    state_id: u32,
    memory_len: usize,
};

// --- BEAM 虚拟机核心能力封装 (SDK 全功能终极版本) ---
pub const Beam = struct {
    
    // ==========================================
    // 1. Atoms, Strings & Exceptions
    // ==========================================
    pub fn makeAtom(env: *Env, name: []const u8) Term {
        return erl.enif_make_atom_len(@ptrCast(env), name.ptr, name.len);
    }
    
    pub fn makeString(env: *Env, str: []const u8) Term {
        return erl.enif_make_string_len(@ptrCast(env), str.ptr, str.len, erl.ERL_NIF_LATIN1);
    }

    pub fn raiseException(env: *Env, reason: []const u8) Term {
        return erl.enif_raise_exception(@ptrCast(env), Beam.makeAtom(env, reason));
    }

    /// 构造标准的 Erlang 错误元组 {error, Reason}
    pub fn makeError(env: *Env, reason: []const u8) Term {
        const atom_err = Beam.makeAtom(env, "error");
        const atom_reason = Beam.makeAtom(env, reason);
        return erl.enif_make_tuple2(@ptrCast(env), atom_err, atom_reason);
    }

    // ==========================================
    // 2. Numbers (Integer & Float)
    // ==========================================
    pub fn makeInt(env: *Env, val: i32) Term { return erl.enif_make_int(@ptrCast(env), val); }
    pub fn makeInt64(env: *Env, val: i64) Term { return erl.enif_make_int64(@ptrCast(env), val); }
    pub fn getInt64(env: *Env, term: Term) !i64 {
        var val: i64 = 0;
        if (erl.enif_get_int64(@ptrCast(env), term, &val) == 0) return BeamError.InvalidType;
        return val;
    }
    pub fn makeUint64(env: *Env, val: u64) Term { return erl.enif_make_uint64(@ptrCast(env), val); }
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
        if (erl.enif_inspect_binary(@ptrCast(env), term, &bin) == 0) return BeamError.InvalidType;
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
    // 4. Tuples, Lists & Maps
    // ==========================================
    pub fn makeTuple(env: *Env, terms: []const Term) Term { return erl.enif_make_tuple_from_array(@ptrCast(env), terms.ptr, @intCast(terms.len)); }
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
    pub fn makeMap(env: *Env) Term { return erl.enif_make_new_map(@ptrCast(env)); }
    pub fn putMap(env: *Env, map_term: Term, key: Term, value: Term) !Term {
        var new_map: Term = undefined;
        if (erl.enif_make_map_put(@ptrCast(env), map_term, key, value, &new_map) == 0) {
            return BeamError.MapPutFailed;
        }
        return new_map;
    }

    // ==========================================
    // 5. Signal Sensitivity (信号感知)
    // ==========================================
    pub fn isProcessAlive(env: *Env, pid: *Pid) bool {
        return erl.enif_is_process_alive(@ptrCast(env), @ptrCast(pid)) != 0;
    }
    pub fn shouldTerminate(env: *Env) bool {
        var my_pid: erl.ErlNifPid = undefined;
        if (erl.enif_self(@ptrCast(env), &my_pid) == null) return true;
        return erl.enif_is_process_alive(@ptrCast(env), &my_pid) == 0;
    }

    // ==========================================
    // 6. Processes & PIDs (修复恢复)
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
    // 7. Messaging (Fast 8-Byte Path)
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
        return Beam.makeTuple(env, &[_]Term{ atom_js_msg_fast, bin_term });
    }

    pub fn send(env: *Env, to_pid: *Pid, msg: Term) !void {
        if (erl.enif_send(@ptrCast(env), @ptrCast(to_pid), @ptrCast(env), msg) == 0) {
            return BeamError.MessageSendFailed;
        }
    }

    // ==========================================
    // 8. Scheduling & Memory Management
    // ==========================================
    pub fn consumeTimeslice(env: *Env, percent: i32) bool {
        return erl.enif_consume_timeslice(@ptrCast(env), percent) != 0;
    }

    pub fn alloc(size: usize) ?*anyopaque {
        return erl.enif_alloc(size);
    }

    pub fn free(ptr: *anyopaque) void {
        erl.enif_free(ptr);
    }

    // ==========================================
    // 9. RocksDB Native Support
    // ==========================================
    pub const RocksDB = struct {
        const rdb = @cImport({ @cInclude("rocksdb/c.h"); });

        pub fn open(path: [:0]const u8) !*anyopaque {
            const opts = rdb.rocksdb_options_create();
            rdb.rocksdb_options_set_create_if_missing(opts, 1);
            var err: [*c]char = null;
            const db_ptr = rdb.rocksdb_open(opts, path.ptr, &err);
            if (err != null) return error.OpenFailed;
            return @ptrCast(db_ptr);
        }

        pub fn put(db_ptr: *anyopaque, key: []const u8, value: []const u8) !void {
            const wopts = rdb.rocksdb_writeoptions_create();
            var err: [*c]char = null;
            rdb.rocksdb_put(@ptrCast(db_ptr), wopts, key.ptr, key.len, value.ptr, value.len, &err);
            if (err != null) return error.PutFailed;
        }

        pub fn get(db_ptr: *anyopaque, key: []const u8) !?[]const u8 {
            const ropts = rdb.rocksdb_readoptions_create();
            var err: [*c]char = null;
            var len: usize = 0;
            const val = rdb.rocksdb_get(@ptrCast(db_ptr), ropts, key.ptr, key.len, &len, &err);
            if (err != null or val == null) return null;
            return @as([*]u8, @ptrCast(val))[0..len];
        }
    };
};

pub const ERL_NIF_MAJOR_VERSION = erl.ERL_NIF_MAJOR_VERSION;
pub const ERL_NIF_MINOR_VERSION = erl.ERL_NIF_MINOR_VERSION;

pub const SystemAction = enum(u32) {
    NULL_ACTION = 0,
    WAKE = 1,
    SLEEP = 2,
    STOP = 3,
    NET_CONNECT_REQ = 10,
    NET_CONNECT_RES = 11,
    NET_WRITE_REQ = 12,
    NET_READ_REQ = 13,
    NET_READ_RES = 14,
    FS_READ_REQ = 20,
    FS_READ_RES = 21,
    FS_WRITE_REQ = 22,
    FS_WRITE_RES = 23,
    HTTP_FETCH_REQ = 30,
    HTTP_FETCH_RES = 31,
    IO_ERROR = 99,
};

pub const NifFunc = erl.ErlNifFunc;
pub const NifEntry = erl.ErlNifEntry;

pub fn defineNifEntry(
    comptime name: []const u8,
    comptime funcs: []const NifFunc,
) NifEntry {
    return .{
        .major = erl.ERL_NIF_MAJOR_VERSION,
        .minor = erl.ERL_NIF_MINOR_VERSION,
        .name = name.ptr,
        .num_of_funcs = funcs.len,
        .funcs = @constCast(funcs.ptr),
        .vm_variant = "beam.vanilla",
        .options = 1,
        .sizeof_ErlNifResourceTypeInit = @sizeOf(erl.ErlNifResourceTypeInit),
        .min_erts = "erts-10.0",
    };
}

pub const NativeDispatchFunc = fn (env: ?*erl.ErlNifEnv, argc: c_int, argv: [*c]const erl.ERL_NIF_TERM) callconv(.C) erl.ERL_NIF_TERM;
