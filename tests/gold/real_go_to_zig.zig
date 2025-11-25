// UN1C⓪ v0.2: Go → Zig translation
// Expected output for examples/go/real.go

const std = @import("std");

pub fn Fibonacci(n: i32) i32 {
    if (n <= 1) {
        return n;
    }
    return Fibonacci(n - 1) + Fibonacci(n - 2);
}

pub fn FibonacciIterative(n: i32) i32 {
    if (n <= 1) {
        return n;
    }
    var a: i32 = 0;
    var b: i32 = 1;
    var i: i32 = 2;
    while (i <= n) : (i += 1) {
        const temp = a + b;
        a = b;
        b = temp;
    }
    return b;
}

pub fn main() !void {
    const n = 10;
    std.debug.print("Fibonacci({}) = {} (recursive)\n", .{n, Fibonacci(n)});
    std.debug.print("Fibonacci({}) = {} (iterative)\n", .{n, FibonacciIterative(n)});
}
