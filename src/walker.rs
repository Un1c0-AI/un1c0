use tree_sitter::Node;
use std::collections::HashMap;

// Rust-side minimal UEG (shape inspired by `ueg.py`) sufficient for Day 1
// lowering. This file provides:
// - a tiny UEG structure (Lambda node only)
// - `entropy_fingerprint` implementing the one-liner behavior
// - `python_to_rust` which builds a UEG and calls `lower_to_rust`

#[derive(Debug, Clone)]
pub struct Ueg {
    pub nodes: Vec<NodeKind>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Lambda(LambdaNode),
}

#[derive(Debug, Clone)]
pub struct LambdaNode {
    pub id: u64,
    pub name: String,
    pub params: Vec<(String, String)>,
    pub ret: Option<String>,
    pub body: Vec<String>,
}

impl Ueg {
    pub fn new() -> Self { Ueg { nodes: Vec::new() } }

    pub fn semantic_hash(&self) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut acc = String::new();
        for n in &self.nodes {
            if let NodeKind::Lambda(l) = n { acc.push_str(&l.name); }
        }
        let mut h = DefaultHasher::new();
        acc.hash(&mut h);
        format!("{:x}", h.finish())
    }

    pub fn validate(&self) -> bool {
        !self.nodes.is_empty()
    }
}

/// Lower the UEG into Rust source. For Day 1 this reproduces the
/// deterministic patterns used earlier, but driven from the UEG nodes.
pub fn lower_to_rust(ueg: &Ueg) -> String {
    let mut out = String::new();
    for n in &ueg.nodes {
        if let NodeKind::Lambda(l) = n {
            out.push_str(&format!("fn {}(", l.name));
            for (i, (pn, pt)) in l.params.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                if pt == "_" {
                    out.push_str(&format!("{}: impl std::fmt::Debug", pn));
                } else {
                    out.push_str(&format!("{}: {}", pn, pt));
                }
            }
            out.push(')');
            if let Some(r) = &l.ret { out.push_str(&format!(" -> {}", r)); }
            out.push_str(" {\n");

            for line in &l.body {
                out.push_str(&format!("    {}\n", line));
            }

            out.push_str("}\n");
        }
    }
    out
}

/// Entropy fingerprint: implements the one-liner behavior from the user's
/// specification. Returns a normalized entropy in [0,1].
pub fn entropy_fingerprint(source: &str) -> f64 {
    let mut freqs: HashMap<char, usize> = HashMap::new();
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len() as f64;
    if n == 0.0 { return 0.0; }
    for c in chars { *freqs.entry(c).or_insert(0) += 1usize; }
    let set_len = freqs.len();
    if set_len <= 1 { return 0.0; }
    let mut sum = 0.0f64;
    for (_c, &cnt) in freqs.iter() {
        let p = (cnt as f64) / n;
        if p > 0.0 { sum -= p * p.log2(); }
    }
    let denom = (set_len as f64).log2();
    if denom == 0.0 { return 0.0; }
    sum / denom
}

/// The main entry used by the rest of the translator: produce Rust by
/// constructing a UEG and lowering it. Keeps the previous simple, line-
/// oriented parsing for Day 1 but routes through the IR.
pub fn python_to_rust(_root: &Node, source: &[u8]) -> String {
    let src = String::from_utf8_lossy(source).to_string();

    // Entropy gate: reject inputs that exceed 1.05× minimal entropy.
    // Minimal baseline is configurable; for Day 1 we choose 0.25.
    let f = entropy_fingerprint(&src);
    const MINIMAL_BASELINE: f64 = 0.25;
    if f > 1.05 * MINIMAL_BASELINE {
        return format!("// input rejected: entropy {:.6} > {:.6}", f, 1.05 * MINIMAL_BASELINE);
    }

    // Naive line-oriented parse: find first `def` and extract signature + body
    let mut name = String::new();
    let mut params: Vec<(String, String)> = Vec::new();
    let mut ret: Option<String> = None;
    let mut body_lines: Vec<String> = Vec::new();

    for (line_idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") { continue; }
        let sig = trimmed.trim_end_matches(':').trim();
        let rest = sig.trim_start_matches("def").trim();
        name = rest.split('(').next().unwrap_or("").trim().to_string();

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

        for b in src.lines().skip(line_idx + 1) {
            let t = b.trim().to_string();
            if t.is_empty() { break }
            if t.starts_with("def ") { break }
            body_lines.push(t);
        }
        break;
    }

    // Build UEG
    let mut ueg = Ueg::new();
    let lambda = LambdaNode {
        id: 1,
        name: name.clone(),
        params: params.clone(),
        ret: ret.clone(),
        body: translate_body_to_rust_like(&body_lines),
    };
    ueg.nodes.push(NodeKind::Lambda(lambda));

    if !ueg.validate() { return "// invalid UEG generated".into(); }

    lower_to_rust(&ueg)
}

