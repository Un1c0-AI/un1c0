use std::fs;
use std::path::Path;
use tree_sitter::Parser as TsParser;
extern crate tree_sitter_python;

#[test]
fn emit_and_write_examples() {
    let code = fs::read_to_string("examples/python/fib.py").expect("read example");
    let mut parser = TsParser::new();
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).unwrap();
    let tree = parser.parse(&code, None).unwrap();
    let root = tree.root_node();

    let ueg = un1c0::walker::python_to_ueg(&root, code.as_bytes());
    let go = un1c0::targets::lower_to_go(&ueg);
    let zig = un1c0::targets::lower_to_zig(&ueg);

    // create output dir
    let out_dir = Path::new("examples/generated");
    if !out_dir.exists() { fs::create_dir_all(out_dir).expect("mkdir"); }
    fs::write(out_dir.join("fib.go"), &go).expect("write go");
    fs::write(out_dir.join("fib.zig"), &zig).expect("write zig");
    // Attempt to run formatters if available; ignore errors if not installed.
    use std::process::Command;
    let go_path = out_dir.join("fib.go");
    if Command::new("gofmt").arg("-w").arg(&go_path).status().is_ok() {
        println!("gofmt ran on {}", go_path.display());
    } else {
        println!("gofmt not available or failed; skipping");
    }
    let zig_path = out_dir.join("fib.zig");
    if Command::new("zig").arg("fmt").arg(&zig_path).status().is_ok() {
        println!("zig fmt ran on {}", zig_path.display());
    } else {
        println!("zig fmt not available or failed; skipping");
    }

    // print for manual inspection in CI logs
    println!("--- emitted Go ---\n{}\n--- emitted Zig ---\n{}", go, zig);

    assert!(go.contains("func ") || go.contains("package main"));
    assert!(zig.contains("pub fn ") || zig.contains("const std"));
}
