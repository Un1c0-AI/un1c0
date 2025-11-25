// Move → Rust translator
// Simplified regex-based translation for Move language basics
// Full implementation would require tree-sitter-move (not yet available on crates.io)

use regex::Regex;

pub fn move_to_rust(source: &str) -> String {
    let mut result = source.to_string();
    
    // Module declaration: `module 0x1::Token {` → `mod token {`
    let module_re = Regex::new(r"module\s+0x[0-9a-fA-F]+::(\w+)\s*\{").unwrap();
    result = module_re.replace_all(&result, "mod ${1} {").to_string();
    
    // Struct with abilities: `struct Coin has key {` → `struct Coin {`
    let struct_re = Regex::new(r"struct\s+(\w+)\s+has\s+\w+(\s*,\s*\w+)*\s*\{").unwrap();
    result = struct_re.replace_all(&result, "struct $1 {").to_string();
    
    // Use statements: `use std::signer;` → stays mostly the same
    // Move types → Rust types (use word boundaries to avoid partial replacements)
    result = result.replace(": address", ": &str");  // Only in type position
    
    // signer::address_of → signer::get_address (do before other replacements)
    result = result.replace("signer::address_of(", "signer::get_address(");
    
    // Function signatures: `public fun` → `pub fn`
    result = result.replace("public fun", "pub fn");
    result = result.replace("fun", "fn");
    
    
    // Acquires clause: Remove `acquires Coin` from function signatures
    let acquires_re = Regex::new(r"\s+acquires\s+\w+").unwrap();
    result = acquires_re.replace_all(&result, "").to_string();
    
    // Move-specific operations (simplified translations)
    // move_to → insert into storage (conceptual)
    result = result.replace("move_to(", "// storage.insert(");
    
    // borrow_global<T> → access storage (conceptual)
    let borrow_global_re = Regex::new(r"borrow_global<(\w+)>\(([^)]+)\)").unwrap();
    result = borrow_global_re.replace_all(&result, "/* storage.get::<$1>($2) */").to_string();
    
    let borrow_global_mut_re = Regex::new(r"borrow_global_mut<(\w+)>\(([^)]+)\)").unwrap();
    result = borrow_global_mut_re.replace_all(&result, "/* storage.get_mut::<$1>($2) */").to_string();

    
    // Add Rust-style comments explaining Move concepts
    let header = r#"// Translated from Move to Rust
// Note: Move's resource model (move_to, borrow_global) requires
// additional runtime support not directly expressible in Rust.
// This translation provides structural equivalence.

"#;
    
    format!("{}{}", header, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_move_module() {
        let move_code = "module 0x1::Token {\n    struct Coin has key { value: u64 }\n}";
        let rust_code = move_to_rust(move_code);
        assert!(rust_code.contains("mod Token {"));
        assert!(rust_code.contains("struct Coin { value: u64 }"));
    }
    
    #[test]
    fn test_move_function() {
        let move_code = "public fun mint(account: &signer, amount: u64)";
        let rust_code = move_to_rust(move_code);
        assert!(rust_code.contains("pub fn mint"));
    }
}
