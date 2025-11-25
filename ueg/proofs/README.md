# UEG Proof Artifacts

This directory contains formal verification artifacts for UN1C⓪DE proof-carrying UEG.

## Structure

- `*.smt2` - Z3 SMT-LIB format proof files
- `*.dfy` - Dafny verification source files
- `manifest.json` - Proof metadata and hashes

## Verification Properties

Every UEG node carries embedded proofs for:

1. **NO_OVERFLOW** - Arithmetic operations never exceed type bounds
2. **TERMINATING** - All loops/recursion provably terminate
3. **CONSTANT_TIME** - No secret-dependent branches (optional, for crypto)

## Usage

Proofs are automatically verified when using `--prove` flag:

```bash
./target/release/un1c0 zig rust examples/zig/comptime.zig --prove
```

## Version

- UEG v0.9.0
- Z3 4.12.2
- Dafny 4.8.0 (planned)
