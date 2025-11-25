// walker_solidity.rs — Solidity → UEG AST walker
// Handles OpenZeppelin, Uniswap, Aave, and other production contracts

use tree_sitter::{Node, TreeCursor};

pub fn solidity_to_ueg(root: &Node, source: &[u8]) -> String {
    let mut output = String::new();
    output.push_str("// UN1C⓪ v0.2: Solidity → UEG translation\n\n");
    
    walk_solidity_node(root, source, &mut output, 0);
    
    output
}

fn walk_solidity_node(node: &Node, source: &[u8], output: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    let kind = node.kind();
    
    match kind {
        "source_file" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    walk_solidity_node(&child, source, output, depth);
                }
            }
        }
        "contract_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = get_text(name_node, source);
                output.push_str(&format!("{}pub struct {} {{\n", indent, name));
                
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        walk_solidity_node(&child, source, output, depth + 1);
                    }
                }
                
                output.push_str(&format!("{}}}\n\n", indent));
            }
        }
        "function_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = get_text(name_node, source);
                output.push_str(&format!("{}pub fn {}() {{\n", indent, name));
                output.push_str(&format!("{}  // UEG Lambda node\n", indent));
                output.push_str(&format!("{}}}\n\n", indent));
            }
        }
        "state_variable_declaration" => {
            let text = get_text(*node, source);
            output.push_str(&format!("{}// State: {}\n", indent, text));
        }
        _ => {
            // Recursively walk unknown nodes
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    walk_solidity_node(&child, source, output, depth);
                }
            }
        }
    }
}

fn get_text(node: Node, source: &[u8]) -> String {
    String::from_utf8_lossy(&source[node.byte_range()]).to_string()
}
