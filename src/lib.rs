// Rongyász Spine — Safe Self-Evolving Rust System
// AST-level mutation, Spine IPC, and Safety Anchors
//
// Author: Máté Róbert (silentnoisehun)
// License: MIT

//! # Rongyász Spine (`rongy-spine`)
//!
//! `rongy-spine` provides high-performance inter-process shared memory layout (`Spine`),
//! AST-level Rust code analysis and transformation (`AstRefactor`), and an immutable safety anchor
//! system (`SafetyCore`) for self-modifying or evolutionary Rust architectures.
//!
//! ## Modules Overview
//! - [`spine`]: 2048-byte memory-mapped / shared buffer layout with sub-nanosecond reads/writes and lock-free ring buffer queue.
//! - [`ast_refactor`]: AST-driven code mutator based on `syn` that replaces `.unwrap()` with `?` and converts functions to return `Result`.
//! - [`safety`]: Immutable file and directory anchors preventing self-evolution routines from mutating system-critical code.
//!
//! ## Quick Example
//! ```rust
//! use rongy_spine::{Spine, AstRefactor, SafetyCore};
//!
//! // 1. Create Spine shared memory bus
//! let mut spine = Spine::new();
//! spine.set_tension(0.2);
//! spine.set_confidence(0.95);
//! spine.tick();
//! assert_eq!(spine.current_tick(), 1);
//!
//! // 2. Check mutation safety
//! let safety = SafetyCore::new();
//! assert!(safety.can_mutate("src/my_module.rs").is_allowed());
//! assert!(!safety.can_mutate("src/safety.rs").is_allowed());
//!
//! // 3. Refactor code safely using AST
//! let raw_code = "fn data() -> String { std::fs::read_to_string(\"file.txt\").unwrap() }";
//! if let Ok((improved, report)) = AstRefactor::refactor_file(raw_code) {
//!     println!("Converted unwraps: {}", report.unwraps_converted);
//! }
//! ```

pub mod spine;
pub mod ast_refactor;
pub mod safety;

pub use spine::Spine;
pub use ast_refactor::AstRefactor;
pub use safety::SafetyCore;

#[cfg(test)]
mod tests;
