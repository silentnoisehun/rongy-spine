// Rongyász Spine — Main CLI & Benchmark Runner
// Demonstrates Spine IPC, AST-level refactoring, and Safety Anchors
//
// Author: Máté Róbert (silentnoisehun)
// License: MIT

use std::time::Instant;
use rongy_spine::{Spine, AstRefactor, SafetyCore};

fn main() {
    println!("============================================================");
    println!("       Rongyász — Safe Self-Evolving Rust System            ");
    println!("============================================================");

    // 1. Benchmark Spine IPC Operations
    println!("\n[1] Benchmarking Spine IPC Bus (2048-byte shared memory layout)...");
    let mut spine = Spine::new();
    let write_read_iters = 20_000_000;

    let start = Instant::now();
    for i in 0..write_read_iters {
        spine.set_tension(i as f32 * 0.00001);
        let _ = spine.tension();
    }
    let write_read_elapsed = start.elapsed();
    let ns_per_write_read = write_read_elapsed.as_nanos() as f64 / (write_read_iters * 2) as f64;
    println!("    Spine f32 read/write: {:.2} ns/op (total: {:?}, {} iterations)",
             ns_per_write_read, write_read_elapsed, write_read_iters);

    let tick_iters = 10_000_000;
    let start = Instant::now();
    for _ in 0..tick_iters {
        spine.tick();
    }
    let tick_elapsed = start.elapsed();
    let ns_per_tick = tick_elapsed.as_nanos() as f64 / tick_iters as f64;
    println!("    Spine full tick cycle: {:.2} ns/op (total: {:?}, {} ticks)",
             ns_per_tick, tick_elapsed, tick_iters);

    // 2. Benchmark Ring Buffer Inter-Module IPC
    let ring_iters = 5_000_000;
    let msg = b"AFFECTIVE_STATE_UPDATE:TENSION=0.45,PHI=0.82";
    let start = Instant::now();
    for _ in 0..ring_iters {
        spine.ring_push(msg);
        let _ = spine.ring_pop();
    }
    let ring_elapsed = start.elapsed();
    let ns_per_ring = ring_elapsed.as_nanos() as f64 / (ring_iters * 2) as f64;
    println!("    Ring buffer push+pop: {:.2} ns/op (total: {:?}, {} ops)",
             ns_per_ring, ring_elapsed, ring_iters);

    // 3. Benchmark AST Refactoring
    println!("\n[2] Benchmarking AST Parse & Mutation (ast_refactor.rs)...");
    let sample_code = r#"
fn load_config(path: &str) -> String {
    let content = std::fs::read_to_string(path).unwrap();
    content
}

fn parse_value(raw: &str) -> i32 {
    let val: i32 = raw.trim().parse().unwrap();
    val
}
"#;

    let ast_runs = 100;
    let start = Instant::now();
    let mut last_output = String::new();
    let mut last_report = None;
    for _ in 0..ast_runs {
        if let Ok((out, rep)) = AstRefactor::refactor_file(sample_code) {
            last_output = out;
            last_report = Some(rep);
        }
    }
    let ast_elapsed = start.elapsed();
    let ms_per_ast = (ast_elapsed.as_secs_f64() * 1000.0) / ast_runs as f64;
    println!("    AST parse + refactor (avg file): {:.3} ms ({:.1} µs)", ms_per_ast, ms_per_ast * 1000.0);
    if let Some(rep) = last_report {
        println!("    Report: {:?}", rep);
    }
    println!("\n    --- Refactored Code Output Sample ---");
    for line in last_output.lines() {
        println!("    {}", line);
    }

    // 4. Test Safety Anchor Core
    println!("\n[3] Testing Safety Core Anchor File Protection (safety.rs)...");
    let safety = SafetyCore::new();
    let immutable_files = ["src/safety.rs", "src/spine.rs", "Cargo.toml", "main.rs"];
    let mutable_files = ["src/worker.rs", "src/parser.rs", "src/pipeline.rs"];

    for file in &immutable_files {
        println!("    Check '{:<15}': {:?}", file, safety.can_mutate(file));
    }
    for file in &mutable_files {
        println!("    Check '{:<15}': {:?}", file, safety.can_mutate(file));
    }

    println!("\n============================================================");
    println!("                     Benchmark Complete                     ");
    println!("============================================================");
}