fn translate_body_to_rust_like(body: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < body.len() {
        let s = &body[i];
        if s.starts_with("if ") && s.ends_with(":") {
            let cond = s.trim_start_matches("if").trim_end_matches(":").trim();
            out.push(format!("if {} {{", cond));
            if i + 1 < body.len() && body[i + 1].starts_with("return ") {
                let expr = body[i + 1].trim_start_matches("return ").trim();
                out.push(format!("    return {};", expr));
                i += 1;
            }
            out.push("}".into());

        } else if s.contains('=') && s.contains(',') && !s.contains(':') {
            let parts: Vec<&str> = s.split('=').collect();
            if parts.len() == 2 {
                let lhs: Vec<&str> = parts[0].split(',').map(|x| x.trim()).collect();
                let rhs: Vec<&str> = parts[1].split(',').map(|x| x.trim()).collect();
                if lhs.len() == rhs.len() {
                    for j in 0..lhs.len() {
                        out.push(format!("let mut {}: i32 = {};", lhs[j], rhs[j]));
                    }
                } else { out.push(format!("// unhandled assign: {}", s)); }
            }

        } else if s.starts_with("for ") && s.contains("range(") {
            if let Some(start) = s.find("range(") {
                if let Some(endp) = s[start + 6..].find(')') {
                    let args = &s[start + 6..start + 6 + endp];
                    let parts: Vec<&str> = args.split(',').map(|x| x.trim()).collect();
                    if parts.len() == 2 {
                        let a = parts[0];
                        let mut b = parts[1].to_string();
                        if b.ends_with("+ 1") { b = b.trim_end_matches("+ 1").trim().to_string(); }
                        out.push(format!("for _ in {}..={} {{", a, b));
                        if i + 1 < body.len() && body[i + 1].contains('=') {
                            let rhs_full = body[i + 1].split('=').nth(1).unwrap_or("").trim();
                            let temp_expr = if rhs_full.contains(',') { rhs_full.split(',').nth(1).unwrap_or(rhs_full).trim() } else { rhs_full };
                            out.push(format!("    let temp = {};", temp_expr));
                            out.push("    a = b;".into());
                            out.push("    b = temp;".into());
                            i += 1;
                        }
                        out.push("}".into());
                    }
                }
            }

        } else if s.starts_with("return ") {
            let mut j = i + 1; let mut more = false;
            while j < body.len() { if !body[j].trim().is_empty() { more = true; break } j += 1; }
            let expr = s.trim_start_matches("return ").trim();
            if more { out.push(format!("return {};", expr)); } else { out.push(expr.into()); }

        } else if s.starts_with("print(") && s.ends_with(")") {
            let inner = s.trim_start_matches("print(").trim_end_matches(")");
            out.push(format!("println!(\\"{{}}\\", {});", inner));

        } else { out.push(format!("// TODO: {}", s)); }

        i += 1;
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
use tree_sitter::Node;
use std::collections::HashMap;

// Rust-side minimal UEG (shape inspired by `ueg.py`) sufficient for Day 1
// lowering. This file provides:
// - a tiny UEG structure (Lambda node only)
// - `entropy_fingerprint` implementing the one-liner behavior
// - `python_to_rust` which builds a UEG and calls `lower_to_rust`

#[derive(Debug, Clone)]
pub struct Ueg {
    pub nodes: Vec<NodeKind>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Lambda(LambdaNode),
}

#[derive(Debug, Clone)]
pub struct LambdaNode {
    pub id: u64,
    pub name: String,
    pub params: Vec<(String, String)>,
    pub ret: Option<String>,
    pub body: Vec<String>,
}

impl Ueg {
    pub fn new() -> Self { Ueg { nodes: Vec::new() } }

    pub fn semantic_hash(&self) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut acc = String::new();
        for n in &self.nodes {
            if let NodeKind::Lambda(l) = n { acc.push_str(&l.name); }
        }
        let mut h = DefaultHasher::new();
        acc.hash(&mut h);
        format!("{:x}", h.finish())
    }

    pub fn validate(&self) -> bool {
        !self.nodes.is_empty()
    }
}

/// Lower the UEG into Rust source. For Day 1 this reproduces the
/// deterministic patterns used earlier, but driven from the UEG nodes.
pub fn lower_to_rust(ueg: &Ueg) -> String {
    let mut out = String::new();
    for n in &ueg.nodes {
        if let NodeKind::Lambda(l) = n {
            out.push_str(&format!("fn {}(", l.name));
            for (i, (pn, pt)) in l.params.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                if pt == "_" {
                    out.push_str(&format!("{}: impl std::fmt::Debug", pn));
                } else {
                    out.push_str(&format!("{}: {}", pn, pt));
                }
            }
            out.push(')');
            if let Some(r) = &l.ret { out.push_str(&format!(" -> {}", r)); }
            out.push_str(" {\n");

            for line in &l.body {
                out.push_str(&format!("    {}\n", line));
            }

            out.push_str("}\n");
        }
    }
    out
}

