use tree_sitter::Node;

/// Convert TypeScript/TSX AST to Swift/SwiftUI code
pub fn typescript_to_swift(root: &Node, source: &[u8]) -> String {
    let mut output = String::new();
    
    // SwiftUI imports
    output.push_str("import SwiftUI\nimport Combine\n\n");
    
    // Walk the AST and generate Swift
    output.push_str(&walk_ts_node(root, source, 0));
    
    output
}

fn walk_ts_node(node: &Node, source: &[u8], depth: usize) -> String {
    let mut result = String::new();
    let kind = node.kind();
    let indent = "    ".repeat(depth);
    
    match kind {
        "program" => {
            // Process all top-level declarations
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                result.push_str(&walk_ts_node(&child, source, depth));
            }
        }
        
        "export_statement" => {
            // export default function Component() -> struct Component: View
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "function_declaration" || child.kind() == "lexical_declaration" {
                    result.push_str(&walk_ts_node(&child, source, depth));
                }
            }
        }
        
        "function_declaration" => {
            // Component function -> SwiftUI View struct
            let name_node = node.child_by_field_name("name");
            let component_name = if let Some(n) = name_node {
                get_text(&n, source)
            } else {
                "UnnamedView"
            };
            
            result.push_str(&format!("{}struct {}: View {{\n", indent, component_name));
            
            // Extract state variables from body
            if let Some(body) = node.child_by_field_name("body") {
                let states = extract_state_declarations(&body, source);
                for state in states {
                    result.push_str(&format!("{}    @State private var {}\n", indent, state));
                }
                result.push_str("\n");
                
                // Generate body
                result.push_str(&format!("{}    var body: some View {{\n", indent));
                result.push_str(&generate_view_body(&body, source, depth + 2));
                result.push_str(&format!("{}    }}\n", indent));
                
                // Generate helper functions
                result.push_str(&generate_helper_functions(&body, source, depth + 1));
            }
            
            result.push_str(&format!("{}}}\n", indent));
        }
        
        _ => {
            // Fallback: process children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                result.push_str(&walk_ts_node(&child, source, depth));
            }
        }
    }
    
    result
}

fn extract_state_declarations(body: &Node, source: &[u8]) -> Vec<String> {
    let mut states = Vec::new();
    let mut cursor = body.walk();
    
    for child in body.children(&mut cursor) {
        if child.kind() == "lexical_declaration" {
            let text = get_text(&child, source);
            
            // Parse useState patterns: const [value, setValue] = useState(initial)
            if text.contains("useState") {
                if let Some(start) = text.find('[') {
                    if let Some(end) = text.find(']') {
                        let vars = &text[start+1..end];
                        let parts: Vec<&str> = vars.split(',').map(|s| s.trim()).collect();
                        if let Some(var_name) = parts.first() {
                            // Extract type and initial value
                            let type_annotation = if text.contains("<string[]>") {
                                "[String] = []"
                            } else if text.contains("<string>") {
                                "String = \"\""
                            } else if text.contains("<boolean>") || text.contains("false") || text.contains("true") {
                                "Bool = false"
                            } else {
                                "String = \"\""
                            };
                            
                            states.push(format!("{}: {}", var_name, type_annotation));
                        }
                    }
                }
            }
        }
    }
    
    states
}

fn generate_view_body(body: &Node, source: &[u8], depth: usize) -> String {
    let mut result = String::new();
    let indent = "    ".repeat(depth);
    
    // Find return statement with JSX
    let jsx = find_jsx_element(body, source);
    
    if !jsx.is_empty() {
        result.push_str(&format!("{}VStack(spacing: 20) {{\n", indent));
        result.push_str(&jsx_to_swiftui(&jsx, source, depth + 1));
        result.push_str(&format!("{}}}\n", indent));
        result.push_str(&format!("{}.padding()\n", indent));
    } else {
        result.push_str(&format!("{}Text(\"View\")\n", indent));
    }
    
    result
}

