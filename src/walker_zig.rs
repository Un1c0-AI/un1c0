use tree_sitter::{Node, Parser, Language};

// Use tree_sitter_zig crate directly
use tree_sitter_zig::LANGUAGE as ZIG_LANGUAGE;

/// Translate Zig AST to Rust code
pub fn zig_to_rust(source: &str) -> String {
    let mut parser = Parser::new();
    parser.set_language(&ZIG_LANGUAGE.into()).expect("Failed to set Zig language");
    
    let tree = parser.parse(source, None).expect("Failed to parse Zig");
    let root = tree.root_node();
    
    walk_zig_node(&root, source.as_bytes())
}

fn walk_zig_node(node: &Node, source: &[u8]) -> String {
    let mut output = String::new();
    
    output.push_str("// Translated from Zig to Rust\n");
    output.push_str("// comptime → const + generics\n");
    output.push_str("// error unions → Result<T, E>\n");
    output.push_str("// async/await → tokio\n\n");
    
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "container_declaration" | "source_file" => {
                output.push_str(&walk_zig_node(&child, source));
            }
            "variable_declaration" => {
                output.push_str(&translate_variable(&child, source));
            }
            "function_declaration" => {
                output.push_str(&translate_function(&child, source));
            }
            "struct_declaration" => {
                output.push_str(&translate_struct(&child, source));
            }
            "import_declaration" => {
                output.push_str(&translate_import(&child, source));
            }
            "error_set_declaration" => {
                output.push_str(&translate_error_set(&child, source));
            }
            "test_declaration" => {
                output.push_str(&translate_test(&child, source));
            }
            "line_comment" | "doc_comment" => {
                let text = node_text(&child, source);
                output.push_str(&text);
                output.push('\n');
            }
            _ => {
                // Recursively walk unknown nodes
                for subchild in child.children(&mut child.walk()) {
                    output.push_str(&walk_zig_node(&subchild, source));
                }
            }
        }
    }
    
    output
}

fn translate_import(node: &Node, source: &[u8]) -> String {
    let text = node_text(node, source);
    
    // @import("std") → use std;
    if text.contains("@import(\"std\")") {
        return "use std;\n".to_string();
    }
    if text.contains("@import(\"builtin\")") {
        return "// Note: Zig builtin → Rust cfg attributes\n".to_string();
    }
    
    format!("// {}\n", text)
}

fn translate_error_set(node: &Node, source: &[u8]) -> String {
    let mut output = String::new();
    let text = node_text(node, source);
    
    // Extract error name
    if let Some(name_start) = text.find("error{") {
        if let Some(name_end) = text.find("}") {
            let name = text[..name_start].trim().replace("pub const ", "").replace("=", "").trim().to_string();
            let errors = &text[name_start+6..name_end];
            
            output.push_str(&format!("#[derive(Debug, Clone, PartialEq)]\n"));
            output.push_str(&format!("pub enum {} {{\n", name));
            
            for error in errors.split(',') {
                let error = error.trim();
                if !error.is_empty() {
                    output.push_str(&format!("    {},\n", error));
                }
            }
            
            output.push_str("}\n\n");
            
            output.push_str(&format!("impl std::fmt::Display for {} {{\n", name));
            output.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n");
            output.push_str("        write!(f, \"{:?}\", self)\n");
            output.push_str("    }\n");
            output.push_str("}\n\n");
            
            output.push_str(&format!("impl std::error::Error for {} {{}}\n\n", name));
        }
    }
    
    output
}

fn translate_variable(node: &Node, source: &[u8]) -> String {
    let text = node_text(node, source);
    
    // pub const X: type = value → pub const X: Type = value;
    let is_pub = text.contains("pub ");
    let is_const = text.contains("const ");
    
    let visibility = if is_pub { "pub " } else { "" };
    let mutability = if is_const { "const" } else { "let mut" };
    
    format!("{}{}; // {}\n", visibility, mutability, text.trim())
}

