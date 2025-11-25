// Real Zig code: compile-time generics, error unions, async/await
// From production memory allocator with comptime reflection

const std = @import("std");
const builtin = @import("builtin");

pub const AllocatorError = error{
    OutOfMemory,
    InvalidAlignment,
    NullPointer,
};

pub fn GenericAllocator(comptime T: type) type {
    return struct {
        const Self = @This();
        
        backing_allocator: std.mem.Allocator,
        total_allocated: usize,
        allocation_count: usize,
        
        pub fn init(allocator: std.mem.Allocator) Self {
            return Self{
                .backing_allocator = allocator,
                .total_allocated = 0,
                .allocation_count = 0,
            };
        }
        
        pub fn allocate(self: *Self, n: usize) AllocatorError![]T {
            const bytes = try self.backing_allocator.alloc(T, n);
            self.total_allocated += @sizeOf(T) * n;
            self.allocation_count += 1;
            return bytes;
        }
        
        pub fn deallocate(self: *Self, memory: []T) void {
            self.backing_allocator.free(memory);
            self.total_allocated -= @sizeOf(T) * memory.len;
            self.allocation_count -= 1;
        }
        
        pub fn stats(self: Self) Stats {
            return Stats{
                .total_bytes = self.total_allocated,
                .allocations = self.allocation_count,
                .average_size = if (self.allocation_count > 0) 
                    self.total_allocated / self.allocation_count 
                else 
                    0,
            };
        }
    };
}

pub const Stats = struct {
    total_bytes: usize,
    allocations: usize,
    average_size: usize,
};

// Async I/O with error unions
pub fn asyncRead(file: std.fs.File, buffer: []u8) !usize {
    var frame = async readInternal(file, buffer);
    return await frame;
}

fn readInternal(file: std.fs.File, buffer: []u8) AllocatorError!usize {
    const bytes_read = file.read(buffer) catch |err| {
        return AllocatorError.InvalidAlignment;
    };
    return bytes_read;
}

// Comptime type inspection
pub fn printTypeInfo(comptime T: type) void {
    const info = @typeInfo(T);
    switch (info) {
        .Struct => |s| {
            std.debug.print("Struct with {} fields\n", .{s.fields.len});
        },
        .Int => |i| {
            std.debug.print("Integer: {} bits, signed={}\n", .{i.bits, i.signedness == .signed});
        },
        .Pointer => {
            std.debug.print("Pointer type\n", .{});
        },
        else => {
            std.debug.print("Other type\n", .{});
        },
    }
}

// Optional types and error unions combined
pub fn safeDiv(a: i32, b: i32) AllocatorError!?i32 {
    if (b == 0) return null;
    if (a < 0 and b < 0) return AllocatorError.InvalidAlignment;
    return @divTrunc(a, b);
}

// Testing with comptime blocks
test "GenericAllocator with u64" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    
    var allocator = GenericAllocator(u64).init(gpa.allocator());
    
    const memory = try allocator.allocate(100);
    defer allocator.deallocate(memory);
    
    const s = allocator.stats();
    try std.testing.expect(s.total_bytes == 800); // 100 * 8 bytes
    try std.testing.expect(s.allocations == 1);
}

// Build-time configuration
pub const Config = struct {
    comptime {
        if (builtin.mode == .Debug) {
            @compileLog("Debug mode enabled");
        }
    }
    
    pub const max_buffer_size: usize = if (builtin.mode == .Debug) 1024 else 4096;
    pub const enable_logging: bool = builtin.mode != .ReleaseFast;
};
