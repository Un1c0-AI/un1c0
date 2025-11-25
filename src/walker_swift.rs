use regex::Regex;

/// Convert Swift code to Rust using regex patterns (tree-sitter-swift incompatible with tree-sitter 0.24)
pub fn swift_to_rust_regex(source: &str) -> String {
    let mut output = String::new();
    
    // Standard Rust imports
    output.push_str("use std::sync::Arc;\n");
    output.push_str("use std::sync::Mutex;\n");
    output.push_str("// Add: uuid = \"1.0\" to Cargo.toml\n");
    output.push_str("// use uuid::Uuid;\n\n");
    output.push_str("// Translated from Swift to Rust\n");
    output.push_str("// SwiftUI → Rust (UI framework: egui/iced/dioxus)\n");
    output.push_str("// @Published → Arc<Mutex<T>>\n");
    output.push_str("// async/await → tokio::spawn\n\n");
    
    // Parse Swift constructs
    let mut rust_code = source.to_string();
    
    // Remove imports
    let import_re = Regex::new(r"import\s+\w+\s*\n").unwrap();
    rust_code = import_re.replace_all(&rust_code, "").to_string();
    
    // Remove decorators
    rust_code = rust_code.replace("@MainActor", "");
    rust_code = rust_code.replace("@Published", "");
    rust_code = rust_code.replace("@StateObject", "");
    rust_code = rust_code.replace("@State", "");
    
    // Class declarations
    let class_re = Regex::new(r"class\s+(\w+):\s*ObservableObject").unwrap();
    rust_code = class_re.replace_all(&rust_code, "pub struct $1").to_string();
    
    let class_simple_re = Regex::new(r"class\s+(\w+)").unwrap();
    rust_code = class_simple_re.replace_all(&rust_code, "pub struct $1").to_string();
    
    // Struct declarations with protocols
    let struct_re = Regex::new(r"struct\s+(\w+):\s*\w+(\s*,\s*\w+)*").unwrap();
    rust_code = struct_re.replace_all(&rust_code, "#[derive(Clone, Debug)]\npub struct $1").to_string();
    
    // Function declarations
    rust_code = rust_code.replace("func ", "pub fn ");
    rust_code = rust_code.replace("private func ", "fn ");
    
    // Init → new
    let init_re = Regex::new(r"pub fn init\(\)").unwrap();
    rust_code = init_re.replace_all(&rust_code, "pub fn new() -> Self").to_string();
    
    // Swift types → Rust types
    rust_code = rust_code.replace(": String", ": String");
    rust_code = rust_code.replace(": Bool", ": bool");
    rust_code = rust_code.replace(": Int", ": i64");
    rust_code = rust_code.replace(": UUID", ": Uuid");
    rust_code = rust_code.replace("[Task]", "Vec<Task>");
    rust_code = rust_code.replace("Set<AnyCancellable>", "Vec<()>");
    
    // Optional types
    let optional_re = Regex::new(r":\s*(\w+)\?").unwrap();
    rust_code = optional_re.replace_all(&rust_code, ": Option<$1>").to_string();
    
    // Swift syntax → Rust syntax
    rust_code = rust_code.replace("var ", "pub ");
    rust_code = rust_code.replace("let ", "let ");
    rust_code = rust_code.replace(" = Set<AnyCancellable>()", " = Vec::new()");
    
    // Property wrappers cleanup
    rust_code = rust_code.replace("private var viewModel = ", "");
    
    output.push_str(&rust_code);
    output.push_str("\n\n// Note: Swift async/await, Combine, SwiftUI require runtime support\n");
    output.push_str("// Recommend: tokio for async, egui/iced for UI\n");
    
    output
}

// Keep the tree-sitter version for when compatibility is fixed
use tree_sitter::Node;

#[allow(dead_code)]
/// Convert Swift AST to Rust code (currently disabled due to tree-sitter version incompatibility)
pub fn swift_to_rust(root: &Node, source: &[u8]) -> String {
    let mut output = String::new();
    
    // Standard Rust imports for common Swift patterns
    output.push_str("use std::sync::Arc;\n");
    output.push_str("use std::sync::Mutex;\n");
    output.push_str("use uuid::Uuid;\n\n");
    output.push_str("// Translated from Swift to Rust\n");
    output.push_str("// SwiftUI → Rust (UI framework TBD: egui/iced/dioxus)\n\n");
    
    // Walk the AST
    output.push_str(&walk_swift_node(root, source, 0));
    
    output
}

