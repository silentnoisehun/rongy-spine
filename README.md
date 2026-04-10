# Rongyász — Safe Self-Evolving Rust System

A production-grade, self-modifying Rust system with AST-level mutation,
fitness-driven selection, and immutable safety anchors.

Built by Máté Róbert — a factory worker from Hungary.

---

## What This Is

Most self-modifying systems break themselves. This one doesn't.

Rongyász evolves its own source code at the **AST level** (not string manipulation),
guided by a fitness function, protected by immutable safety anchors.

The system can improve itself — remove unwraps, refactor error handling,
optimize idioms — without ever touching the files that keep it stable.

---

## Three Core Components

### 1. `spine.rs` — Zero-Copy IPC Bus
A 2048-byte memory-mapped shared memory layout for sub-nanosecond
inter-process communication between all system modules.

- Fixed slot offsets for every module (affective state, consciousness, colony, etc.)
- Lock-free ring buffer for inter-module messaging
- Zero serialization overhead — raw binary reads/writes
- ~1.4 ns/op measured on AMD Ryzen 5 5600X

```rust
// Every module reads/writes shared state at nanosecond speed
spine.set_colony_fitness(0.97);
spine.set_tension(0.3);
let phi = spine.phi(); // consciousness metric
```

### 2. `ast_refactor.rs` — Structural Code Transformation
AST-level code mutation using `syn` — not regex, not string replace.

- Parses Rust source into a full syntax tree
- Identifies functions with unsafe patterns (unwrap, panic, clone chains)
- Rewrites return types, converts `.unwrap()` → `?`, wraps with Result
- Structurally valid output guaranteed by the AST

```rust
// Input: fn load(path: &str) -> String { fs::read_to_string(path).unwrap() }
// Output: fn load(path: &str) -> Result<String, Box<dyn Error>> {
//     let result = fs::read_to_string(path)?;
//     Ok(result)
// }
let (improved_code, report) = AstRefactor::refactor_file(&source)?;
```

### 3. `self_evolve.rs` + `safety.rs` — Evolutionary Loop with Anchors
The full evolution cycle: scan → mutate → fitness check → build check → apply or rollback.

Safety anchors (immutable files the system can NEVER touch):
```rust
const IMMUTABLE_FILES: &[&str] = &[
    "safety.rs",        // This file itself
    "main.rs",          // Entry point
    "hope_watchdog.rs", // Ethics layer
    "self_evolve.rs",   // The mutator cannot mutate itself
    // ...
];
```

Full cycle:
```
1. Collect mutable .rs files (excluding anchors)
2. Measure fitness: grade = f(unwrap_count, clone_count, panic_count)  
3. Apply AST mutation
4. cargo check — if build fails: rollback
5. Measure fitness again — if worse: rollback
6. If better: keep, record improvement
7. Git snapshot before every session
```

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    SPINE (mmap)                      │
│  2048 bytes — zero-copy IPC between all modules      │
│  [header][affect][psi][consciousness][colony][ring]  │
└────────────────────┬────────────────────────────────┘
                     │ nanosecond reads/writes
        ┌────────────┼────────────┐
        │            │            │
   ┌────▼────┐  ┌────▼────┐  ┌───▼─────┐
   │  AST    │  │Fitness  │  │ Safety  │
   │Refactor │  │ Engine  │  │  Core   │
   │(syn)    │  │(grade)  │  │(anchors)│
   └────┬────┘  └────┬────┘  └───┬─────┘
        │            │            │
        └────────────▼────────────┘
                ┌────────┐
                │Self    │
                │Evolve  │
                │(loop)  │
                └────────┘
```

---

## Benchmarks

Measured on AMD Ryzen 5 5600X, 16GB RAM, `--release` build:

| Operation | Speed |
|-----------|-------|
| Spine f32 read/write | ~1.4 ns/op |
| Full tick cycle | ~31.5 ns/op |
| AST parse + mutate (avg file) | ~2-8 ms |
| cargo check (incremental) | ~800ms-2s |

---

## The Key Insight

The system cannot break itself because:

1. **AST guarantees syntactic validity** — mutations produce valid Rust or fail
2. **cargo check is the ground truth** — invalid mutations are rolled back
3. **Fitness gate** — regressions are rolled back even if they compile
4. **Immutable anchors** — the files that control evolution cannot be evolved

This is structural safety, not prompt-based safety.

---

## Installation

```bash
git clone https://github.com/silentnoisehun/rongy-spine
cd rongy-spine
cargo build --release
cargo test
```

Dependencies: `syn`, `quote`, `proc-macro2`, `rand`, `tracing`, `serde`

---

## License

MIT — Use it, build on it, make it better.

---

## Author

Máté Róbert — Factory worker, Mosonmagyaróvár, Hungary.

> "The bottleneck was never compute. It was architecture."

Part of the Rongyász project: a self-evolving AI system built during
night shifts, without a PhD, without venture capital.
