fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    let mut a: i32 = 0;
    let mut b: i32 = 1;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}
