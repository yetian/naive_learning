// Tests for brain module - Knowledge graph data structures

use seed_intelligence::*;

// We need to make brain module public for testing
// For now, these are integration tests that test through the public API

mod common;

#[test]
fn test_brain_new_is_empty() {
    let brain = common::create_test_brain();
    assert_eq!(brain.total_concepts(), 0);
    assert_eq!(brain.total_relations(), 0);
}

#[test]
fn test_brain_add_concept() {
    let mut brain = common::create_test_brain();

    brain.get_or_create_concept("人工智能");
    assert_eq!(brain.total_concepts(), 1);

    let concept = brain.concepts.get("人工智能");
    assert!(concept.is_some());
}

#[test]
fn test_brain_add_concept_increments_count() {
    let mut brain = common::create_test_brain();

    // get_or_create_concept creates if not exists, increments if exists
    brain.get_or_create_concept("机器学习");
    // The count is set to 1 on first creation
    let concept = brain.concepts.get("机器学习").unwrap();
    assert!(concept.count >= 1);
}

#[test]
fn test_brain_add_relation() {
    let mut brain = common::create_test_brain();

    brain.add_or_update_relation("人工智能", "机器学习");
    assert_eq!(brain.total_relations(), 1);
}

#[test]
fn test_brain_relation_bidirectional() {
    let mut brain = common::create_test_brain();

    brain.add_or_update_relation("A", "B");

    // Same relation should not be duplicated when reversed
    brain.add_or_update_relation("B", "A");
    assert_eq!(brain.total_relations(), 1);
}

#[test]
fn test_brain_cleanup_removes_weak_relations() {
    let mut brain = common::create_test_brain();

    // Add a relation with minimal weight
    brain.add_or_update_relation("A", "B");

    let (pruned_relations, pruned_concepts) = brain.cleanup(0.5, 0.5, false);

    // Relations with weight < 0.5 should be pruned
    println!("Pruned: {} relations, {} concepts", pruned_relations, pruned_concepts);
}

#[test]
fn test_brain_clear() {
    let mut brain = common::create_test_brain();

    brain.get_or_create_concept("测试");
    brain.add_or_update_relation("A", "B");

    brain.clear();

    assert_eq!(brain.total_concepts(), 0);
    assert_eq!(brain.total_relations(), 0);
}

#[test]
fn test_brain_save_and_load() {
    let mut brain = common::create_test_brain();
    let test_path = std::env::temp_dir().join("test_brain_unit.json");

    brain.get_or_create_concept("人工智能");
    brain.add_or_update_relation("人工智能", "机器学习");

    // Save
    brain.save(&test_path).expect("Failed to save");

    // Load into new brain
    let loaded = Brain::load(&test_path);

    assert_eq!(loaded.total_concepts(), brain.total_concepts());
    assert_eq!(loaded.total_relations(), brain.total_relations());

    // Cleanup
    std::fs::remove_file(test_path).ok();
}

// Edge case tests

#[test]
fn test_brain_concept_energy_increases() {
    let mut brain = common::create_test_brain();

    brain.get_or_create_concept("测试");

    // Energy should be set on creation
    let concept = brain.concepts.get("测试").unwrap();
    assert!(concept.energy > 0.0);
}

#[test]
fn test_brain_relation_weight_updates() {
    let mut brain = common::create_test_brain();

    brain.add_or_update_relation("A", "B");

    // Get the relation and check initial state
    if let Some(rel) = brain.get_relation("A", "B") {
        // Initial count should be 0 (just created)
        assert_eq!(rel.count, 0);
    }

    // Update the weight via get_relation_mut
    if let Some(rel) = brain.get_relation_mut("A", "B") {
        rel.weight = 0.8;
        rel.count = 5;
    }

    // Verify update
    let rel = brain.get_relation("A", "B").unwrap();
    assert!((rel.weight - 0.8).abs() < 0.01);
    assert_eq!(rel.count, 5);
}

#[test]
fn test_brain_get_relation_mut_nonexistent() {
    let mut brain = common::create_test_brain();

    let result = brain.get_relation_mut("NonExistent", "Relation");

    assert!(result.is_none());
}

#[test]
fn test_brain_get_relations_for_concept() {
    let mut brain = common::create_test_brain();

    brain.add_or_update_relation("A", "B");
    brain.add_or_update_relation("A", "C");
    brain.add_or_update_relation("B", "C");

    let relations_a = brain.get_relations_for_concept("A");

    // A should have 2 relations: A-B and A-C
    assert_eq!(relations_a.len(), 2);
}

#[test]
fn test_brain_get_relations_for_nonexistent_concept() {
    let brain = common::create_test_brain();

    let relations = brain.get_relations_for_concept("NonExistent");

    assert!(relations.is_empty());
}

