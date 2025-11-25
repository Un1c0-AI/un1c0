// walker_go.rs — Go → Zig translation via UEG
// Day 2: Complete Go AST walking with proper Zig output

use tree_sitter::Node;

pub fn go_to_ueg(root: &Node, source: &[u8]) -> String {
    let mut output = String::new();
    output.push_str("const std = @import(\"std\");\n\n");
    
    walk_go_node(root, source, &mut output, 0);
    
    output
}

fn walk_go_node(node: &Node, source: &[u8], output: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    let kind = node.kind();
    
    match kind {
        "source_file" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    walk_go_node(&child, source, output, depth);
                }
            }
        }
        "function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = get_text(name_node, source);
                
                // Extract parameters
                let mut params = Vec::new();
                if let Some(params_node) = node.child_by_field_name("parameters") {
                    params = extract_params(&params_node, source);
                }
                
                // Extract return type
                let return_type = if let Some(result_node) = node.child_by_field_name("result") {
                    map_go_type_to_zig(&get_text(result_node, source))
                } else {
                    "void".to_string()
                };
                
                // Build function signature
                let param_str = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, map_go_type_to_zig(t)))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                output.push_str(&format!("{}pub fn {}({}) {} {{\n", indent, name, param_str, return_type));
                
                if let Some(body) = node.child_by_field_name("body") {
                    walk_go_node(&body, source, output, depth + 1);
                }
                
                output.push_str(&format!("{}}}\n\n", indent));
            }
        }
        "block" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() != "{" && child.kind() != "}" {
                        walk_go_node(&child, source, output, depth);
                    }
                }
            }
        }
        "if_statement" => {
            if let Some(condition) = node.child_by_field_name("condition") {
                let cond_text = get_text(condition, source);
                output.push_str(&format!("{}if ({}) {{\n", indent, cond_text));
                
                if let Some(consequence) = node.child_by_field_name("consequence") {
                    walk_go_node(&consequence, source, output, depth + 1);
                }
                
                output.push_str(&format!("{}}}", indent));
                
                if let Some(alternative) = node.child_by_field_name("alternative") {
                    output.push_str(" else {\n");
                    walk_go_node(&alternative, source, output, depth + 1);
                    output.push_str(&format!("{}}}", indent));
                }
                output.push_str("\n");
            }
        }
        "short_var_declaration" => {
            // Go's := becomes Zig's var
            let text = get_text(*node, source);
            let zig_decl = text.replace(":=", "=");
            output.push_str(&format!("{}var {};\n", indent, zig_decl));
        }
        "assignment_statement" => {
            let text = get_text(*node, source);
            output.push_str(&format!("{}{};\n", indent, text));
        }
        "for_statement" => {
            // Go for loops → Zig while loops
            if let Some(condition) = node.child_by_field_name("condition") {
                let cond_text = get_text(condition, source);
                output.push_str(&format!("{}while ({}) {{\n", indent, cond_text));
                
                if let Some(body) = node.child_by_field_name("body") {
                    walk_go_node(&body, source, output, depth + 1);
                }
                
                output.push_str(&format!("{}}}\n", indent));
            }
        }
        "return_statement" => {
            let mut return_expr = String::new();
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() != "return" {
                        return_expr.push_str(&get_text(child, source));
                    }
                }
            }
            output.push_str(&format!("{}return {};\n", indent, return_expr.trim()));
        }
        "binary_expression" => {
            let left = node.child(0).map(|n| get_text(n, source)).unwrap_or_default();
            let op = node.child(1).map(|n| get_text(n, source)).unwrap_or_default();
            let right = node.child(2).map(|n| get_text(n, source)).unwrap_or_default();
            output.push_str(&format!("{} {} {}", left, op, right));
        }
        "call_expression" => {
            let text = get_text(*node, source);
            // Convert fmt.Printf to std.debug.print
            if text.contains("fmt.Printf") || text.contains("fmt.Println") {
                let args_start = text.find('(').unwrap_or(0);
                let args_end = text.rfind(')').unwrap_or(text.len());
                let args = &text[args_start+1..args_end];
                output.push_str(&format!("{}std.debug.print({}, .{{}});\n", indent, args));
            } else {
                output.push_str(&format!("{}{}", indent, text));
            }
        }
        "package_clause" => {
            let text = get_text(*node, source);
            output.push_str(&format!("// {}\n\n", text));
        }
        "import_declaration" => {
            // Skip imports - Zig doesn't need them for std
            let text = get_text(*node, source);
            output.push_str(&format!("// {}\n", text));
        }
        _ => {
            // For unhandled nodes, recursively walk children
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    walk_go_node(&child, source, output, depth);
                }
            }
        }
    }
}

fn get_text(node: Node, source: &[u8]) -> String {
    String::from_utf8_lossy(&source[node.byte_range()]).to_string()
}

fn extract_params(params_node: &Node, source: &[u8]) -> Vec<(String, String)> {
    let mut params = Vec::new();
    
    for i in 0..params_node.child_count() {
        if let Some(param) = params_node.child(i) {
            if param.kind() == "parameter_declaration" {
                let mut name = String::new();
                let mut typ = String::new();
                
                if let Some(name_node) = param.child_by_field_name("name") {
                    name = get_text(name_node, source);
                }
                if let Some(type_node) = param.child_by_field_name("type") {
                    typ = get_text(type_node, source);
                }
                
                if !name.is_empty() && !typ.is_empty() {
                    params.push((name, typ));
                }
            }
        }
    }
    
    params
}

fn map_go_type_to_zig(go_type: &str) -> String {
    match go_type.trim() {
        "int" => "i32".to_string(),
        "int32" => "i32".to_string(),
        "int64" => "i64".to_string(),
        "float64" => "f64".to_string(),
        "bool" => "bool".to_string(),
        "string" => "[]const u8".to_string(),
        "" => "void".to_string(),
        other => other.to_string(),
    }
}