fn walk_swift_node(node: &Node, source: &[u8], depth: usize) -> String {
    let mut result = String::new();
    let kind = node.kind();
    let indent = "    ".repeat(depth);
    
    match kind {
        "source_file" | "program" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                result.push_str(&walk_swift_node(&child, source, depth));
            }
        }
        
        "class_declaration" => {
            // Swift class → Rust struct with interior mutability
            let name = extract_class_name(node, source);
            let is_observable = has_observable_object(node, source);
            
            if is_observable {
                result.push_str(&format!("{}// ObservableObject → shared mutable state\n", indent));
            }
            
            result.push_str(&format!("{}pub struct {} {{\n", indent, name));
            
            // Extract properties
            let properties = extract_properties(node, source);
            for prop in properties {
                result.push_str(&format!("{}    pub {}: {},\n", indent, prop.name, prop.rust_type));
            }
            
            result.push_str(&format!("{}}}\n\n", indent));
            
            // Generate impl block
            result.push_str(&format!("{}impl {} {{\n", indent, name));
            
            // Extract methods
            let methods = extract_methods(node, source, depth + 1);
            result.push_str(&methods);
            
            result.push_str(&format!("{}}}\n\n", indent));
        }
        
        "struct_declaration" => {
            // Swift struct → Rust struct
            let name = extract_struct_name(node, source);
            let has_identifiable = check_for_protocol(node, source, "Identifiable");
            
            result.push_str(&format!("{}#[derive(Clone", indent));
            if has_identifiable {
                result.push_str(", Debug");
            }
            result.push_str(")]\n");
            
            result.push_str(&format!("{}pub struct {} {{\n", indent, name));
            
            let properties = extract_properties(node, source);
            for prop in properties {
                result.push_str(&format!("{}    pub {}: {},\n", indent, prop.name, prop.rust_type));
            }
            
            result.push_str(&format!("{}}}\n\n", indent));
        }
        
        _ => {
            // Process children for other node types
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                result.push_str(&walk_swift_node(&child, source, depth));
            }
        }
    }
    
    result
}

struct Property {
    name: String,
    rust_type: String,
}

fn extract_class_name(node: &Node, source: &[u8]) -> String {
    if let Some(name_node) = node.child_by_field_name("name") {
        get_text(&name_node, source).to_string()
    } else {
        "UnnamedClass".to_string()
    }
}

fn extract_struct_name(node: &Node, source: &[u8]) -> String {
    if let Some(name_node) = node.child_by_field_name("name") {
        get_text(&name_node, source).to_string()
    } else {
        "UnnamedStruct".to_string()
    }
}

fn has_observable_object(node: &Node, source: &[u8]) -> bool {
    let text = get_text(node, source);
    text.contains("ObservableObject")
}

fn check_for_protocol(node: &Node, source: &[u8], protocol: &str) -> bool {
    let text = get_text(node, source);
    text.contains(protocol)
}

fn extract_properties(node: &Node, source: &[u8]) -> Vec<Property> {
    let mut properties = Vec::new();
    let mut cursor = node.walk();
    
    for child in node.children(&mut cursor) {
        if child.kind() == "property_declaration" {
            if let Some(name_node) = child.child_by_field_name("name") {
                let name = get_text(&name_node, source).to_string();
                
                // Determine Rust type from Swift type
                let swift_type = extract_swift_type(&child, source);
                let rust_type = map_swift_type_to_rust(&swift_type);
                
                properties.push(Property {
                    name: name.trim_start_matches('@').to_string(),
                    rust_type,
                });
            }
        }
    }
    
    properties
}

fn extract_swift_type(node: &Node, source: &[u8]) -> String {
    if let Some(type_node) = node.child_by_field_name("type") {
        get_text(&type_node, source).to_string()
    } else {
        // Try to infer from the node text
        let text = get_text(node, source);
        
        if text.contains("[Task]") || text.contains("Array<Task>") {
            "[Task]".to_string()
        } else if text.contains("Bool") {
            "Bool".to_string()
        } else if text.contains("String") {
            "String".to_string()
        } else if text.contains("Set<") {
            "Set".to_string()
        } else {
            "Any".to_string()
        }
    }
}

fn map_swift_type_to_rust(swift_type: &str) -> String {
    match swift_type.trim() {
        "String" | "String?" => "String".to_string(),
        "Int" | "Int?" => "i64".to_string(),
        "UInt" | "UInt?" => "u64".to_string(),
        "Bool" | "Bool?" => "bool".to_string(),
        "Double" | "Double?" => "f64".to_string(),
        "Float" | "Float?" => "f32".to_string(),
        "[Task]" => "Vec<Task>".to_string(),
        "UUID" => "Uuid".to_string(),
        "Set<AnyCancellable>" => "Vec<()> /* Combine cancellables */".to_string(),
        t if t.starts_with('[') && t.ends_with(']') => {
            let inner = &t[1..t.len()-1];
            format!("Vec<{}>", map_swift_type_to_rust(inner))
        }
        t if t.ends_with('?') => {
            let inner = &t[..t.len()-1];
            format!("Option<{}>", map_swift_type_to_rust(inner))
        }
        other => other.to_string(),
    }
}

