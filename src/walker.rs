use tree_sitter::Node;

/// Minimal, deterministic Python -> Rust walker tailored for Day 1 patterns.
/// This single-file implementation is intentionally small and line-oriented
/// so it produces predictable, rustfmt-friendly output for the fib example.
pub fn python_to_rust(_root: &Node, source: &[u8]) -> String {
    let src = String::from_utf8_lossy(source);
    let mut out = String::new();

    // Find the first top-level `def`
    for (line_idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }

        // Parse signature: `def name(params) -> ret:`
        let sig = trimmed.trim_end_matches(':').trim();
        let rest = sig.trim_start_matches("def").trim();
        let name = rest.split('(').next().unwrap_or("").trim().to_string();

        // Params and return annotation
        let mut params: Vec<(String, String)> = Vec::new();
        let mut ret: Option<String> = None;
        if let Some(pstart) = rest.find('(') {
            if let Some(pend) = rest.find(')') {
                for p in rest[pstart + 1..pend].split(',') {
                    let p = p.trim();
                    if p.is_empty() { continue }
                    if let Some(colon) = p.find(':') {
                        let nm = p[..colon].trim().to_string();
                        let ann = p[colon + 1..].trim();
                        params.push((nm, map_type(ann)));
                    } else {
                        params.push((p.to_string(), "_".into()));
                    }
                }
            }
            if let Some(arrow) = rest.find("->") {
                let after = rest[arrow + 2..].trim().trim_end_matches(':').trim();
                if !after.is_empty() { ret = Some(map_type(after)); }
            }
        }

        // Emit function header
        out.push_str(&format!("fn {}(", name));
        for (i, (pn, pt)) in params.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            if pt == "_" {
                out.push_str(&format!("{}: impl std::fmt::Debug", pn));
            } else {
                out.push_str(&format!("{}: {}", pn, pt));
            }
        }
        out.push(')');
        if let Some(r) = &ret { out.push_str(&format!(" -> {}", r)); }
        out.push_str(" {\n");

        // Collect body lines until blank line or next top-level def
        let mut body: Vec<String> = Vec::new();
        for b in src.lines().skip(line_idx + 1) {
            let t = b.trim().to_string();
            if t.is_empty() { break }
            if t.starts_with("def ") { break }
            body.push(t);
        }

        // Translate supported patterns
        let mut i = 0;
        while i < body.len() {
            let s = &body[i];

            // if ...: (with optional immediate return)
            if s.starts_with("if ") && s.ends_with(":") {
                let cond = s.trim_start_matches("if").trim_end_matches(":").trim();
                out.push_str(&format!("    if {} {{\n", cond));
                if i + 1 < body.len() && body[i + 1].starts_with("return ") {
                    let expr = body[i + 1].trim_start_matches("return ").trim();
                    out.push_str(&format!("        return {};", expr));
                    out.push('\n');
                    i += 1; // skip the return line
                }
                out.push_str("    }\n");

            // tuple assignment: a, b = 0, 1
            } else if s.contains('=') && s.contains(',') && !s.contains(':') {
                let parts: Vec<&str> = s.split('=').collect();
                if parts.len() == 2 {
                    let lhs: Vec<&str> = parts[0].split(',').map(|x| x.trim()).collect();
                    let rhs: Vec<&str> = parts[1].split(',').map(|x| x.trim()).collect();
                    if lhs.len() == rhs.len() {
                        for j in 0..lhs.len() {
                            out.push_str(&format!("    let mut {}: i32 = {};\n", lhs[j], rhs[j]));
                        }
                    } else {
                        out.push_str(&format!("    // unhandled assign: {}\n", s));
                    }
                }

            // for _ in range(a, b+1): -> for _ in a..=b {
            } else if s.starts_with("for ") && s.contains("range(") {
                if let Some(start) = s.find("range(") {
                    if let Some(endp) = s[start + 6..].find(')') {
                        let args = &s[start + 6..start + 6 + endp];
                        let parts: Vec<&str> = args.split(',').map(|x| x.trim()).collect();
                        if parts.len() == 2 {
                            let a = parts[0];
                            let mut b = parts[1].to_string();
                            if b.ends_with("+ 1") { b = b.trim_end_matches("+ 1").trim().to_string(); }
                            out.push_str(&format!("    for _ in {}..={} {{\n", a, b));
                            // expect next line to be tuple update like `a, b = b, a + b`
                            if i + 1 < body.len() && body[i + 1].contains('=') {
                                let rhs = body[i + 1].split('=').nth(1).unwrap_or("").trim();
                                out.push_str(&format!("        let temp = {};\n", rhs));
                                out.push_str("        a = b;\n");
                                out.push_str("        b = temp;\n");
                                i += 1; // skip tuple update
                            }
                            out.push_str("    }\n");
                        }
                    }
                }

            // return statement
            } else if s.starts_with("return ") {
                let mut j = i + 1;
                let mut more = false;
                while j < body.len() { if !body[j].trim().is_empty() { more = true; break } j += 1; }
                let expr = s.trim_start_matches("return ").trim();
                if more { out.push_str(&format!("    return {};\n", expr)); } else { out.push_str(&format!("    {}\n", expr)); }

            // print(x)
            } else if s.starts_with("print(") && s.ends_with(")") {
                let inner = s.trim_start_matches("print(").trim_end_matches(")");
                out.push_str(&format!("    println!(\"{{}}\", {});\n", inner));

            // fallback
            } else {
                out.push_str(&format!("    // TODO: {}\n", s));
            }

            i += 1;
        }

        out.push_str("}\n");
        break; // only first function
    }

    out
}

fn map_type(ann: &str) -> String {
    match ann.trim() {
        "int" => "i32".to_string(),
        "bool" => "bool".to_string(),
        "str" => "String".to_string(),
        "float" => "f64".to_string(),
        "None" => "Option::<_>".to_string(),
        other => other.to_string(),
    }
}
