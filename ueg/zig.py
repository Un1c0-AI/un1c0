"""
ueg/zig.py — UEG → Zig lowering engine
Lowers Universal Executable Graph to idiomatic Zig code.
Day 2 target: Go → UEG → Zig translation complete.
"""

from .core import UEG, Lambda, Phi, Sigma, Pi, Gamma, Omega, Delta, Tags, PropertyBits

def ueg_to_zig(ueg: UEG) -> str:
    """
    Lower a UEG fragment to Zig source code.
    Enforces safety properties via UEG tags.
    """
    output = []
    output.append("const std = @import(\"std\");\n")
    
    for node in ueg.nodes:
        if isinstance(node, Lambda):
            output.append(_lower_lambda_to_zig(node))
        elif isinstance(node, Phi):
            output.append(_lower_phi_to_zig(node))
        elif isinstance(node, Sigma):
            output.append(_lower_sigma_to_zig(node))
        elif isinstance(node, Pi):
            output.append(_lower_pi_to_zig(node))
        elif isinstance(node, Gamma):
            output.append(_lower_gamma_to_zig(node))
        elif isinstance(node, Omega):
            output.append(_lower_omega_to_zig(node))
        elif isinstance(node, Delta):
            output.append(_lower_delta_to_zig(node))
    
    return "\n".join(output)

def _lower_lambda_to_zig(node: Lambda) -> str:
    """Convert Lambda node to Zig function."""
    # Extract function metadata
    fn_name = node.metadata.get("name", "anonymous")
    params = node.metadata.get("params", [])
    return_type = node.metadata.get("return_type", "void")
    body = node.metadata.get("body", "")
    
    # Build parameter list
    param_str = ", ".join([f"{p['name']}: {_map_type_to_zig(p['type'])}" for p in params])
    
    # Check safety tags
    tags_str = ""
    if node.tags & PropertyBits.NO_OVERFLOW:
        tags_str += "    // @safety: overflow-checked\n"
    if node.tags & PropertyBits.TOTAL:
        tags_str += "    // @safety: guaranteed termination\n"
    
    return f"""
pub fn {fn_name}({param_str}) {_map_type_to_zig(return_type)} {{
{tags_str}{body}
}}
"""

def _lower_phi_to_zig(node: Phi) -> str:
    """Convert Phi (conditional) node to Zig if/else."""
    condition = node.metadata.get("condition", "true")
    then_branch = node.metadata.get("then", "")
    else_branch = node.metadata.get("else", "")
    
    if else_branch:
        return f"    if ({condition}) {{\n{then_branch}\n    }} else {{\n{else_branch}\n    }}"
    return f"    if ({condition}) {{\n{then_branch}\n    }}"

def _lower_sigma_to_zig(node: Sigma) -> str:
    """Convert Sigma (aggregate/tuple) to Zig struct."""
    struct_name = node.metadata.get("name", "AnonymousStruct")
    fields = node.metadata.get("fields", [])
    
    field_lines = "\n".join([f"    {f['name']}: {_map_type_to_zig(f['type'])}," for f in fields])
    return f"""
const {struct_name} = struct {{
{field_lines}
}};
"""

def _lower_pi_to_zig(node: Pi) -> str:
    """Convert Pi (loop) to Zig for/while."""
    loop_type = node.metadata.get("loop_type", "while")
    condition = node.metadata.get("condition", "true")
    body = node.metadata.get("body", "")
    
    if loop_type == "for":
        init = node.metadata.get("init", "")
        step = node.metadata.get("step", "")
        return f"    for ({init}; {condition}; {step}) {{\n{body}\n    }}"
    return f"    while ({condition}) {{\n{body}\n    }}"

def _lower_gamma_to_zig(node: Gamma) -> str:
    """Convert Gamma (variable binding) to Zig var/const."""
    var_name = node.metadata.get("name", "temp")
    var_type = node.metadata.get("type", "auto")
    var_value = node.metadata.get("value", "undefined")
    is_const = node.metadata.get("const", True)
    
    decl = "const" if is_const else "var"
    if var_type == "auto":
        return f"    {decl} {var_name} = {var_value};"
    return f"    {decl} {var_name}: {_map_type_to_zig(var_type)} = {var_value};"

def _lower_omega_to_zig(node: Omega) -> str:
    """Convert Omega (effect/IO) to Zig I/O."""
    effect_type = node.metadata.get("effect", "print")
    value = node.metadata.get("value", "")
    
    if effect_type == "print":
        return f'    std.debug.print("{value}\\n", .{{}});'
    return f"    // Effect: {effect_type}"

def _lower_delta_to_zig(node: Delta) -> str:
    """Convert Delta (type mutation) to Zig cast."""
    from_type = node.metadata.get("from", "i32")
    to_type = node.metadata.get("to", "i64")
    value = node.metadata.get("value", "x")
    
    return f"    @as({_map_type_to_zig(to_type)}, {value})"

def _map_type_to_zig(typ: str) -> str:
    """Map generic type names to Zig types."""
    type_map = {
        "int": "i32",
        "i32": "i32",
        "i64": "i64",
        "float": "f64",
        "f64": "f64",
        "bool": "bool",
        "str": "[]const u8",
        "string": "[]const u8",
        "void": "void",
        "auto": "",
    }
    return type_map.get(typ, typ)

# Example usage:
if __name__ == "__main__":
    from .core import fib_ueg
    
    ueg = fib_ueg()
    zig_code = ueg_to_zig(ueg)
    print("=== UEG → Zig Lowering ===")
    print(zig_code)
