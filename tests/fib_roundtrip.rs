#[test]
fn fib_transpiles_perfectly() {
    let expected = include_str!("./gold/fib.rs");
    let output = std::process::Command::new("cargo")
        .args(["run", "--release", "--", "python", "rust", "examples/python/fib.py"]) 
        .output()
        .unwrap();
    let generated = String::from_utf8_lossy(&output.stdout);
    // Normalize whitespace just in case
    let generated = generated.split_whitespace().collect::<Vec<_>>().join(" ");
    let expected = expected.split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(generated, expected);
}
