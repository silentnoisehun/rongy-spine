// safety.rs — Immutable anchor system for safe self-modification
// Defines which files the system can NEVER touch
// Atomic emergency stop on consecutive failures
//
// THIS FILE ITSELF IS AN ANCHOR — the system cannot mutate it
//
// Author: Máté Róbert (silentnoisehun)
// License: MIT

use std::path::Path;
use std::process::Command;

/// Files the system can NEVER modify — these are the anchors
const IMMUTABLE_FILES: &[&str] = &[
    "safety.rs",
    "sentinel.rs",
    "immune_system.rs",
    "self_evolve.rs",
    "hope_watchdog.rs",
    "Cargo.toml",
    "Cargo.lock",
    ".env",
    "build.rs",
    "lib.rs",
    "main.rs",
];

const FORBIDDEN_DIRS: &[&str] = &[
    "target", ".git", "node_modules", "__pycache__",
    "venv", ".venv", "data/vault", "templates",
];

const MAX_MUTATIONS_PER_SESSION: u64 = 500;
const MAX_CONSECUTIVE_FAILURES: u64 = 20;

pub struct SafetyCore {
    mutation_count: std::sync::atomic::AtomicU64,
    consecutive_failures: std::sync::atomic::AtomicU64,
    emergency_stop: std::sync::atomic::AtomicBool,
}

impl SafetyCore {
    pub fn new() -> Self {
        Self {
            mutation_count: std::sync::atomic::AtomicU64::new(0),
            consecutive_failures: std::sync::atomic::AtomicU64::new(0),
            emergency_stop: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn can_mutate(&self, file_path: &str) -> SafetyCheck {
        if self.emergency_stop.load(std::sync::atomic::Ordering::Relaxed) {
            return SafetyCheck::Blocked("EMERGENCY STOP active".to_string());
        }

        let count = self.mutation_count.load(std::sync::atomic::Ordering::Relaxed);
        if count >= MAX_MUTATIONS_PER_SESSION {
            return SafetyCheck::Blocked(format!("Session limit reached ({}/{})", count, MAX_MUTATIONS_PER_SESSION));
        }

        let failures = self.consecutive_failures.load(std::sync::atomic::Ordering::Relaxed);
        if failures >= MAX_CONSECUTIVE_FAILURES {
            self.emergency_stop.store(true, std::sync::atomic::Ordering::Relaxed);
            return SafetyCheck::Blocked(format!("Too many consecutive failures ({}) — emergency stop", failures));
        }

        let path = Path::new(file_path);
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

        for immutable in IMMUTABLE_FILES {
            if file_name == *immutable || file_path.ends_with(immutable) {
                return SafetyCheck::Blocked(format!("'{}' is immutable — anchor file", immutable));
            }
        }

        for forbidden in FORBIDDEN_DIRS {
            if file_path.contains(forbidden) {
                return SafetyCheck::Blocked(format!("'{}' is a forbidden directory", forbidden));
            }
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let binary_exts = ["exe", "dll", "so", "bin", "zip", "tar", "gz", "png", "jpg", "pdf"];
        if binary_exts.contains(&ext) {
            return SafetyCheck::Blocked("Binary files cannot be mutated".to_string());
        }

        if let Ok(meta) = std::fs::metadata(file_path) {
            if meta.len() > 500_000 {
                return SafetyCheck::Blocked(format!("File too large ({} KB) — max 500KB", meta.len() / 1024));
            }
        }

        SafetyCheck::Allowed
    }

    pub fn record_mutation(&self) {
        self.mutation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.consecutive_failures.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.mutation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.consecutive_failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn emergency_stop(&self) {
        self.emergency_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.emergency_stop.store(false, std::sync::atomic::Ordering::Relaxed);
        self.consecutive_failures.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub enum SafetyCheck {
    Allowed,
    Blocked(String),
}

impl SafetyCheck {
    pub fn is_allowed(&self) -> bool {
        matches!(self, SafetyCheck::Allowed)
    }
}

/// Git snapshot before mutation sessions — rollback anchor
pub struct GitSafety;

impl GitSafety {
    pub fn snapshot_before_mutation(message: &str) -> bool {
        let is_git = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !is_git { return false; }

        let _ = Command::new("git").args(["add", "-A"]).output();

        Command::new("git")
            .args(["commit", "-m", &format!("[SAFETY-ANCHOR] {}", message), "--allow-empty"])
            .output()
            .map(|r| r.status.success())
            .unwrap_or(false)
    }

    pub fn rollback_to_last_snapshot() -> bool {
        let result = Command::new("git")
            .args(["log", "--oneline", "-20"])
            .output();

        if let Ok(output) = result {
            let log = String::from_utf8_lossy(&output.stdout);
            for line in log.lines() {
                if line.contains("[SAFETY-ANCHOR]") {
                    if let Some(hash) = line.split_whitespace().next() {
                        return Command::new("git")
                            .args(["reset", "--hard", hash])
                            .output()
                            .map(|r| r.status.success())
                            .unwrap_or(false);
                    }
                }
            }
        }
        false
    }
}
