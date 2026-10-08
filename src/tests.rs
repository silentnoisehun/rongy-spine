#[cfg(test)]
mod tests {
    use crate::spine::{Spine, slot};
    use crate::ast_refactor::AstRefactor;
    use crate::safety::SafetyCore;

    #[test]
    fn test_spine_basic() {
        let mut spine = Spine::new();
        assert_eq!(spine.current_tick(), 0);
        spine.tick();
        assert_eq!(spine.current_tick(), 1);

        spine.set_tension(0.75);
        assert!((spine.tension() - 0.75).abs() < 1e-5);

        spine.set_confidence(0.9);
        assert!((spine.confidence() - 0.9).abs() < 1e-5);

        spine.set_phi(0.85);
        assert!((spine.phi() - 0.85).abs() < 1e-5);

        spine.set_colony_fitness(0.97);
        assert!((spine.colony_fitness() - 0.97).abs() < 1e-5);

        spine.set_colony_generation(42);
        assert_eq!(spine.colony_generation(), 42);
    }

    #[test]
    fn test_spine_from_bytes() {
        let spine1 = Spine::new();
        let bytes = spine1.as_bytes();
        let spine2 = Spine::from_bytes(bytes).expect("Failed to recreate Spine from bytes");
        assert_eq!(spine1.read_f32(slot::CONFIDENCE), spine2.read_f32(slot::CONFIDENCE));
    }

    #[test]
    fn test_spine_ring_buffer() {
        let mut spine = Spine::new();
        assert_eq!(spine.ring_len(), 0);

        let msg1 = b"hello world";
        let msg2 = b"second message";

        assert!(spine.ring_push(msg1));
        assert!(spine.ring_push(msg2));
        assert_eq!(spine.ring_len(), 2);

        let popped1 = spine.ring_pop();
        assert_eq!(popped1, Some(msg1.to_vec()));
        assert_eq!(spine.ring_len(), 1);

        let popped2 = spine.ring_pop();
        assert_eq!(popped2, Some(msg2.to_vec()));
        assert_eq!(spine.ring_len(), 0);

        assert_eq!(spine.ring_pop(), None);
    }

    #[test]
    fn test_ast_refactor_single_func() {
        let code = r#"
fn read_file_content(path: &str) -> String {
    let data = std::fs::read_to_string(path).unwrap();
    data
}
"#;
        let (refactored, report) = AstRefactor::refactor_file(code).expect("refactor failed");
        assert_eq!(report.functions_found, 1);
        assert_eq!(report.functions_refactored, 1);
        assert_eq!(report.unwraps_converted, 1);
        assert!(refactored.contains("Result<String, Box<dyn std::error::Error>>"));
        assert!(refactored.contains("?"));
        assert!(refactored.contains("Ok(data)"));
    }

    #[test]
    fn test_ast_refactor_cascade() {
        let code = r#"
fn fetch_raw(url: &str) -> String {
    let resp = std::fs::read_to_string(url).unwrap();
    resp
}

fn process_url(url: &str) -> String {
    let data = fetch_raw(url);
    data
}
"#;
        let (refactored, report) = AstRefactor::cascade_refactor(code).expect("cascade refactor failed");
        assert!(report.functions_found >= 2);
        assert!(report.functions_refactored >= 1);
        assert!(refactored.contains("Result"));
    }

    #[test]
    fn test_safety_core_anchors() {
        let safety = SafetyCore::new();
        assert!(!safety.can_mutate("src/safety.rs").is_allowed());
        assert!(!safety.can_mutate("src/spine.rs").is_allowed());
        assert!(!safety.can_mutate("Cargo.toml").is_allowed());
        assert!(safety.can_mutate("src/worker.rs").is_allowed());
        assert!(safety.can_mutate("src/utils.rs").is_allowed());
    }

    #[test]
    fn test_safety_core_limits() {
        let safety = SafetyCore::new();
        for _ in 0..20 {
            safety.record_failure();
        }
        assert!(!safety.can_mutate("src/worker.rs").is_allowed());

        safety.reset();
        assert!(safety.can_mutate("src/worker.rs").is_allowed());
    }
}