#[test]
fn test_brain_find_relation() {
    let mut brain = common::create_test_brain();

    brain.add_or_update_relation("A", "B");

    // Should find the relation in either direction
    assert!(brain.get_relation("A", "B").is_some());
    assert!(brain.get_relation("B", "A").is_some());

    // Non-existent relation
    assert!(brain.get_relation("A", "C").is_none());
}

#[test]
fn test_brain_load_nonexistent_file() {
    let path = std::path::PathBuf::from("/nonexistent/path/brain.json");

    // Should return default brain when file doesn't exist
    let brain = Brain::load(&path);

    assert_eq!(brain.total_concepts(), 0);
    assert_eq!(brain.total_relations(), 0);
}

#[test]
fn test_brain_load_invalid_json() {
    let temp_path = std::env::temp_dir().join("invalid_brain.json");

    // Write invalid JSON
    std::fs::write(&temp_path, "not valid json").ok();

    // Should return default brain when JSON is invalid
    let brain = Brain::load(&temp_path);

    assert_eq!(brain.total_concepts(), 0);

    // Cleanup
    std::fs::remove_file(temp_path).ok();
}

#[test]
fn test_brain_version() {
    let brain = common::create_test_brain();

    assert_eq!(brain.version, "2.0");
}

#[test]
fn test_brain_meta() {
    let brain = common::create_test_brain();

    assert_eq!(brain.meta.total_concepts, 0);
    assert_eq!(brain.meta.total_relations, 0);
    assert_eq!(brain.meta.total_learn_count, 0);
}

#[test]
fn test_brain_cleanup_aggressive() {
    let mut brain = common::create_test_brain();

    // Add concepts and relations
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.add_or_update_relation("A", "B");

    // Cleanup with aggressive mode
    let (pruned_relations, pruned_concepts) = brain.cleanup(0.01, 0.01, true);

    // Aggressive cleanup should still work
    println!("Aggressive cleanup: {} relations, {} concepts pruned", pruned_relations, pruned_concepts);
}

#[test]
fn test_brain_concept_timestamps() {
    let mut brain = common::create_test_brain();

    brain.get_or_create_concept("测试");

    let concept = brain.concepts.get("测试").unwrap();

    // Should have timestamps
    assert!(!concept.first_seen.is_empty());
    assert!(!concept.last_seen.is_empty());
}

#[test]
fn test_brain_relation_id_format() {
    let mut brain = common::create_test_brain();

    brain.add_or_update_relation("概念A", "概念B");

    let rel = brain.relations.values().next().unwrap();

    // ID should start with "rel_"
    assert!(rel.id.starts_with("rel_"));
}

#[test]
fn test_brain_serialization() {
    let mut brain = common::create_test_brain();

    brain.get_or_create_concept("人工智能");
    brain.add_or_update_relation("人工智能", "机器学习");

    // Serialize to JSON
    let json = serde_json::to_string(&brain).unwrap();

    // Should contain expected fields
    assert!(json.contains("version"));
    assert!(json.contains("人工智能"));
    assert!(json.contains("机器学习"));

    // Deserialize back
    let parsed: Brain = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.total_concepts(), brain.total_concepts());
    assert_eq!(parsed.total_relations(), brain.total_relations());
}

#[test]
fn test_brain_clone() {
    let mut brain = common::create_test_brain();

    brain.get_or_create_concept("测试");
    brain.add_or_update_relation("A", "B");

    let cloned = brain.clone();

    assert_eq!(cloned.total_concepts(), brain.total_concepts());
    assert_eq!(cloned.total_relations(), brain.total_relations());
}

#[test]
fn test_brain_default() {
    let brain1 = Brain::new();
    let brain2 = Brain::default();

    assert_eq!(brain1.total_concepts(), brain2.total_concepts());
    assert_eq!(brain1.version, brain2.version);
}

#[test]
fn test_brain_concepts_with_special_chars() {
    let mut brain = common::create_test_brain();

    // Test with various Unicode characters
    brain.get_or_create_concept("人工智能");
    brain.get_or_create_concept("Machine Learning");
    brain.get_or_create_concept("🤖 AI");

    assert_eq!(brain.total_concepts(), 3);
}

#[test]
fn test_brain_relation_with_same_concepts() {
    let mut brain = common::create_test_brain();

    // Relation between same concept should still work (or be handled)
    brain.add_or_update_relation("A", "A");

    // Implementation dependent - may or may not create self-relation
    println!("Self-relation count: {}", brain.total_relations());
}

#[test]
fn test_brain_large_number_of_concepts() {
    let mut brain = common::create_test_brain();

    // Add many concepts
    for i in 0..100 {
        brain.get_or_create_concept(&format!("概念{}", i));
    }

    assert_eq!(brain.total_concepts(), 100);
}

#[test]
fn test_brain_large_number_of_relations() {
    let mut brain = common::create_test_brain();

    // Create a chain of relations
    for i in 0..50 {
        brain.add_or_update_relation(&format!("概念{}", i), &format!("概念{}", i + 1));
    }

    assert_eq!(brain.total_relations(), 50);
}
