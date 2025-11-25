// walker_go.rs — Go → UEG AST walker
// Handles real-world Go code for Day 2 Go → Zig translation

use tree_sitter::Node;

pub fn go_to_ueg(root: &Node, source: &[u8]) -> String {
    let mut output = String::new();
    output.push_str("// UN1C⓪ v0.2: Go → UEG translation\n\n");
    
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
                output.push_str(&format!("{}pub fn {}() {{\n", indent, name));
                
                if let Some(body) = node.child_by_field_name("body") {
                    walk_go_node(&body, source, output, depth + 1);
                }
                
                output.push_str(&format!("{}}}\n\n", indent));
            }
        }
        "package_clause" => {
            let text = get_text(*node, source);
            output.push_str(&format!("// {}\n\n", text));
        }
        "import_declaration" => {
            let text = get_text(*node, source);
            output.push_str(&format!("// {}\n", text));
        }
        "return_statement" => {
            let text = get_text(*node, source);
            output.push_str(&format!("{}return {};\n", indent, text.trim_start_matches("return ").trim()));
        }
        "if_statement" => {
            output.push_str(&format!("{}if condition {{\n", indent));
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    walk_go_node(&child, source, output, depth + 1);
                }
            }
            output.push_str(&format!("{}}}\n", indent));
        }
        _ => {
            // Recursively walk unknown nodes
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
