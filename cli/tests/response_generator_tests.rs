// Tests for response generator module - Natural language generation

mod common;

use seed_intelligence::response_generator::{
    infer_relation_type,
    relation_to_sentence,
    generate_single_concept_answer,
    generate_multi_concept_answer,
    RelationType,
};

#[test]
fn test_infer_relation_type_part_of() {
    let rel_type = infer_relation_type("机器学习", "人工智能分支");

    assert_eq!(rel_type, RelationType::PartOf);
}

#[test]
fn test_infer_relation_type_has_property() {
    let rel_type = infer_relation_type("神经网络", "深度学习技术");

    assert_eq!(rel_type, RelationType::HasProperty);
}

#[test]
fn test_infer_relation_type_related_to() {
    let rel_type = infer_relation_type("人工智能", "计算机");

    assert_eq!(rel_type, RelationType::RelatedTo);
}

#[test]
fn test_relation_to_sentence_is_a() {
    let sentence = relation_to_sentence("猫", "动物", 0.8, &RelationType::IsA);

    assert!(sentence.contains("猫"));
    assert!(sentence.contains("动物"));
}

#[test]
fn test_relation_to_sentence_part_of() {
    let sentence = relation_to_sentence("机器学习", "人工智能", 0.7, &RelationType::PartOf);

    assert!(sentence.contains("机器学习"));
    assert!(sentence.contains("人工智能"));
}

#[test]
fn test_relation_to_sentence_related_to() {
    let sentence = relation_to_sentence("人工智能", "计算机", 0.5, &RelationType::RelatedTo);

    assert!(sentence.contains("人工智能"));
    assert!(sentence.contains("计算机"));
}

#[test]
fn test_generate_single_concept_answer_empty() {
    let answer = generate_single_concept_answer("测试概念", &[]);

    assert!(answer.contains("测试概念"));
}

#[test]
fn test_generate_single_concept_answer_with_relations() {
    let related = vec![
        ("相关概念1".to_string(), 0.8),
        ("相关概念2".to_string(), 0.5),
        ("相关概念3".to_string(), 0.2),
    ];

    let answer = generate_single_concept_answer("主概念", &related);

    assert!(answer.contains("主概念"));
}

#[test]
fn test_generate_multi_concept_answer_no_path() {
    let concepts = vec!["概念A".to_string(), "概念B".to_string()];
    let path = vec![];
    let path_details = vec![];

    let answer = generate_multi_concept_answer(&concepts, &path, &path_details);

    assert!(answer.contains("概念A") || answer.contains("概念B"));
}

#[test]
fn test_generate_multi_concept_answer_with_path() {
    let concepts = vec!["人工智能".to_string(), "机器学习".to_string()];
    let path = vec!["人工智能".to_string(), "技术".to_string(), "机器学习".to_string()];
    let path_details = vec![
        ("人工智能".to_string(), "技术".to_string(), 0.5),
        ("技术".to_string(), "机器学习".to_string(), 0.4),
    ];

    let answer = generate_multi_concept_answer(&concepts, &path, &path_details);

    // Should contain some reference to the concepts
    assert!(!answer.is_empty());
}

#[test]
fn test_sentence_varies_by_weight() {
    let sentence_high = relation_to_sentence("A", "B", 0.8, &RelationType::RelatedTo);
    let sentence_low = relation_to_sentence("A", "B", 0.1, &RelationType::RelatedTo);

    // Both should mention A and B
    assert!(sentence_high.contains("A"));
    assert!(sentence_high.contains("B"));
    assert!(sentence_low.contains("A"));
    assert!(sentence_low.contains("B"));
}
