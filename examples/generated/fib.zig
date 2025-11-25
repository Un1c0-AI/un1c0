const std = @import("std");

pub fn fib(n: i32) i32 {
    if n <= 1 {;
    return n;
    var a = 0;
    var b = 1;
    var i: i32 = 2;
    while (i <= n) {
        let temp = a + b;
        a = b;
        b = temp;
    b
}
