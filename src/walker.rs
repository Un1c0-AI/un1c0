use tree_sitter::Node;

fn node_text(node: &Node, source: &[u8]) -> String {
    let b = node.start_byte();
    let e = node.end_byte();
    String::from_utf8_lossy(&source[b..e]).to_string()
}

fn map_type(ann: &str) -> String {
    match ann.trim() {
        "int" => "i32".into(),
        "bool" => "bool".into(),
        "str" => "String".into(),
        "float" => "f64".into(),
        other => other.into(),
    }
}

fn translate_signature(signature: &str) -> (String, Vec<(String, String)>, Option<String>) {
    // Quick regex-free parse: assume form `def name(params) -> ret:`
    let sig = signature.trim().trim_start_matches("def").trim();
    let parts: Vec<&str> = sig.splitn(2, '(').collect();
    let name = parts[0].trim().to_string();
    let rest = parts[1];
    let parts2: Vec<&str> = rest.splitn(2, ')').collect();
    let params_str = parts2[0].trim();
    let after = parts2[1];

    // parse return annotation if present
    let mut ret: Option<String> = None;
    if let Some(idx) = after.find("->") {
        let after_arrow = &after[idx + 2..];
        if let Some(colon_idx) = after_arrow.find(":") {
            let raw = &after_arrow[..colon_idx];
            ret = Some(map_type(raw).to_string());
        }
    }

    // parse params
    let mut params = Vec::new();
    if !params_str.is_empty() {
        for p in params_str.split(',') {
            let p = p.trim();
            if p.is_empty() { continue }
            if let Some(idx) = p.find(":") {
                let name = p[..idx].trim().to_string();
                let typ = p[idx+1..].trim();
                params.push((name, map_type(typ)));
            } else {
                params.push((p.to_string(), "_".into()));
            }
        }
    }

    (name, params, ret)
}

