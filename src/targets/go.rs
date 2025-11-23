use crate::walker::Ueg;

// Very small Go emitter scaffold. Produces a Go function signature and body
// from a Lambda node; body lines are emitted as comments or simple translations.

pub fn lower_to_go(ueg: &Ueg) -> String {
    let mut out = String::new();
    out.push_str("package main\n\n");
    for n in &ueg.nodes {
        if let crate::walker::NodeKind::Lambda(l) = n {
            out.push_str(&format!("func {}() {{\n", l.name));
            for line in &l.body {
                out.push_str(&format!("    // {}
", line));
            }
            out.push_str("}\n");
        }
    }
    out
}
