use crate::walker::Ueg;

// Minimal Zig emitter scaffold. Emits a Zig fn with body comments.

pub fn lower_to_zig(ueg: &Ueg) -> String {
    let mut out = String::new();
    out.push_str("const std = @import(\"std\");\n\n");
    for n in &ueg.nodes {
        if let crate::walker::NodeKind::Lambda(l) = n {
            out.push_str(&format!("pub fn {}() void {{\n", l.name));
            for line in &l.body {
                out.push_str(&format!("    // {}\n", line));
            }
            out.push_str("}\n");
        }
    }
    out
}
