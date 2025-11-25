package main

import "fmt"

// Fibonacci calculates the nth Fibonacci number
func Fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return Fibonacci(n-1) + Fibonacci(n-2)
}

// FibonacciIterative calculates Fibonacci iteratively (more efficient)
func FibonacciIterative(n int) int {
	if n <= 1 {
		return n
	}
	a, b := 0, 1
	for i := 2; i <= n; i++ {
		a, b = b, a+b
	}
	return b
}

func main() {
	n := 10
	fmt.Printf("Fibonacci(%d) = %d (recursive)\n", n, Fibonacci(n))
	fmt.Printf("Fibonacci(%d) = %d (iterative)\n", n, FibonacciIterative(n))
}
