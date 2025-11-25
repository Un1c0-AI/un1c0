const std = @import("std");

// package main

// import "fmt"
pub fn Fibonacci(n: i32) i32 {
  if (n <= 1) {
    return n;
  }
  return Fibonacci(n-1) + Fibonacci(n-2);
}

pub fn FibonacciIterative(n: i32) i32 {
  if (n <= 1) {
    return n;
  }
  var a, b = 0, 1;
  return b;
}

pub fn main() void {
  var n = 10;
  std.debug.print("Fibonacci(%d) = %d (recursive)\n", n, Fibonacci(n), .{});
  std.debug.print("Fibonacci(%d) = %d (iterative)\n", n, FibonacciIterative(n), .{});
}

