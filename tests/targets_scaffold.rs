use std::fs;
use tree_sitter::Parser as TsParser;
extern crate tree_sitter_python;

#[test]
fn go_zig_scaffold_produces_output() {
    let code = fs::read_to_string("examples/fib.py").expect("read example");
    let mut parser = TsParser::new();
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).unwrap();
    let tree = parser.parse(&code, None).unwrap();
    let root = tree.root_node();

    let ueg = un1c0::walker::python_to_ueg(&root, code.as_bytes());
    let go = un1c0::targets::lower_to_go(&ueg);
    let zig = un1c0::targets::lower_to_zig(&ueg);
    assert!(!go.is_empty());
    assert!(!zig.is_empty());
}
