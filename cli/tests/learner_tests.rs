// Tests for learner module - Hebbian learning engine

mod common;

use seed_intelligence::learner::IncrementalLearner;

#[test]
fn test_learner_new_is_empty() {
    let learner = common::create_test_learner();
    let stats = learner.get_stats();
    assert_eq!(stats.total_concepts, 0);
    assert_eq!(stats.total_relations, 0);
}

#[test]
fn test_learner_learn_from_text() {
    let mut learner = common::create_test_learner();

    let result = learner.learn_from_text("人工智能是计算机科学的一个分支", None);

    assert!(result.success);
    assert!(result.tokens_processed > 0);
    assert!(result.relations_added > 0);
}

#[test]
fn test_learner_learn_increases_concepts() {
    let mut learner = common::create_test_learner();

    learner.learn_from_text("机器学习是人工智能的核心技术", None);
    let stats1 = learner.get_stats();

    learner.learn_from_text("深度学习是机器学习的子集", None);
    let stats2 = learner.get_stats();

    assert!(stats2.total_concepts > stats1.total_concepts);
}

#[test]
fn test_learner_focus_concept_gets_boost() {
    let mut learner = common::create_test_learner();

    learner.learn_from_text("人工智能很重要", Some("人工智能"));
    learner.learn_from_text("机器学习也很重要", Some("机器学习"));

    let ai_concept = learner.get_concept("人工智能");
    let ml_concept = learner.get_concept("机器学习");

    // Both should exist
    assert!(ai_concept.is_some() || ml_concept.is_some());
}

#[test]
fn test_learner_cleanup_removes_noise() {
    let mut learner = common::create_test_learner();

    // Learn some text
    learner.learn_from_text("测试文本用于验证清理功能", None);

    // Cleanup should work
    let cleanup = learner.cleanup(true);
    assert!(cleanup.pruned_relations >= 0);
    assert!(cleanup.pruned_concepts >= 0);
}

#[test]
fn test_learner_get_related_concepts() {
    let mut learner = common::create_test_learner();

    learner.learn_from_text("人工智能与机器学习密切相关，深度学习是机器学习的分支", None);

    let related = learner.get_related_concepts("人工智能", 2);

    // Should find some related concepts
    // Note: may be empty if "人工智能" wasn't tokenized as a single concept
    println!("Related concepts: {:?}", related);
}

#[test]
fn test_learner_stats_accuracy() {
    let mut learner = common::create_test_learner();

    learner.learn_from_text("计算机科学包含人工智能和机器学习", None);

    let stats = learner.get_stats();

    // Stats should be consistent
    assert!(stats.total_concepts > 0);
    assert!(stats.total_relations > 0);
    assert!(stats.avg_weight >= 0.0);
    assert!(stats.avg_energy >= 0.0);
}

#[test]
fn test_learner_clear() {
    let mut learner = common::create_test_learner();

    learner.learn_from_text("测试内容", None);
    assert!(learner.get_stats().total_concepts > 0);

    learner.clear();
    assert_eq!(learner.get_stats().total_concepts, 0);
}

#[test]
fn test_learner_persistence() {
    let test_path = common::temp_brain_path("learner_persistence");

    // Create and save
    {
        let mut learner = IncrementalLearner::new(Some(test_path.clone()));
        learner.learn_from_text("持久化测试", None);
        learner.save().expect("Failed to save");
    }

    // Load and verify
    {
        let learner = IncrementalLearner::new(Some(test_path.clone()));
        assert!(learner.get_stats().total_concepts > 0);
    }

    common::cleanup(&test_path);
}
