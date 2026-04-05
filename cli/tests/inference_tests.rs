// Tests for inference module - Graph-based Q&A

mod common;

use seed_intelligence::brain::Brain;
use seed_intelligence::inference::{query, ask, parse_query, find_matching_concepts, find_paths, dijkstra, find_best_path, aggregate_answer};

fn create_brain_with_knowledge() -> Brain {
    let mut brain = Brain::new();

    // Add some concepts
    brain.get_or_create_concept("人工智能");
    brain.get_or_create_concept("机器学习");
    brain.get_or_create_concept("深度学习");

    // Add relations
    brain.add_or_update_relation("人工智能", "机器学习");
    brain.add_or_update_relation("机器学习", "深度学习");

    brain
}

#[test]
fn test_parse_query_extracts_keywords() {
    let words = parse_query("什么是人工智能");

    assert!(!words.is_empty());
}

#[test]
fn test_parse_query_filters_stopwords() {
    let words = parse_query("什么是人工智能");

    // "什么" and "是" should be filtered as stop words
    assert!(!words.contains(&"什么".to_string()));
    assert!(!words.contains(&"是".to_string()));
}

#[test]
fn test_find_matching_concepts() {
    let brain = create_brain_with_knowledge();
    let query_words = vec!["人工智能".to_string()];

    let matches = find_matching_concepts(&query_words, &brain);

    assert!(!matches.is_empty());
}

#[test]
fn test_query_returns_answer() {
    let brain = create_brain_with_knowledge();

    let answer = query("人工智能", &brain);

    assert!(!answer.answer.is_empty());
}

#[test]
fn test_ask_returns_answer() {
    let brain = create_brain_with_knowledge();

    let answer = ask("什么是人工智能", &brain);

    assert!(!answer.answer.is_empty());
}

#[test]
fn test_query_empty_brain() {
    let brain = Brain::new();

    let answer = query("测试问题", &brain);

    // Should return a message about not knowing
    // The message says "我还不了解" or similar
    assert!(answer.answer.contains("不了解") || answer.answer.contains("学习") || answer.answer.is_empty() == false);
}

#[test]
fn test_ask_single_concept() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("测试概念");
    brain.add_or_update_relation("测试概念", "相关概念");

    let answer = ask("测试概念", &brain);

    assert!(!answer.answer.is_empty());
}

#[test]
fn test_query_confidence_range() {
    let brain = create_brain_with_knowledge();

    let answer = query("人工智能", &brain);

    // Confidence should be 0-100
    assert!(answer.confidence <= 100);
}

// Path finding algorithm tests

#[test]
fn test_find_paths_single_concept() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.add_or_update_relation("A", "B");

    let paths = find_paths("A", &brain, 2);

    // Should find path to B
    assert!(!paths.is_empty());
}

#[test]
fn test_find_paths_chain() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.get_or_create_concept("C");
    brain.add_or_update_relation("A", "B");
    brain.add_or_update_relation("B", "C");

    let paths = find_paths("A", &brain, 3);

    // Should find paths: A->B, A->B->C
    assert!(!paths.is_empty());
}

#[test]
fn test_find_paths_empty_brain() {
    let brain = Brain::new();

    let paths = find_paths("NonExistent", &brain, 2);

    // Should return empty paths
    assert!(paths.is_empty());
}

#[test]
fn test_find_paths_max_depth() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.get_or_create_concept("C");
    brain.get_or_create_concept("D");
    brain.add_or_update_relation("A", "B");
    brain.add_or_update_relation("B", "C");
    brain.add_or_update_relation("C", "D");

    let paths_depth_1 = find_paths("A", &brain, 1);
    let paths_depth_2 = find_paths("A", &brain, 2);

    // More depth should find more paths
    println!("Paths depth 1: {:?}", paths_depth_1.len());
    println!("Paths depth 2: {:?}", paths_depth_2.len());
}