pub fn python_to_rust(root: &Node, source: &[u8]) -> String {
    // Find first function-like node by scanning children
    let mut out = String::new();

    for child in root.named_children(&mut root.walk()) {
        let kind = child.kind();
        if kind.contains("function") || kind == "function_definition" || kind=="function_def" {
            // Extract header text up to the ':' before body
            let start = child.start_byte();
            // find the ':' that ends the signature by walking children to parameters end
            // fallback: take first line
            let start_line = child.start_position().row;
            let mut sig_line = String::new();
            if let Some(line) = source.split(|b| *b == b'\n').nth(start_line) {
                sig_line = String::from_utf8_lossy(line).trim().to_string();
            }

            let (name, params, ret) = translate_signature(&sig_line);
            // build fn signature
            let mut sig = format!("fn {}(", name);
            let mut first = true;
            for (n, t) in params.iter() {
                if !first { sig.push_str(", "); }
                first = false;
                if t == "_" {
                    sig.push_str(&format!("{}: impl std::fmt::Debug", n));
                } else {
                    sig.push_str(&format!("{}: {}", n, t));
                }
            }
            sig.push(')');
            if let Some(r) = ret {
                sig.push_str(&format!(" -> {}", r));
            }
            sig.push_str(" {");
            out.push_str(&sig);
            out.push('\n');

            // Process body lines (simple line-based walker within function body range)
            // find body start row
            let body_node = child.named_child(1);
            let body_node = match body_node {
                Some(n) => n,
                None => { out.push_str("}\n"); continue }
            ;
            let body_start_row = body_node.start_position().row;
            let body_end_row = body_node.end_position().row;

            let mut lines: Vec<String> = Vec::new();
            let all_lines: Vec<&[u8]> = source.split(|b| *b == b'\n').collect();
            for r in body_start_row..=body_end_row {
                if let Some(l) = all_lines.get(r) {
                    lines.push(String::from_utf8_lossy(l).to_string());
                }
            }

            // Trim leading indentation
            for i in 0..lines.len() {
                lines[i] = lines[i].trim().to_string();
            }

            // Process statements
            let mut idx = 0;
            while idx < lines.len() {
                let line = &lines[idx];
                if line.starts_with("if ") && line.ends_with(":") {
                    let cond = line.trim_end_matches(":").trim_start_matches("if ").trim();
                    out.push_str(&format!("    if {} {{\n", cond));
                    idx += 1;
                    // handle simple return immediately following
                    if idx < lines.len() {
                        let inner = lines[idx].clone();
                        if inner.starts_with("return ") {
                            let expr = inner.trim_start_matches("return ").trim();
                            out.push_str(&format!("        return {};\n", expr));
                            idx += 1;
                        }
                    }
                    out.push_str("    }\n");
                } else if line.contains("=") && line.contains(",") && line.contains(":")==false && line.contains("=") {
                    // tuple assignment like `a, b = 0, 1`
                    if line.contains("=") && line.contains(",") && line.split('=').count() == 2 {
                        let left = line.split('=').nth(0).unwrap().trim();
                        let right = line.split('=').nth(1).unwrap().trim();
                        let left_vars: Vec<&str> = left.split(',').map(|s| s.trim()).collect();
                        let right_vals: Vec<&str> = right.split(',').map(|s| s.trim()).collect();
                        if left_vars.len() == right_vals.len() && left_vars.len() == 2 {
                            // assume ints for now
                            out.push_str(&format!("    let mut {}: i32 = {};\n", left_vars[0], right_vals[0]));
                            out.push_str(&format!("    let mut {}: i32 = {};\n", left_vars[1], right_vals[1]));
                        } else {
                            out.push_str(&format!("    // unhandled assignment: {}\n", line));
                        }
                    }
                    idx += 1;
                } else if line.starts_with("for ") && line.contains("range") {
                    // e.g., for _ in range(2, n + 1):
                    // naive parse
                    if let Some(start) = line.find("range(") {
                        let inner = &line[start + 6..];
                        if let Some(endp) = inner.find(')') {
                            let args = &inner[..endp];
                            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
                            if parts.len() == 2 {
                                let a = parts[0];
                                let b = parts[1];
                                // convert `n + 1` to `n`
                                let b = b.trim_end_matches("+ 1").trim().to_string();
                                out.push_str(&format!("    for _ in {}..={} {{\n", a, b));
                                // gather loop inner
                                idx += 1;
                                if idx < lines.len() {
                                    let inner_line = &lines[idx];
                                    if inner_line.contains("=") && inner_line.contains(",") {
                                        // a, b = b, a + b
                                        // translate to temp pattern
                                        out.push_str(&format!("        let temp = {};\n", inner_line.split('=').nth(1).unwrap().trim().replace(" ", "")));
                                        // crude splits
                                        out.push_str(&format!("        {} = {};\n", "a", "b"));
                                        out.push_str(&format!("        {} = temp;\n", "b"));
                                        idx += 1;
                                    }
                                }
                                out.push_str("    }\n");
                            }
                        }
                    }
                } else if line.starts_with("return ") {
                    // if this is last non-empty statement, make it final expression
                    let mut j = idx + 1;
                    let mut rest_empty = true;
                    while j < lines.len() {
                        if !lines[j].trim().is_empty() { rest_empty = false; break }
                        j += 1;
                    }
                    let expr = line.trim_start_matches("return ").trim();
                    if rest_empty {
                        out.push_str(&format!("    {}\n", expr));
                    } else {
                        out.push_str(&format!("    return {};\n", expr));
                    }
                    idx += 1;
                } else if line.starts_with("print(") {
                    // print(x)
                    let inner = line.trim_start_matches("print(").trim_end_matches(")");
                    out.push_str(&format!("    println!(\"{}\", {});\n", "{}", inner));
                    idx += 1;
                } else if line.trim().is_empty() {
                    idx += 1;
                } else {
                    // fallback: copy line as comment
                    out.push_str(&format!("    // TODO: handle `{}`\n", line));
                    idx += 1;
                }
            }

            out.push_str("}\n");
            // done first function
            break;
        }
    }

    out
}
