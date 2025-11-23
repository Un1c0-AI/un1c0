use std::fs;
use tree_sitter::Parser as TsParser;
extern crate tree_sitter_python;

#[test]
fn ueg_to_python_roundtrip() {
    let code = fs::read_to_string("examples/fib.py").expect("read example");
    let mut parser = TsParser::new();
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).unwrap();
    let tree = parser.parse(&code, None).unwrap();
    let root = tree.root_node();

    // Build UEG
    let ueg = un1c0::walker::python_to_ueg(&root, code.as_bytes());
    assert!(!ueg.nodes.is_empty());

    // Lower to Python
    let py = un1c0::ueg_python::lower_to_python(&ueg);
    assert!(py.contains("def fib("));
}
