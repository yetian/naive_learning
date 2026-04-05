// Response Generator - Converts graph knowledge to natural language
// Uses templates and optional LM refinement

use crate::brain::Brain;
use crate::lm;

/// Relationship types that can be inferred
#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    IsA,         // A 是 B (A is a type of B)
    PartOf,      // A 是 B 的部分 (A is part of B)
    RelatedTo,   // A 与 B 相关 (A is related to B)
    Causes,      // A 导致 B (A causes B)
    HasProperty, // A 具有 B 属性 (A has property B)
    Unknown,     // 未知关系
}

/// Infer relationship type from concept names
pub fn infer_relation_type(source: &str, target: &str) -> RelationType {
    let s = source.to_lowercase();
    let t = target.to_lowercase();

    // Check for "是" (is) pattern
    if s.contains("是") || t.contains("是") {
        return RelationType::IsA;
    }

    // Check for part-of patterns
    let part_indicators = ["部分", "分支", "子领域", "子类", "类型", "种类"];
    for indicator in &part_indicators {
        if s.contains(indicator) || t.contains(indicator) {
            return RelationType::PartOf;
        }
    }

    // Check for causal patterns
    let cause_indicators = ["导致", "引起", "产生", "造成", "影响"];
    for indicator in &cause_indicators {
        if s.contains(indicator) || t.contains(indicator) {
            return RelationType::Causes;
        }
    }

    // Check for property patterns
    let property_indicators = ["技术", "方法", "特点", "特征", "能力", "功能"];
    for indicator in &property_indicators {
        if t.contains(indicator) {
            return RelationType::HasProperty;
        }
    }

    RelationType::RelatedTo
}

/// Generate natural language sentence for a relation
pub fn relation_to_sentence(source: &str, target: &str, weight: f64, rel_type: &RelationType) -> String {
    let strength = if weight > 0.6 { "密切" } else if weight > 0.3 { "一定" } else { "某种" };

    match rel_type {
        RelationType::IsA => {
            format!("{}是{}的一种。", source, target)
        }
        RelationType::PartOf => {
            format!("{}属于{}的范畴。", source, target)
        }
        RelationType::Causes => {
            format!("{}对{}有着{}影响。", source, target, strength)
        }
        RelationType::HasProperty => {
            format!("{}具有{}的特点。", source, target)
        }
        RelationType::RelatedTo => {
            if weight > 0.5 {
                format!("{}与{}有着{}的关联。", source, target, strength)
            } else {
                format!("{}与{}存在一定的联系。", source, target)
            }
        }
        RelationType::Unknown => {
            format!("{}与{}相关。", source, target)
        }
    }
}

/// Generate a coherent paragraph from multiple relations
pub fn generate_paragraph(relations: &[(String, String, f64)], main_concept: &str) -> String {
    if relations.is_empty() {
        return format!("关于\"{}\"，我目前了解的信息还不够多。", main_concept);
    }

    let mut sentences = Vec::new();
    let mut mentioned = std::collections::HashSet::new();
    mentioned.insert(main_concept.to_string());

    for (source, target, weight) in relations {
        // Determine which concept is the main one in this relation
        let (from, to) = if source == main_concept || mentioned.contains(source) {
            (source.as_str(), target.as_str())
        } else if target == main_concept || mentioned.contains(target) {
            (target.as_str(), source.as_str())
        } else {
            (source.as_str(), target.as_str())
        };

        let rel_type = infer_relation_type(from, to);
        let sentence = relation_to_sentence(from, to, *weight, &rel_type);

        if !sentences.contains(&sentence) {
            sentences.push(sentence);
            mentioned.insert(from.to_string());
            mentioned.insert(to.to_string());
        }

        if sentences.len() >= 4 {
            break;
        }
    }

    sentences.join("")
}

/// Generate answer for single concept query
pub fn generate_single_concept_answer(concept: &str, related: &[(String, f64)]) -> String {
    if related.is_empty() {
        return format!(
            "关于\"{}\"，这是我刚刚学到的新概念。如果你能告诉我更多相关信息，我可以更好地理解它。",
            concept
        );
    }

    // Filter out noise (short words, punctuation)
    let filtered: Vec<_> = related.iter()
        .filter(|(c, w)| c.len() >= 2 && *w > 0.01 && !c.starts_with('，') && !c.starts_with('。'))
        .cloned()
        .collect();

    if filtered.is_empty() {
        return format!(
            "关于\"{}\"，我目前了解的信息还不够多。建议我通过 `./seed init \"{}\"` 来获取更多信息？",
            concept, concept
        );
    }

    let mut response = format!("关于\"{}\"，我了解到：\n\n", concept);

    // Group related concepts by strength
    let strong: Vec<_> = filtered.iter().filter(|(_, w)| *w > 0.3).collect();
    let medium: Vec<_> = filtered.iter().filter(|(_, w)| *w > 0.1 && *w <= 0.3).collect();
    let weak: Vec<_> = filtered.iter().filter(|(_, w)| *w <= 0.1).collect();

    if !strong.is_empty() {
        let concepts: Vec<_> = strong.iter().take(3).map(|(c, _)| c.as_str()).collect();
        response += &format!("它与{}等概念有着密切的关联。\n", concepts.join("、"));
    }

    if !medium.is_empty() {
        let concepts: Vec<_> = medium.iter().take(3).map(|(c, _)| c.as_str()).collect();
        response += &format!("此外，它还与{}等概念有所关联。\n", concepts.join("、"));
    }

    if !weak.is_empty() && weak.len() > 1 {
        let concepts: Vec<_> = weak.iter().take(2).map(|(c, _)| c.as_str()).collect();
        response += &format!("另外，{}也值得了解。\n", concepts.join("和"));
    }

    response
}