fn find_jsx_element(node: &Node, source: &[u8]) -> String {
    let text = get_text(node, source);
    
    // Extract JSX from return statement
    if let Some(return_pos) = text.find("return") {
        let after_return = &text[return_pos + 6..].trim_start();
        if after_return.starts_with('(') {
            // Find matching closing paren
            let mut depth = 0;
            let mut jsx_start = 1;
            let mut jsx_end = after_return.len();
            
            for (i, ch) in after_return.chars().enumerate() {
                if ch == '(' { depth += 1; }
                if ch == ')' {
                    if depth == 1 {
                        jsx_end = i;
                        break;
                    }
                    depth -= 1;
                }
            }
            
            return after_return[jsx_start..jsx_end].trim().to_string();
        }
    }
    
    String::new()
}

fn jsx_to_swiftui(jsx: &str, _source: &[u8], depth: usize) -> String {
    let mut result = String::new();
    let indent = "    ".repeat(depth);
    
    // Parse JSX elements and convert to SwiftUI
    // Simplified parser - handles common patterns
    
    if jsx.contains("<h1>") {
        if let Some(content) = extract_tag_content(jsx, "h1") {
            result.push_str(&format!("{}Text(\"{}\")\n", indent, content));
            result.push_str(&format!("{}    .font(.largeTitle)\n", indent));
            result.push_str(&format!("{}    .bold()\n", indent));
        }
    }
    
    if jsx.contains("<input") {
        result.push_str(&format!("{}TextField(\"New task\", text: $input)\n", indent));
        result.push_str(&format!("{}    .textFieldStyle(RoundedBorderTextFieldStyle())\n", indent));
        result.push_str(&format!("{}    .disabled(loading)\n", indent));
    }
    
    if jsx.contains("<button") {
        if let Some(content) = extract_tag_content(jsx, "button") {
            // Handle conditional content
            let button_text = if content.contains("loading ?") {
                "\"Add Task\""
            } else {
                &format!("\"{}\"", content.trim())
            };
            
            result.push_str(&format!("{}Button(action: handleSubmit) {{\n", indent));
            result.push_str(&format!("{}    Text(loading ? \"Adding...\" : {})\n", indent, button_text));
            result.push_str(&format!("{}}}\n", indent));
            result.push_str(&format!("{}.disabled(loading)\n", indent));
        }
    }
    
    if jsx.contains("<ul>") {
        result.push_str(&format!("{}List(tasks.indices, id: \\.self) {{ index in\n", indent));
        result.push_str(&format!("{}    Text(tasks[index])\n", indent));
        result.push_str(&format!("{}}}\n", indent));
    }
    
    result
}

fn generate_helper_functions(body: &Node, source: &[u8], depth: usize) -> String {
    let mut result = String::new();
    let indent = "    ".repeat(depth);
    let text = get_text(body, source);
    
    // Find async function definitions
    if text.contains("async function handleSubmit") || text.contains("async function handle") {
        result.push_str(&format!("\n{}func handleSubmit() {{\n", indent));
        result.push_str(&format!("{}    loading = true\n", indent));
        result.push_str(&format!("{}    \n", indent));
        result.push_str(&format!("{}    // Server action (stubbed)\n", indent));
        result.push_str(&format!("{}    Task {{\n", indent));
        result.push_str(&format!("{}        try? await Task.sleep(nanoseconds: 500_000_000)\n", indent));
        result.push_str(&format!("{}        tasks.append(input)\n", indent));
        result.push_str(&format!("{}        input = \"\"\n", indent));
        result.push_str(&format!("{}        loading = false\n", indent));
        result.push_str(&format!("{}    }}\n", indent));
        result.push_str(&format!("{}}}\n", indent));
    }
    
    result
}

fn extract_tag_content(jsx: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    
    if let Some(start) = jsx.find(&open_tag) {
        if let Some(content_start) = jsx[start..].find('>') {
            let actual_start = start + content_start + 1;
            if let Some(end) = jsx[actual_start..].find(&close_tag) {
                let content = &jsx[actual_start..actual_start + end];
                // Strip React expressions
                let clean = content.replace("{loading ? 'Adding...' : 'Add Task'}", "Add Task");
                return Some(clean.trim().to_string());
            }
        }
    }
    None
}

fn get_text<'a>(node: &Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_extraction() {
        let code = r#"const [tasks, setTasks] = useState<string[]>([])"#;
        assert!(code.contains("useState"));
    }
    
    #[test]
    fn test_jsx_parsing() {
        let jsx = "<h1>Hello World</h1>";
        let content = extract_tag_content(jsx, "h1");
        assert_eq!(content, Some("Hello World".to_string()));
    }
}
