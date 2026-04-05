// Common test utilities

use seed_intelligence::brain::Brain;
use seed_intelligence::learner::IncrementalLearner;
use std::path::PathBuf;

/// Create a fresh brain for testing
#[allow(dead_code)]
pub fn create_test_brain() -> Brain {
    Brain::new()
}

/// Create a fresh learner with a temp brain path
#[allow(dead_code)]
pub fn create_test_learner() -> IncrementalLearner {
    let temp_path = std::env::temp_dir().join(format!("seed_test_{}.json", uuid()));
    IncrementalLearner::new(Some(temp_path))
}

/// Create a temporary file path for test data
#[allow(dead_code)]
pub fn temp_brain_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("seed_test_{}.json", name))
}

/// Clean up test files
#[allow(dead_code)]
pub fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
}

/// Generate a unique ID for test isolation
fn uuid() -> String {
    format!("{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0))
}