/// Generate answer for multi-concept query (finding relationship)
pub fn generate_multi_concept_answer(
    concepts: &[String],
    path: &[String],
    path_details: &[(String, String, f64)],
) -> String {
    if concepts.len() < 2 {
        return generate_single_concept_answer(&concepts[0], &[]);
    }

    let concept_a = &concepts[0];
    let concept_b = &concepts[1];

    if path.len() < 2 {
        return format!(
            "\"{}\"和\"{}\"是两个不同的概念。目前我还没有发现它们之间的直接关联。\n\n也许你可以告诉我更多关于它们的关系？",
            concept_a, concept_b
        );
    }

    if path.len() == 2 {
        // Direct connection
        let rel_type = infer_relation_type(concept_a, concept_b);
        return relation_to_sentence(concept_a, concept_b, path_details[0].2, &rel_type);
    }

    // Indirect connection - describe the path
    // Filter out noise in path
    let clean_path: Vec<_> = path.iter()
        .filter(|p| p.len() >= 2 && !p.starts_with('，') && !p.starts_with('。'))
        .cloned()
        .collect();

    if clean_path.len() < 2 {
        return format!(
            "\"{}\"和\"{}\"虽然都与某些概念有关联，但我还不能清晰地描述它们之间的关系。",
            concept_a, concept_b
        );
    }

    let mut response = format!("\"{}\"和\"{}\"之间可以通过以下路径联系起来：\n\n", concept_a, concept_b);

    // Describe the connection chain
    response += &format!("{} ", clean_path[0]);
    for i in 1..clean_path.len().min(5) {
        response += &format!("→ {} ", clean_path[i]);
    }
    response += "\n\n";

    // Generate natural language description
    response += "简单来说：\n";
    for i in 0..clean_path.len().saturating_sub(1).min(3) {
        let from = &clean_path[i];
        let to = &clean_path[i + 1];
        response += &format!("• {}与{}存在关联\n", from, to);
    }

    response
}

/// Use local LM to refine the answer (make it more natural)
pub fn refine_with_lm(answer: &str, _concepts: &[String], model: Option<&lm::CausalLM>) -> String {
    // If no model provided, return original
    let model = match model {
        Some(m) => m,
        None => return answer.to_string(),
    };

    // Create a prompt for refinement
    let prompt = format!(
        "请用更自然的语言重述：{}",
        answer.chars().take(200).collect::<String>()
    );

    // Generate refined text (short)
    let refined = model.generate(&prompt, 50, 0.8);

    // If refinement is too short or empty, return original
    if refined.len() < 10 {
        return answer.to_string();
    }

    refined
}

/// Main function to generate human-readable answer
pub fn generate_human_answer(
    question: &str,
    _brain: &Brain,
    matches: &[String],
    paths: &[Vec<String>],
    path_details: &[(String, String, f64)],
    related: &[(String, f64)],
) -> String {
    // Determine query type
    if matches.len() == 1 {
        // Single concept query
        generate_single_concept_answer(&matches[0], related)
    } else if matches.len() >= 2 && !paths.is_empty() {
        // Multi-concept query with path
        generate_multi_concept_answer(matches, &paths[0], path_details)
    } else if matches.len() >= 2 {
        // Multi-concept but no path
        let concepts_str = matches.join("和");
        format!(
            "你提到了{}这两个概念。目前我还没有发现它们之间的直接关联。\n\n如果你能提供更多上下文，我可以更好地回答你的问题。",
            concepts_str
        )
    } else {
        // No matches
        format!(
            "我暂时没有找到与\"{}\"相关的信息。\n\n你可以使用 `./seed init \"概念名\"` 或 `./seed learn-file <文件>` 来帮助我学习新知识。",
            question
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_relation_type() {
        assert_eq!(infer_relation_type("机器学习", "人工智能分支"), RelationType::PartOf);
        assert_eq!(infer_relation_type("深度学习", "机器学习技术"), RelationType::HasProperty);
    }

    #[test]
    fn test_relation_to_sentence() {
        let s = relation_to_sentence("深度学习", "机器学习", 0.8, &RelationType::PartOf);
        assert!(s.contains("属于"));
    }

    #[test]
    fn test_generate_paragraph() {
        let relations = vec![
            ("机器学习".to_string(), "人工智能".to_string(), 0.7),
            ("深度学习".to_string(), "机器学习".to_string(), 0.8),
        ];
        let p = generate_paragraph(&relations, "机器学习");
        assert!(!p.is_empty());
    }
}
