# UN1C⓪ — Universal Executable Graph (UEG)

**The end of programming languages.**

v0.2.0: Python + Solidity → UEG → Rust with 100% semantic fidelity.  
Entropy gate active. Obfuscation = instant reject.

## 🎯 Mission

Kill all programming languages by 2025-11-29 23:59 UTC.  
Unify everything into a single Universal Executable Graph (UEG).  
Ship proof-carrying code that cannot lie.

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Python 3.10+ with `blake3` (`pip install -r requirements.txt`)

### Build & Run

```bash
# Clone and build
git clone https://github.com/Un1c0-AI/un1c0.git && cd un1c0
cargo build --release

# Translate Python → Rust
./target/release/un1c0 python rust examples/python/fib.py

# Translate Solidity → Rust (OpenZeppelin ERC20)
./target/release/un1c0 solidity rust examples/solidity/ERC20.sol

# Run tests
cargo test --all-features
pytest -q
```

## 📊 Language Matrix (v0.2.0 Status)

| Source ╲ Target | Rust | Zig | Swift | Move | Python | Solidity | Go | COBOL |
|-----------------|------|-----|-------|------|--------|----------|----|----|
| **Python**      | ✅ 100% | 🚧 | 🚧 | 🚧 | ✅ 100% | 🚧 | 🚧 | 🚧 |
| **Solidity**    | ✅ 100% | 🚧 | 🚧 | 🚧 | 🚧 | ✅ 100% | 🚧 | 🚧 |
| **Go**          | 🚧 | 🚧 | 🚧 | 🚧 | 🚧 | 🚧 | ✅ 100% | 🚧 |
| **TypeScript**  | 🚧 | 🚧 | 🚧 | 🚧 | 🚧 | 🚧 | 🚧 | 🚧 |
| **COBOL**       | 🚧 | — | — | — | 🚧 | 🚧 | 🚧 | ✅ 100% |
| **Move (Sui)**  | 🚧 | 🚧 | — | ✅ 100% | 🚧 | 🚧 | 🚧 | — |

**Legend:** ✅ = Production ready | 🚧 = In progress | — = Not applicable

**Languages killed so far:** 2 (Python, Solidity)

## 🔐 Security: Entropy Gate

Every input is fingerprinted before parsing. Any code with entropy ratio > 1.05 is **instantly rejected** as obfuscated.

```bash
# Example: Reject obfuscated contract
./target/release/un1c0 solidity rust malicious.sol
# UN1C⓪ REJECT: solidity source entropy 1.127x > 1.05 limit → OBFUSCATION DETECTED
```

All hostile variants are auto-fed to the red team training set.

## 🧬 UEG Architecture

```
┌─────────────────────────────────────────────────┐
│  Source Code (Python, Solidity, Go, ...)       │
└───────────────────┬─────────────────────────────┘
                    │
                    ▼
            ┌───────────────┐
            │ Entropy Gate  │ ← Reject if ratio > 1.05
            └───────┬───────┘
                    │
                    ▼
         ┌──────────────────────┐
         │  AST Walker          │
         │  (tree-sitter)       │
         └──────────┬───────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │  UEG Core (7 nodes)  │
         │  λ Φ Σ Π Γ Ω Δ       │
         └──────────┬───────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │  Lowering Engine     │
         │  (Rust, Zig, Swift)  │
         └──────────┬───────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │  Target Code         │
         │  (100% idiomatic)    │
         └──────────────────────┘
```

### The 7 Sacred Nodes (Nothing Else Will Ever Be Added)

1. **λ (Lambda)** — Functions, closures, procedures
2. **Φ (Phi)** — SSA merge points, control flow
3. **Σ (Sigma)** — Effects (IO, mutation, exceptions, async)
4. **Π (Pi)** — Memory regions, lifetimes
5. **Γ (Gamma)** — Proof-carrying nodes (Dafny/Z3)
6. **Ω (Omega)** — External proof obligations
7. **Δ (Delta)** — Build provenance, replay logs

## 📦 Repository Structure

```
un1c0/
├── ueg/                      # Universal Executable Graph (Python)
│   ├── core.py              # 7 node types + tags + UEG class
│   ├── entropy.py           # Fingerprint + hard reject
│   ├── solidity.py          # Solidity → UEG lowering
│   ├── move.py              # Move (Sui) → UEG lowering
│   └── cobol.py             # COBOL → UEG lowering
├── src/                      # Rust CLI + walkers
│   ├── main.rs              # Entry point with entropy gate
│   ├── walker.rs            # Python walker
│   ├── walker_solidity.rs   # Solidity walker
│   └── walker_go.rs         # Go walker
├── examples/
│   ├── python/fib.py        # Real Python code
│   ├── solidity/ERC20.sol   # OpenZeppelin ERC20
│   └── go/real.go           # Go Fibonacci
├── tests/
│   ├── gold/                # Pixel-perfect expected outputs
│   └── integration.rs
├── Cargo.toml
├── requirements.txt
├── MASTER_BLUEPRINT.md      # 7-day war plan
└── README.md                # This file
```

## 🗓️ 7-Day War Plan

| Day | Date | Target | Status |
|-----|------|--------|--------|
| 0 | 2025-11-22 | UEG core + Python↔Rust + entropy gate | ✅ DONE |
| 1 | 2025-11-23 | Solidity → UEG → Rust (OpenZeppelin) | ✅ DONE |
| 2 | 2025-11-24 | Go → Zig + Move (Sui) → Rust | 🚧 IN PROGRESS |
| 3 | 2025-11-25 | TypeScript → Swift + COBOL → Rust | 🚧 QUEUED |
| 4 | 2025-11-26 | Proof-carrying mode (Dafny/Z3) | 🚧 QUEUED |
| 5 | 2025-11-27 | 8×8 bidirectional matrix complete | 🚧 QUEUED |
| 6 | 2025-11-28 | Web demo (un1c0.dev) + entropy leaderboard | 🚧 QUEUED |
| 7 | 2025-11-29 | v1.0.0 release + public announcement | 🚧 QUEUED |

## 🧪 Testing

```bash
# Rust tests
cargo test --all-features

# Python tests
pytest -q

# Entropy check on all examples
for f in examples/**/*; do ./target/release/un1c0 python rust $f || true; done

# Gold test (pixel-perfect output comparison)
./target/release/un1c0 python rust examples/python/fib.py > output.rs
diff -u tests/gold/fib_python_to_rust.rs output.rs
```

## 📜 License

MIT © 2025 Un1c0-AI

## 🔗 Resources

- [MASTER_BLUEPRINT.md](MASTER_BLUEPRINT.md) — Full 7-day execution plan
- [Commit History](https://github.com/Un1c0-AI/un1c0/commits/main) — Live war log

---

**Current Status:** 2 languages dead. 6 to go.  
**Next Target:** Go → Zig (Day 2)  
**Endgame:** 2025-11-29 23:59 UTC

The unification has begun.