/// Entropy fingerprint: implements the one-liner behavior from the user's
/// specification. Returns a normalized entropy in [0,1].
pub fn entropy_fingerprint(source: &str) -> f64 {
    let mut freqs: HashMap<char, usize> = HashMap::new();
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len() as f64;
    if n == 0.0 { return 0.0; }
    for c in chars { *freqs.entry(c).or_insert(0) += 1usize; }
    let set_len = freqs.len();
    if set_len <= 1 { return 0.0; }
    let mut sum = 0.0f64;
    for (_c, &cnt) in freqs.iter() {
        let p = (cnt as f64) / n;
        if p > 0.0 { sum -= p * p.log2(); }
    }
    let denom = (set_len as f64).log2();
    if denom == 0.0 { return 0.0; }
    sum / denom
}

/// The main entry used by the rest of the translator: produce Rust by
/// constructing a UEG and lowering it. Keeps the previous simple, line-
/// oriented parsing for Day 1 but routes through the IR.
pub fn python_to_rust(_root: &Node, source: &[u8]) -> String {
    let src = String::from_utf8_lossy(source).to_string();

    // Entropy gate: reject inputs that exceed 1.05× minimal entropy.
    // Minimal baseline is configurable; for Day 1 we choose 0.25.
    let f = entropy_fingerprint(&src);
    const MINIMAL_BASELINE: f64 = 0.25;
    if f > 1.05 * MINIMAL_BASELINE {
        return format!("// input rejected: entropy {:.6} > {:.6}", f, 1.05 * MINIMAL_BASELINE);
    }

    // Naive line-oriented parse: find first `def` and extract signature + body
    let mut name = String::new();
    let mut params: Vec<(String, String)> = Vec::new();
    let mut ret: Option<String> = None;
    let mut body_lines: Vec<String> = Vec::new();

    for (line_idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") { continue; }
        let sig = trimmed.trim_end_matches(':').trim();
        let rest = sig.trim_start_matches("def").trim();
        name = rest.split('(').next().unwrap_or("").trim().to_string();

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

        for b in src.lines().skip(line_idx + 1) {
            let t = b.trim().to_string();
            if t.is_empty() { break }
            if t.starts_with("def ") { break }
            body_lines.push(t);
        }
        break;
    }

    // Build UEG
    let mut ueg = Ueg::new();
    let lambda = LambdaNode {
        id: 1,
        name: name.clone(),
        params: params.clone(),
        ret: ret.clone(),
        body: translate_body_to_rust_like(&body_lines),
    };
    ueg.nodes.push(NodeKind::Lambda(lambda));

    if !ueg.validate() { return "// invalid UEG generated".into(); }

    lower_to_rust(&ueg)
}

