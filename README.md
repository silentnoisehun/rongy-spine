<!-- badges:start -->
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.22983073.svg)](https://doi.org/10.5281/zenodo.22983073)
[![ORCID iD](https://img.shields.io/badge/ORCID-0009--0003--3986--6039-green)](https://orcid.org/0009-0003-3986-6039)
<!-- badges:end -->
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
- Sub-nanosecond slot access and ~2.7 ns ring buffer operations

```rust
use rongy_spine::Spine;

let mut spine = Spine::new();
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
use rongy_spine::AstRefactor;

// Input: fn load(path: &str) -> String { fs::read_to_string(path).unwrap() }
// Output: fn load(path: &str) -> Result<String, Box<dyn std::error::Error>> {
//     let content = fs::read_to_string(path)?;
//     Ok(content)
// }
let (improved_code, report) = AstRefactor::refactor_file(&source)?;
```

### 3. `self_evolve.rs` + `safety.rs` — Evolutionary Loop with Anchors
The full evolution cycle: scan → mutate → fitness check → build check → apply or rollback.

Safety anchors (immutable files the system can NEVER touch):
```rust
const IMMUTABLE_FILES: &[&str] = &[
    "safety.rs",        // Safety anchor core
    "spine.rs",         // IPC backbone
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

## Benchmarks & Real Measured Performance

Measured on **4-core Intel Xeon CPU @ 2.30GHz, Linux x86_64**, `--release` build:

| Operation | Measured Performance | Notes |
|-----------|----------------------|-------|
| Spine f32 read/write | ~0.06 ns/op | Pure in-memory slot manipulation |
| Spine full tick cycle | ~33.2 ns/op | Tick count increment + timestamp generation |
| Ring buffer push + pop | ~2.70 ns/op | Lock-free SPSC messaging queue |
| AST parse + mutate (avg file) | ~0.127 ms (127 µs) | Complete `syn` AST parse, traversal & refactoring |
| Incremental build check (`cargo check`) | ~800ms - 2s | External Rust compiler ground truth check |

---

## The Key Insight

The system cannot break itself because:

1. **AST guarantees syntactic validity** — mutations produce valid Rust or fail
2. **cargo check is the ground truth** — invalid mutations are rolled back
3. **Fitness gate** — regressions are rolled back even if they compile
4. **Immutable anchors** — the files that control evolution cannot be evolved

This is structural safety, not prompt-based safety.

---

## Quickstart & Usage

Anyone can use `rongy-spine` as a binary or as a library in their own Rust projects.

### 1. Run Benchmarks & Demonstration CLI

```bash
git clone https://github.com/silentnoisehun/rongy-spine
cd rongy-spine
cargo test
cargo run --release
```

### 2. Use as a Library

Add `rongy-spine` to your `Cargo.toml`:

```toml
[dependencies]
rongy-spine = { path = "." } # or from crates.io / git repo
```

Example Rust usage:

```rust
use rongy_spine::{Spine, AstRefactor, SafetyCore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize 2048-byte IPC bus
    let mut spine = Spine::new();
    spine.set_tension(0.25);
    spine.set_confidence(0.98);
    spine.tick();

    println!("Current tick: {}, Tension: {}", spine.current_tick(), spine.tension());

    // 2. Check mutation safety rules
    let safety = SafetyCore::new();
    if safety.can_mutate("src/my_module.rs").is_allowed() {
        // 3. Perform AST-level code refactoring
        let code = "fn run() -> String { std::fs::read_to_string(\"test.txt\").unwrap() }";
        let (refactored, report) = AstRefactor::refactor_file(code)?;
        println!("Refactored code:\n{}", refactored);
    }

    Ok(())
}
```

---

## License

MIT — Use it, build on it, make it better.

---

## Author

Máté Róbert — Factory worker, Mosonmagyaróvár, Hungary.

> "The bottleneck was never compute. It was architecture."

Part of the Rongyász project: a self-evolving AI system built during
night shifts, without a PhD, without venture capital.
