// Striped concurrent hash map in CP
// Lock contention partitioned across stripes using Wyhash
import { Mutex } from "./mutex";
import { Wyhash } from "./wyhash";
const std = @import("std");

export fn ConcurrentMap(comptime K: type, comptime V: type): type {
    const StripeCount: usize = 32;
    const is_string = @comptime (@typeInfo(K) == .pointer and @typeInfo(K).pointer.size == .slice and @typeInfo(K).pointer.child == u8);
    const MapType = @if (is_string) std.StringHashMap(V) else std.AutoHashMap(K, V);

    const Stripe = struct {
        mutex: Mutex = .{},
        map: MapType,
    };

    return struct {
        const Self = @This();

        stripes: [StripeCount]Stripe = undefined,
        allocator: std.mem.Allocator = undefined,

        pub fn init(allocator: std.mem.Allocator): Self {
            var self: Self = .{
                .stripes = undefined,
                .allocator = allocator,
            };
            for (&self.stripes) |*s| {
                s.mutex = .{};
                s.map = MapType.init(allocator);
            }
            return self;
        }

        pub fn deinit(self: *Self): void {
            for (&self.stripes) |*s| {
                s.mutex.lock();
                s.map.deinit();
                s.mutex.unlock();
                s.mutex.deinit();
            }
        }

        inline fn getStripeIndex(key: K): usize {
            const hash_val = if (comptime is_string)
                Wyhash.hash(0x12345678, key)
            else
                std.hash.Wyhash.hash(0x12345678, std.mem.asBytes(&key));
            return @as(usize, @intCast(hash_val % StripeCount));
        }

        pub fn get(self: *Self, key: K): ?V {
            const idx = getStripeIndex(key);
            const s = &self.stripes[idx];
            s.mutex.lock();
            defer s.mutex.unlock();
            return s.map.get(key);
        }

        pub fn put(self: *Self, key: K, value: V): !void {
            const idx = getStripeIndex(key);
            const s = &self.stripes[idx];
            s.mutex.lock();
            defer s.mutex.unlock();
            try s.map.put(key, value);
        }

        pub fn remove(self: *Self, key: K): bool {
            const idx = getStripeIndex(key);
            const s = &self.stripes[idx];
            s.mutex.lock();
            defer s.mutex.unlock();
            return s.map.remove(key);
        }

        pub fn contains(self: *Self, key: K): bool {
            const idx = getStripeIndex(key);
            const s = &self.stripes[idx];
            s.mutex.lock();
            defer s.mutex.unlock();
            return s.map.contains(key);
        }

        pub fn count(self: *Self): usize {
            var total: usize = 0;
            for (&self.stripes) |*s| {
                s.mutex.lock();
                total += s.map.count();
                s.mutex.unlock();
            }
            return total;
        }

        pub fn clear(self: *Self): void {
            for (&self.stripes) |*s| {
                s.mutex.lock();
                s.map.clearRetainingCapacity();
                s.mutex.unlock();
            }
        }
    };
}
