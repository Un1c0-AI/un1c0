use crate::walker::Ueg;

// Minimal UEG -> Python lowering stub for Day 1. Produces a human-readable
// Python function from a Lambda node. This is intentionally simple: it
// emits the stored body lines and parameter list.

pub fn lower_to_python(ueg: &Ueg) -> String {
    let mut out = String::new();
    for n in &ueg.nodes {
        match n {
            crate::walker::NodeKind::Lambda(l) => {
                out.push_str(&format!("def {}(", l.name));
                for (i, (pn, _pt)) in l.params.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(pn);
                }
                out.push_str("):");
                out.push('\n');
                for line in &l.body {
                    out.push_str("    ");
                    // naive re-conversion: remove Rust-isms like `let mut` and `println!`
                    let python_line = line.replace("let mut ", "").replace(": i32 =", "=").replace("println!(\"{ }\",", "print(");
                    out.push_str(&python_line);
                    out.push('\n');
                }
            }
            _ => {}
        }
    }
    out
}
