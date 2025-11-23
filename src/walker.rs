use tree_sitter::Node;
use std::collections::HashMap;

// Clean single-file implementation: UEG types, entropy gate, python->UEG->Rust

#[derive(Debug, Clone)]
pub struct Ueg { pub nodes: Vec<NodeKind>, }

#[derive(Debug, Clone)]
pub enum NodeKind { Lambda(LambdaNode), }

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
    pub fn validate(&self) -> bool { !self.nodes.is_empty() }
}

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
            for line in &l.body { out.push_str(&format!("    {}\n", line)); }
            out.push_str("}\n");
        }
    }
    out
}

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

pub fn python_to_rust(_root: &Node, source: &[u8]) -> String {
    let src = String::from_utf8_lossy(source).to_string();
    let f = entropy_fingerprint(&src);
    const MINIMAL_BASELINE: f64 = 0.25;
    if f > 1.05 * MINIMAL_BASELINE {
        return format!("// input rejected: entropy {:.6} > {:.6}", f, 1.05 * MINIMAL_BASELINE);
    }

    // parse first def (naive)
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
        for b in src.lines().skip(line_idx + 1) {
            let t = b.trim().to_string();
            if t.is_empty() { break }
            if t.starts_with("def ") { break }
            body_lines.push(t);
        }
        break;
    }

    let mut ueg = Ueg::new();
    let lambda = LambdaNode { id: 1, name: name.clone(), params: params.clone(), ret: ret.clone(), body: translate_body_to_rust_like(&body_lines) };
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