fn extract_methods(node: &Node, source: &[u8], depth: usize) -> String {
    let mut result = String::new();
    let _indent = "    ".repeat(depth);
    let mut cursor = node.walk();
    
    for child in node.children(&mut cursor) {
        if child.kind() == "function_declaration" {
            let method_code = translate_swift_function(&child, source, depth);
            result.push_str(&method_code);
        }
    }
    
    result
}

fn translate_swift_function(node: &Node, source: &[u8], depth: usize) -> String {
    let mut result = String::new();
    let indent = "    ".repeat(depth);
    
    // Extract function name
    let name = if let Some(name_node) = node.child_by_field_name("name") {
        get_text(&name_node, source).to_string()
    } else {
        "unnamed".to_string()
    };
    
    // Check for init
    let is_init = name == "init";
    let rust_name = if is_init { "new" } else { &name };
    
    // Check for async
    let text = get_text(node, source);
    let is_async = text.contains("async") || text.contains("await");
    
    // Build function signature
    result.push_str(&format!("{}pub ", indent));
    if is_async {
        result.push_str("async ");
    }
    result.push_str(&format!("fn {}(", rust_name));
    
    // Parameters
    let params = extract_parameters(node, source);
    if !is_init {
        result.push_str("&self");
        if !params.is_empty() {
            result.push_str(", ");
        }
    }
    
    for (i, param) in params.iter().enumerate() {
        result.push_str(&format!("{}: {}", param.name, param.rust_type));
        if i < params.len() - 1 {
            result.push_str(", ");
        }
    }
    
    result.push_str(")");
    
    // Return type
    if is_init {
        result.push_str(" -> Self");
    }
    
    result.push_str(" {\n");
    
    // Extract method body from Swift AST
    let mut cursor = node.walk();
    let mut body_found = false;
    for child in node.children(&mut cursor) {
        if child.kind() == "function_body" || child.kind() == "code_block" {
            let body = translate_swift_body(&child, source, &format!("{}    ", indent));
            result.push_str(&body);
            body_found = true;
            break;
        }
    }
    
    if !body_found {
        if is_init {
            result.push_str(&format!("{}    Self {{ /* initialize fields */ }}\n", indent));
        } else if is_async {
            result.push_str(&format!("{}    Ok(()) // async implementation\n", indent));
        } else {
            result.push_str(&format!("{}    () // implementation\n", indent));
        }
    }
    
    result.push_str(&format!("{}}}\n\n", indent));
    
    result
}

struct Parameter {
    name: String,
    rust_type: String,
}

fn extract_parameters(node: &Node, source: &[u8]) -> Vec<Parameter> {
    let mut params = Vec::new();
    
    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "parameter" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = get_text(&name_node, source).to_string();
                    let swift_type = extract_swift_type(&child, source);
                    let rust_type = map_swift_type_to_rust(&swift_type);
                    
                    params.push(Parameter { name, rust_type });
                }
            }
        }
    }
    
    params
}

fn get_text<'a>(node: &Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

fn translate_swift_body(node: &Node, source: &[u8], indent: &str) -> String {
    let mut output = String::new();
    let mut cursor = node.walk();
    
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        let text = get_text(&child, source);
        
        match kind {
            "property_declaration" | "variable_declaration" => {
                let rust_line = text.replace("var ", "let mut ")
                    .replace("let ", "let ")
                    .replace(": String", ": String")
                    .replace("await ", ".await");
                output.push_str(&format!("{}{};\\n", indent, rust_line));
            }
            "call_expression" => {
                let rust_call = text.replace("await ", ".await ");
                output.push_str(&format!("{}{};\\n", indent, rust_call));
            }
            "return_statement" => {
                output.push_str(&format!("{}{}\\n", indent, text));
            }
            "if_statement" | "guard_statement" => {
                output.push_str(&format!("{}{}\\n", indent, text.replace("guard", "if")));
            }
            _ => {
                if !text.trim().is_empty() && text.trim() != "{" && text.trim() != "}" {
                    output.push_str(&format!("{}{}\\n", indent, text));
                }
            }
        }
    }
    
    if output.trim().is_empty() {
        output.push_str(&format!("{}// empty body\\n", indent));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regex_basic_translation() {
        let swift = "class MyClass: ObservableObject {}";
        let rust = swift_to_rust_regex(swift);
        assert!(rust.contains("pub struct MyClass"));
    }
    
    #[test]
    fn test_swift_bool_to_rust() {
        let swift = "var isActive: Bool = false";
        let rust = swift_to_rust_regex(swift);
        assert!(rust.contains("bool"));
    }
}
