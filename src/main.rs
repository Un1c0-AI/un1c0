use clap::Parser;
use std::fs;
use tree_sitter::{Parser as TsParser, Language};
extern crate tree_sitter_python;
extern crate tree_sitter_rust;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Source language (e.g., python)
    from: String,

    /// Target language (e.g., rust)
    to: String,

    /// Input file
    input: String,
}

fn main() {
    let args = Args::parse();
    let code = fs::read_to_string(&args.input).expect("Failed to read file");

    match args.from.as_str() {
        "python" => {
            let mut parser = TsParser::new();
            let language = tree_sitter_python::LANGUAGE;
            parser.set_language(&language.into()).expect("Failed to set language");
            let tree = parser.parse(&code, None).expect("Parse failed");
            let root = tree.root_node();
            println!("// UN1C⓪ v0.1: Python → {} translation", args.to);
            println!("// Parsed {} nodes", root.child_count());
            // Basic rewrite stub: def → fn (expand in next commit)
            let rust_code = code
                .replace("def ", "fn ")
                .replace(": int) -> int:", "(n: i32) -> i32")
                .replace("return ", "return ")
                .replace("if ", "if ")
                .replace("for ", "for ");
            println!("{}", rust_code);
        }
        "rust" => {
            // Stub for Rust → Python (round-trip later)
            println!("// Rust → Python stub: Coming in v0.2");
            println!("{}", code);
        }
        _ => eprintln!("Unsupported lang: {}", args.from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let code = "def add(a, b): return a + b";
        let mut parser = TsParser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        // We assert child_count >= 1 to be resilient across grammar differences
        assert!(tree.root_node().child_count() >= 1);
    }
}

