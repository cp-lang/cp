__zig__ {
    extern "c" fn sqlite3_open(filename: [*c]const u8, ppDb: *?*anyopaque) c_int;
    extern "c" fn sqlite3_close(db: ?*anyopaque) c_int;
    extern "c" fn sqlite3_exec(db: ?*anyopaque, sql: [*c]const u8, callback: ?*anyopaque, arg: ?*anyopaque, errmsg: ?*?*anyopaque) c_int;
    extern "c" fn sqlite3_errmsg(db: ?*anyopaque) [*c]const u8;
    extern "c" fn sqlite3_prepare_v2(db: ?*anyopaque, zSql: [*c]const u8, nByte: c_int, ppStmt: *?*anyopaque, pzTail: ?*?*anyopaque) c_int;
    extern "c" fn sqlite3_step(stmt: ?*anyopaque) c_int;
    extern "c" fn sqlite3_column_text(stmt: ?*anyopaque, iCol: c_int) [*c]const u8;
    extern "c" fn sqlite3_finalize(stmt: ?*anyopaque) c_int;
}

export class Database {
    ptr: any?,

    fn init(this, path: string): ?void {
        __zig__ {
            var db: ?*anyopaque = null;
            const allocator = std.heap.page_allocator;
            const path_z = allocator.alloc(u8, path.len + 1) catch return error.OutOfMemory;
            std.mem.copyForwards(u8, path_z, path);
            path_z[path.len] = 0;
            defer allocator.free(path_z);
            
            if (sqlite3_open(path_z.ptr, &db) != 0) {
                if (db != null) {
                    _ = sqlite3_close(db);
                }
                return error.SQLiteOpenFailed;
            }
            self.ptr = db;
            std.debug.print("DB Open OK! db={*}\n", .{self.ptr});
        }
    }

    fn exec(this, sql: string): ?void {
        __zig__ {
            std.debug.print("Executing SQL on db={*}...\n", .{self.ptr});
            const allocator = std.heap.page_allocator;
            const sql_z = allocator.alloc(u8, sql.len + 1) catch return error.OutOfMemory;
            std.mem.copyForwards(u8, sql_z, sql);
            sql_z[sql.len] = 0;
            defer allocator.free(sql_z);

            if (sqlite3_exec(self.ptr, sql_z.ptr, null, null, null) != 0) {
                const err_msg = sqlite3_errmsg(self.ptr);
                std.debug.print("SQLite Error: {s}\n", .{err_msg});
                return error.SQLiteExecFailed;
            }
        }
    }

    fn query_string(this, sql: string): ?string {
        __zig__ {
            const allocator = std.heap.page_allocator;
            const sql_z = allocator.alloc(u8, sql.len + 1) catch return error.OutOfMemory;
            std.mem.copyForwards(u8, sql_z, sql);
            sql_z[sql.len] = 0;
            defer allocator.free(sql_z);

            var stmt: ?*anyopaque = null;
            if (sqlite3_prepare_v2(self.ptr, sql_z.ptr, -1, &stmt, null) != 0) {
                return error.SQLitePrepareFailed;
            }
            defer _ = sqlite3_finalize(stmt);

            if (sqlite3_step(stmt) == 100) { // 100 is SQLITE_ROW
                const text_ptr = sqlite3_column_text(stmt, 0);
                if (text_ptr != null) {
                    const span = std.mem.span(text_ptr);
                    const result = allocator.alloc(u8, span.len) catch return error.OutOfMemory;
                    std.mem.copyForwards(u8, result, span);
                    return result;
                }
            }
            return "";
        }
    }

    fn release(this): void {
        __zig__ {
            if (self.ptr) |db_ptr| {
                _ = sqlite3_close(db_ptr);
                self.ptr = null;
                std.debug.print("[Auto-Drop] SQLite Database closed.\n", .{});
            }
        }
    }
}