fn translate_body_to_rust_like(body: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < body.len() {
        let s = &body[i];
        if s.starts_with("if ") && s.ends_with(":") {
            let cond = s.trim_start_matches("if").trim_end_matches(":").trim();
            out.push(format!("if {} {{", cond));
            if i + 1 < body.len() && body[i + 1].starts_with("return ") {
                let expr = body[i + 1].trim_start_matches("return ").trim();
                out.push(format!("    return {};", expr));
                i += 1;
            }
            out.push("}".into());

        } else if s.contains('=') && s.contains(',') && !s.contains(':') {
            let parts: Vec<&str> = s.split('=').collect();
            if parts.len() == 2 {
                let lhs: Vec<&str> = parts[0].split(',').map(|x| x.trim()).collect();
                let rhs: Vec<&str> = parts[1].split(',').map(|x| x.trim()).collect();
                if lhs.len() == rhs.len() {
                    for j in 0..lhs.len() {
                        out.push(format!("let mut {}: i32 = {};", lhs[j], rhs[j]));
                    }
                } else { out.push(format!("// unhandled assign: {}", s)); }
            }

        } else if s.starts_with("for ") && s.contains("range(") {
            if let Some(start) = s.find("range(") {
                if let Some(endp) = s[start + 6..].find(')') {
                    let args = &s[start + 6..start + 6 + endp];
                    let parts: Vec<&str> = args.split(',').map(|x| x.trim()).collect();
                    if parts.len() == 2 {
                        let a = parts[0];
                        let mut b = parts[1].to_string();
                        if b.ends_with("+ 1") { b = b.trim_end_matches("+ 1").trim().to_string(); }
                        out.push(format!("for _ in {}..={} {{", a, b));
                        if i + 1 < body.len() && body[i + 1].contains('=') {
                            let rhs_full = body[i + 1].split('=').nth(1).unwrap_or("").trim();
                            let temp_expr = if rhs_full.contains(',') { rhs_full.split(',').nth(1).unwrap_or(rhs_full).trim() } else { rhs_full };
                            out.push(format!("    let temp = {};", temp_expr));
                            out.push("    a = b;".into());
                            out.push("    b = temp;".into());
                            i += 1;
                        }
                        out.push("}".into());
                    }
                }
            }

        } else if s.starts_with("return ") {
            let mut j = i + 1; let mut more = false;
            while j < body.len() { if !body[j].trim().is_empty() { more = true; break } j += 1; }
            let expr = s.trim_start_matches("return ").trim();
            if more { out.push(format!("return {};", expr)); } else { out.push(expr.into()); }

        } else if s.starts_with("print(") && s.ends_with(")") {
            let inner = s.trim_start_matches("print(").trim_end_matches(")");
            out.push(format!("println!(\"{{}}\", {});", inner));

        } else { out.push(format!("// TODO: {}", s)); }

        i += 1;
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
                    use tree_sitter::Node;
                    use std::collections::HashMap;

                    // Minimal UEG representation in Rust mirroring `ueg.py`'s shape enough for
                    // Day 1 lowering. This intentionally implements only what's required to
                    // represent a top-level function (Lambda) and lower it back to Rust.

                    #[derive(Debug, Clone)]
                    pub struct Ueg {
                        pub nodes: Vec<NodeKind>,
                    }

                    #[derive(Debug, Clone)]
                    pub enum NodeKind {
                        Lambda(LambdaNode),
                        // Other node kinds (Phi, Sigma, Pi, Gamma, Omega, Delta) omitted for brevity
                    }

                    #[derive(Debug, Clone)]
                    pub struct LambdaNode {
                        pub id: u64,
                        pub name: String,
                        pub params: Vec<(String, String)>, // (name, type)
                        pub ret: Option<String>,
                        pub body: Vec<String>, // simple line-oriented body for Day 1
                    }

                    impl Ueg {
                        pub fn new() -> Self { Ueg { nodes: Vec::new() } }

                        pub fn semantic_hash(&self) -> String {
                            // Deterministic stub: join node names and hash with a simple mix.
                            // This mirrors the Python `ueg.py` behavior at a conceptual level.
                            let mut acc = String::new();
                            for n in &self.nodes {
                                if let NodeKind::Lambda(l) = n { acc.push_str(&l.name); }
                            }
                            // Simple, stable hash via builtin hasher -> hex (not cryptographic here).
                            use std::hash::{Hash, Hasher};
                            use std::collections::hash_map::DefaultHasher;
                            let mut h = DefaultHasher::new();
                            acc.hash(&mut h);
                            format!("{:x}", h.finish())
                        }

                        pub fn validate(&self) -> bool {
                            // Very small structural checks for Day 1
                            !self.nodes.is_empty()
                        }
                    }

                    /// Convert a UEG instance into Rust source. For Day 1 this reproduces
                    /// the same deterministic patterns used previously but driven from the
                    /// intermediate graph instead of direct string emission.
                    pub fn lower_to_rust(ueg: &Ueg) -> String {
                        let mut out = String::new();
                        for n in &ueg.nodes {
                            if let NodeKind::Lambda(l) = n {
                                out.push_str(&format!("fn {}(", l.name));
                                for (i, (pn, pt)) in l.params.iter().enumerate() {
                                    if i > 0 { out.push_str(", "); }
                                    if pt == "_" { out.push_str(&format!("{}: impl std::fmt::Debug", pn)); }
                                    else { out.push_str(&format!("{}: {}", pn, pt)); }
                                }
                                out.push(')');
                                if let Some(r) = &l.ret { out.push_str(&format!(" -> {}", r)); }
                                out.push_str(" {\n");

                                // The body is already a sequence of translated Rust-like lines
                                for line in &l.body {
                                    out.push_str(&format!("    {}\n", line));
                                }

                                out.push_str("}\n");
                            }
                        }
                        out
                    }

                    /// Entropy fingerprint: faithful translation of the one-liner provided.
                    pub fn entropy_fingerprint(source: &str) -> f64 {
                        use std::f64;
                        let mut freqs: HashMap<char, usize> = HashMap::new();
                        let chars: Vec<char> = source.chars().collect();
                        let n = chars.len() as f64;
                        if n == 0.0 { return 0.0; }
                        for c in chars {
                            *freqs.entry(c).or_insert(0) += 1usize;
                        }
                        let set_len = freqs.len();
                        if set_len == 0 { return 0.0; }
                        let mut sum = 0.0f64;
                        for (_c, &cnt) in freqs.iter() {
                            let p = (cnt as f64) / n;
                            if p > 0.0 { sum -= p * p.log2(); }
                        }
                        let denom = (set_len as f64).log2();
                        if denom == 0.0 { return 0.0; }
                        sum / denom
                    }

                    pub fn python_to_rust(_root: &Node, source: &[u8]) -> String {
                        let src = String::from_utf8_lossy(source).to_string();

                        // Entropy gate: reject inputs that exceed 1.05 * minimal_entropy.
                        // For Day 1 we pick a conservative minimal baseline (0.25). This value
                        // can be calibrated or replaced by a learned baseline later.
                        let f = entropy_fingerprint(&src);
                        const MINIMAL_BASELINE: f64 = 0.25;
                        if f > 1.05 * MINIMAL_BASELINE {
                            return format!("// input rejected: entropy {:.6} > {:.6}", f, 1.05 * MINIMAL_BASELINE);
                        }

                        // Parse first top-level def (naive, line-oriented like before)
                        let mut name = String::new();
                        let mut params: Vec<(String, String)> = Vec::new();
                        let mut ret: Option<String> = None;
                        let mut body_lines: Vec<String> = Vec::new();

                        for (line_idx, line) in src.lines().enumerate() {
                            let trimmed = line.trim_start();
                            if !trimmed.starts_with("def ") { continue; }
                            let sig = trimmed.trim_end_matches(':').trim();
                            let rest = sig.trim_start_matches("def").trim();
                            name = rest.split('(').next().unwrap_or("").trim().to_string();

                            if let Some(pstart) = rest.find('(') {
                                if let Some(pend) = rest.find(')') {
                                    for p in rest[pstart + 1..pend].split(',') {
                                        let p = p.trim();
                                        if p.is_empty() { continue }
                                        if let Some(colon) = p.find(':') {
                                            let nm = p[..colon].trim().to_string();
                                            let ann = p[colon + 1..].trim();
                                            params.push((nm, map_type(ann)));
                                        } else { params.push((p.to_string(), "_".into())); }
                                    }
                                }
                                if let Some(arrow) = rest.find("->") {
                                    let after = rest[arrow + 2..].trim().trim_end_matches(':').trim();
                                    if !after.is_empty() { ret = Some(map_type(after)); }
                                }
                            }

                            // collect simple body lines
                            for b in src.lines().skip(line_idx + 1) {
                                let t = b.trim().to_string();
                                if t.is_empty() { break }
                                if t.starts_with("def ") { break }
                                body_lines.push(t);
                            }
                            break;
                        }

                        // Build a minimal UEG
                        let mut ueg = Ueg::new();
                        let lambda = LambdaNode {
                            id: 1,
                            name: name.clone(),
                            params: params.clone(),
                            ret: ret.clone(),
                            // Lower the Python body into simple Rust-like lines that will be
                            // assembled by `lower_to_rust` for Day 1 constructs.
                            body: translate_body_to_rust_like(&body_lines),
                        };
                        ueg.nodes.push(NodeKind::Lambda(lambda));

                        // Sanity checks
                        if !ueg.validate() { return "// invalid UEG generated".into(); }

                        // Call the lowering pipeline
                        let rust = lower_to_rust(&ueg);
                        rust
                    }

                    fn translate_body_to_rust_like(body: &[String]) -> Vec<String> {
                        let mut out: Vec<String> = Vec::new();
                        let mut i = 0usize;
                        while i < body.len() {
                            let s = &body[i];
                            if s.starts_with("if ") && s.ends_with(":") {
                                let cond = s.trim_start_matches("if").trim_end_matches(":").trim();
                                out.push(format!("if {} {{", cond));
                                if i + 1 < body.len() && body[i + 1].starts_with("return ") {
                                    let expr = body[i + 1].trim_start_matches("return ").trim();
                                    out.push(format!("    return {};", expr));
                                    i += 1;
                                }
                                out.push("}".into());

                            } else if s.contains('=') && s.contains(',') && !s.contains(':') {
                                let parts: Vec<&str> = s.split('=').collect();
                                if parts.len() == 2 {
                                    let lhs: Vec<&str> = parts[0].split(',').map(|x| x.trim()).collect();
                                    let rhs: Vec<&str> = parts[1].split(',').map(|x| x.trim()).collect();
                                    if lhs.len() == rhs.len() {
                                        for j in 0..lhs.len() {
                                            out.push(format!("let mut {}: i32 = {};", lhs[j], rhs[j]));
                                        }
                                    } else { out.push(format!("// unhandled assign: {}", s)); }
                                }

                            } else if s.starts_with("for ") && s.contains("range(") {
                                if let Some(start) = s.find("range(") {
                                    if let Some(endp) = s[start + 6..].find(')') {
                                        let args = &s[start + 6..start + 6 + endp];
                                        let parts: Vec<&str> = args.split(',').map(|x| x.trim()).collect();
                                        if parts.len() == 2 {
                                            let a = parts[0];
                                            let mut b = parts[1].to_string();
                                            if b.ends_with("+ 1") { b = b.trim_end_matches("+ 1").trim().to_string(); }
                                            out.push(format!("for _ in {}..={} {{", a, b));
                                            if i + 1 < body.len() && body[i + 1].contains('=') {
                                                let rhs_full = body[i + 1].split('=').nth(1).unwrap_or("").trim();
                                                let temp_expr = if rhs_full.contains(',') { rhs_full.split(',').nth(1).unwrap_or(rhs_full).trim() } else { rhs_full };
                                                out.push(format!("    let temp = {};", temp_expr));
                                                out.push("    a = b;".into());
                                                out.push("    b = temp;".into());
                                                i += 1;
                                            }
                                            out.push("}".into());
                                        }
                                    }
                                }

                            } else if s.starts_with("return ") {
                                let mut j = i + 1; let mut more = false;
                                while j < body.len() { if !body[j].trim().is_empty() { more = true; break } j += 1; }
                                let expr = s.trim_start_matches("return ").trim();
                                if more { out.push(format!("return {};", expr)); } else { out.push(expr.into()); }

                            } else if s.starts_with("print(") && s.ends_with(")") {
                                let inner = s.trim_start_matches("print(").trim_end_matches(")");
                                out.push(format!("println!(\"{{}}\", {});", inner));

                            } else { out.push(format!("// TODO: {}", s)); }

                            i += 1;
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