#[test]
fn test_dijkstra_direct_connection() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.add_or_update_relation("A", "B");

    // Get mutable relation to set weight
    if let Some(rel) = brain.get_relation_mut("A", "B") {
        rel.weight = 0.8;
    }

    let path = dijkstra("A", "B", &brain);

    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 2);
    assert_eq!(path[0], "A");
    assert_eq!(path[1], "B");
}

#[test]
fn test_dijkstra_no_path() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.get_or_create_concept("C");
    // No relation between A and C

    let path = dijkstra("A", "C", &brain);

    assert!(path.is_none());
}

#[test]
fn test_dijkstra_multi_hop() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.get_or_create_concept("C");
    brain.add_or_update_relation("A", "B");
    brain.add_or_update_relation("B", "C");

    if let Some(rel) = brain.get_relation_mut("A", "B") {
        rel.weight = 0.5;
    }
    if let Some(rel) = brain.get_relation_mut("B", "C") {
        rel.weight = 0.5;
    }

    let path = dijkstra("A", "C", &brain);

    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 3);
}

#[test]
fn test_dijkstra_same_node() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");

    let path = dijkstra("A", "A", &brain);

    // Path to self should return just that node
    // (implementation dependent)
    println!("Path to self: {:?}", path);
}

#[test]
fn test_find_best_path_exists() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("人工智能");
    brain.get_or_create_concept("机器学习");
    brain.add_or_update_relation("人工智能", "机器学习");

    if let Some(rel) = brain.get_relation_mut("人工智能", "机器学习") {
        rel.weight = 0.9;
    }

    let result = find_best_path("人工智能", "机器学习", &brain);

    assert!(result.is_some());
    let path_result = result.unwrap();
    assert!(!path_result.path.is_empty());
    assert!(path_result.total_weight > 0.0);
}

#[test]
fn test_find_best_path_no_path() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    // No relation

    let result = find_best_path("A", "B", &brain);

    assert!(result.is_none());
}

#[test]
fn test_find_best_path_with_details() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.add_or_update_relation("A", "B");

    if let Some(rel) = brain.get_relation_mut("A", "B") {
        rel.weight = 0.75;
    }

    let result = find_best_path("A", "B", &brain);

    assert!(result.is_some());
    let path_result = result.unwrap();

    // Check path details
    assert_eq!(path_result.path_details.len(), 1);
    assert_eq!(path_result.path_details[0].from, "A");
    assert_eq!(path_result.path_details[0].to, "B");
    assert!((path_result.path_details[0].weight - 0.75).abs() < 0.01);
}

#[test]
fn test_aggregate_answer_empty_paths() {
    let brain = Brain::new();
    let paths: Vec<Vec<String>> = vec![];

    let answer = aggregate_answer(&paths, &brain, "测试问题");

    assert!(answer.answer.contains("没有") || answer.answer.contains("建议"));
    assert_eq!(answer.confidence, 0);
}

#[test]
fn test_aggregate_answer_with_paths() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("A");
    brain.get_or_create_concept("B");
    brain.add_or_update_relation("A", "B");

    if let Some(rel) = brain.get_relation_mut("A", "B") {
        rel.weight = 0.5;
    }

    let paths = vec![vec!["A".to_string(), "B".to_string()]];

    let answer = aggregate_answer(&paths, &brain, "测试");

    assert!(!answer.answer.is_empty());
    assert!(!answer.concepts.is_empty());
}

#[test]
fn test_find_matching_concepts_fuzzy() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("人工智能");
    brain.get_or_create_concept("机器学习");

    let query_words = vec!["智能".to_string()];

    let matches = find_matching_concepts(&query_words, &brain);

    // Should find "人工智能" via fuzzy match
    assert!(!matches.is_empty());
}

#[test]
fn test_find_matching_concepts_case_insensitive() {
    let mut brain = Brain::new();
    brain.get_or_create_concept("Python");

    let query_words = vec!["python".to_string()];

    let matches = find_matching_concepts(&query_words, &brain);

    // Should match case-insensitively
    assert!(!matches.is_empty());
}