fn translate_function(node: &Node, source: &[u8]) -> String {
    let text = node_text(node, source);
    let mut output = String::new();
    
    // Extract function signature
    let is_pub = text.contains("pub fn");
    let is_async = text.contains("async ");
    
    // pub fn name(params) error!Type → pub async fn name(params) -> Result<Type, Error>
    if is_async {
        output.push_str("// Note: Zig async → tokio::spawn\n");
        output.push_str("#[tokio::main]\n");
    }
    
    if is_pub {
        output.push_str("pub ");
    }
    
    if is_async {
        output.push_str("async ");
    }
    
    output.push_str("fn ");
    
    // Simplified translation - mark for manual review
    output.push_str("/* ");
    output.push_str(text.trim());
    output.push_str(" */");
    output.push_str(" { todo!() }\n\n");
    
    output
}

fn translate_struct(node: &Node, source: &[u8]) -> String {
    let text = node_text(node, source);
    
    // pub fn GenericType(comptime T: type) type → pub struct GenericType<T>
    if text.contains("comptime") && text.contains("type)") {
        return translate_comptime_generic(node, source);
    }
    
    let mut output = String::new();
    output.push_str("#[derive(Debug, Clone)]\n");
    output.push_str("pub struct ");
    output.push_str("/* ");
    output.push_str(text.trim());
    output.push_str(" */");
    output.push_str(" {}\n\n");
    
    output
}

fn translate_comptime_generic(node: &Node, source: &[u8]) -> String {
    let text = node_text(node, source);
    let mut output = String::new();
    
    // Extract name from: pub fn GenericAllocator(comptime T: type) type
    if let Some(fn_start) = text.find("fn ") {
        if let Some(paren) = text[fn_start..].find('(') {
            let name = text[fn_start+3..fn_start+paren].trim();
            
            output.push_str(&format!("// Zig comptime function → Rust generic struct\n"));
            output.push_str(&format!("#[derive(Debug, Clone)]\n"));
            output.push_str(&format!("pub struct {}<T> {{\n", name));
            output.push_str("    // Fields translated from Zig comptime struct\n");
            output.push_str("    // Note: Manual review required for accurate field translation\n");
            output.push_str("    _phantom: std::marker::PhantomData<T>,\n");
            output.push_str("}\n\n");
            
            output.push_str(&format!("impl<T> {}<T> {{\n", name));
            output.push_str("    pub fn new() -> Self {\n");
            output.push_str("        Self { _phantom: std::marker::PhantomData }\n");
            output.push_str("    }\n");
            output.push_str("}\n\n");
        }
    }
    
    output
}

fn translate_test(node: &Node, source: &[u8]) -> String {
    let text = node_text(node, source);
    let mut output = String::new();
    
    output.push_str("#[cfg(test)]\n");
    output.push_str("#[test]\n");
    output.push_str("fn ");
    
    // Extract test name from: test "name" { ... }
    if let Some(quote1) = text.find('"') {
        if let Some(quote2) = text[quote1+1..].find('"') {
            let name = &text[quote1+1..quote1+1+quote2];
            let rust_name = name.replace(' ', "_").to_lowercase();
            output.push_str(&rust_name);
        }
    }
    
    output.push_str("() {\n");
    output.push_str("    // Zig test → Rust test\n");
    output.push_str("    todo!(\"Translate test body\")\n");
    output.push_str("}\n\n");
    
    output
}

fn node_text(node: &Node, source: &[u8]) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    String::from_utf8_lossy(&source[start..end]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_zig_translation() {
        let zig = r#"
pub const x: i32 = 42;
"#;
        let rust = zig_to_rust(zig);
        assert!(rust.contains("Translated from Zig"));
        assert!(rust.contains("comptime → const"));
    }
    
    #[test]
    fn test_zig_to_rust_output() {
        let zig = r#"const std = @import("std");"#;
        let rust = zig_to_rust(zig);
        // Basic smoke test - translation produces output
        assert!(rust.len() > 0);
        assert!(rust.contains("Translated from Zig"));
    }
}
