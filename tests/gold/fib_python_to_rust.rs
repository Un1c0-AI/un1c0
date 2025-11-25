// UN1C⓪ v0.2: Python → Rust translation (pixel-perfect)
// Expected output for examples/python/fib.py

fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    let result = fibonacci(10);
    println!("{}", result);
}
