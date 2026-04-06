// Inference Engine - Graph-based Q&A using path aggregation
//
// This module provides the core Q&A functionality:
// - Parse queries into keywords
// - Find matching concepts in the knowledge graph
// - Discover paths between concepts
// - Aggregate answers from graph traversal

use crate::brain::Brain;
use crate::nlp::{filter_stop_words, tokenize};
use crate::response_generator;
use std::collections::{HashMap, HashSet, VecDeque};

// =============================================================================
// Query Parsing
// =============================================================================

/// Parse user question into keywords
pub fn parse_query(question: &str) -> Vec<String> {
    let tokens = tokenize(question);
    filter_stop_words(&tokens)
}

/// Find matching concepts in brain
pub fn find_matching_concepts(query_words: &[String], brain: &Brain) -> Vec<Match> {
    let mut matches = Vec::new();
    let mut matched_names = HashSet::new();
    let mut matched_word_indices = HashSet::new();

    // Get all concepts from SQLite
    let concepts = brain.get_all_concepts();

    // Try multi-word phrases first, longest to shortest (e.g., "Machine Learning" before "machine")
    for phrase_len in (2..=query_words.len().min(4)).rev() {
        for i in 0..=(query_words.len().saturating_sub(phrase_len)) {
            let j = i + phrase_len;
            // Skip if any word in this range is already matched
            if (i..j).any(|idx| matched_word_indices.contains(&idx)) {
                continue;
            }

            let phrase = query_words[i..j].join(" ");
            let phrase_lower = phrase.to_lowercase();

            // Case-insensitive match
            for (concept_name, concept) in &concepts {
                if concept_name.to_lowercase() == phrase_lower && !matched_names.contains(concept_name) {
                    matches.push(Match {
                        word: phrase.clone(),
                        concept_name: concept_name.clone(),
                        energy: concept.energy,
                        exact: true,
                    });
                    matched_names.insert(concept_name.clone());
                    // Mark word indices as matched
                    for idx in i..j {
                        matched_word_indices.insert(idx);
                    }
                    break;
                }
            }
        }
    }

    for (idx, word) in query_words.iter().enumerate() {
        if matched_word_indices.contains(&idx) {
            continue;
        }

        let lower_word = word.to_lowercase();

        // Exact match (priority)
        if let Some(concept) = concepts.get(word) {
            if !matched_names.contains(word) {
                matches.push(Match {
                    word: word.clone(),
                    concept_name: word.clone(),
                    energy: concept.energy,
                    exact: true,
                });
                matched_names.insert(word.clone());
                matched_word_indices.insert(idx);
            }
        }

        // Fuzzy match (only if no exact match)
        if !matched_word_indices.contains(&idx) {
            for (concept_name, concept) in &concepts {
                if concept_name.to_lowercase().contains(&lower_word)
                    && !matched_names.contains(concept_name)
                {
                    matches.push(Match {
                        word: word.clone(),
                        concept_name: concept_name.clone(),
                        energy: concept.energy,
                        exact: false,
                    });
                    matched_names.insert(concept_name.clone());
                    matched_word_indices.insert(idx);
                    break; // Only match one concept per word
                }
            }
        }
    }

    matches
}

// =============================================================================
// Graph Traversal
// =============================================================================

/// Build adjacency list from brain relations
fn build_adjacency(brain: &Brain) -> HashMap<String, Vec<(String, f64)>> {
    let mut adj: HashMap<String, Vec<(String, f64)>> = HashMap::new();

    let relations = brain.get_all_relations();
    for rel in relations.values() {
        adj.entry(rel.source.clone())
            .or_default()
            .push((rel.target.clone(), rel.weight));
        adj.entry(rel.target.clone())
            .or_default()
            .push((rel.source.clone(), rel.weight));
    }

    adj
}

/// BFS to find all paths from start concept
pub fn find_paths(start_concept: &str, brain: &Brain, max_depth: usize) -> Vec<Vec<String>> {
    let adj = build_adjacency(brain);
    let mut paths = Vec::new();
    let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();

    queue.push_back((start_concept.to_string(), vec![start_concept.to_string()]));

    while let Some((current, path)) = queue.pop_front() {
        if path.len() > max_depth {
            continue;
        }

        if let Some(connections) = adj.get(&current) {
            for (neighbor, _weight) in connections {
                if !path.contains(neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.clone());
                    paths.push(new_path.clone());
                    queue.push_back((neighbor.clone(), new_path));
                }
            }
        }
    }

    paths
}

/// Dijkstra's algorithm to find the highest-weight path between two nodes
pub fn dijkstra(start: &str, end: &str, brain: &Brain) -> Option<Vec<String>> {
    let adj = build_adjacency(brain);
    let concepts = brain.get_all_concepts();

    let mut dist: HashMap<String, f64> = HashMap::new();
    let mut prev: HashMap<String, Option<String>> = HashMap::new();
    let mut visited = HashSet::new();

    // Initialize distances
    for node in concepts.keys() {
        dist.insert(node.clone(), f64::NEG_INFINITY);
    }
    dist.insert(start.to_string(), 0.0);

    // Priority queue (using simple Vec and sort by highest weight)
    let mut pq: Vec<(String, f64)> = vec![(start.to_string(), 0.0)];

    while let Some((current, _weight)) = pq.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        if current == end {
            break;
        }

        if let Some(connections) = adj.get(&current) {
            for (neighbor, weight) in connections {
                if visited.contains(neighbor) {
                    continue;
                }

                let new_dist = dist.get(&current).unwrap_or(&f64::NEG_INFINITY) + weight;

                if new_dist > *dist.get(neighbor).unwrap_or(&f64::NEG_INFINITY) {
                    dist.insert(neighbor.clone(), new_dist);
                    prev.insert(neighbor.clone(), Some(current.clone()));
                    pq.push((neighbor.clone(), new_dist));
                }
            }
        }

        // Sort to simulate priority queue (highest weight first)
        pq.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    // Reconstruct path
    if !prev.contains_key(end) && end != start {
        return None;
    }

    reconstruct_path(end, &prev)
}

/// Reconstruct path from previous nodes map
fn reconstruct_path(end: &str, prev: &HashMap<String, Option<String>>) -> Option<Vec<String>> {
    let mut path = Vec::new();
    let mut current = Some(end.to_string());

    while let Some(node) = current {
        path.push(node.clone());
        current = prev.get(&node).and_then(|p| p.clone());
    }

    path.reverse();
    Some(path)
}

/// Find best path between two nodes with details
pub fn find_best_path(node_a: &str, node_b: &str, brain: &Brain) -> Option<PathResult> {
    let path = dijkstra(node_a, node_b, brain)?;
    let relations = brain.get_all_relations();

    let mut path_details = Vec::new();
    let mut total_weight = 0.0;

    for i in 0..path.len() - 1 {
        let source = &path[i];
        let target = &path[i + 1];

        // Find the edge weight
        for rel in relations.values() {
            if (rel.source == *source && rel.target == *target)
                || (rel.source == *target && rel.target == *source)
            {
                path_details.push(PathEdge {
                    from: source.clone(),
                    to: target.clone(),
                    weight: rel.weight,
                });
                total_weight += rel.weight;
                break;
            }
        }
    }

    Some(PathResult {
        path,
        path_details,
        total_weight,
    })
}

// =============================================================================
// Answer Generation
// =============================================================================

/// Aggregate paths into answer
pub fn aggregate_answer(paths: &[Vec<String>], brain: &Brain, question: &str) -> Answer {
    if paths.is_empty() {
        return Answer {
            answer: format!(
                "我暂时没有找到与\"{}\"相关的信息。\n\n你可以使用 `./seed init \"概念名\"` 来帮助我学习新知识。",
                question
            ),
            confidence: 0,
            paths: vec![],
            concepts: vec![],
            associations: None,
            elapsed_ms: 0,
        };
    }

    let relations = brain.get_all_relations();

    // Collect all concepts from paths
    let mut all_concepts = HashSet::new();
    let mut all_relations: Vec<(String, String, f64)> = Vec::new();

    for path in paths {
        for i in 0..path.len() - 1 {
            all_concepts.insert(&path[i]);
            all_concepts.insert(&path[i + 1]);

            // Find relation
            for rel in relations.values() {
                if (rel.source == path[i] && rel.target == path[i + 1])
                    || (rel.source == path[i + 1] && rel.target == path[i])
                {
                    all_relations.push((rel.source.clone(), rel.target.clone(), rel.weight));
                    break;
                }
            }
        }
    }

    let main_concepts: Vec<_> = all_concepts.iter().take(5).cloned().collect();

    // Use response generator for human-readable answer
    let main_concepts_set: HashSet<_> = main_concepts.iter().cloned().collect();
    let related: Vec<(String, f64)> = all_relations
        .iter()
        .filter(|(s, t, _)| main_concepts_set.contains(s) || main_concepts_set.contains(t))
        .map(|(s, t, w)| {
            if main_concepts_set.contains(s) {
                (t.clone(), *w)
            } else {
                (s.clone(), *w)
            }
        })
        .collect();

    // Get description for first concept
    let description = main_concepts.first()
        .and_then(|c| brain.get_concept(*c))
        .and_then(|c| c.description);

    let answer = if main_concepts.len() == 1 {
        response_generator::generate_single_concept_answer(main_concepts[0], &related, description.as_deref())
    } else {
        response_generator::generate_paragraph(&all_relations, &main_concepts[0])
    };

    // Calculate confidence
    let avg_weight = if all_relations.is_empty() {
        0.5
    } else {
        all_relations.iter().map(|(_, _, w)| w).sum::<f64>() / all_relations.len() as f64
    };

    Answer {
        answer,
        confidence: (avg_weight * 100.0) as u32,
        paths: paths.iter().take(5).cloned().collect(),
        concepts: main_concepts.iter().map(|s| s.to_string()).collect(),
        associations: None,
        elapsed_ms: 0,
    }
}

/// Main query function
pub fn query(question: &str, brain: &Brain) -> Answer {
    let query_words = parse_query(question);

    if query_words.is_empty() {
        return Answer {
            answer: "请告诉我你想了解什么？".to_string(),
            confidence: 0,
            paths: vec![],
            concepts: vec![],
            associations: None,
            elapsed_ms: 0,
        };
    }

    println!("[Query] Parsed words: {:?}", query_words);

    let matches = find_matching_concepts(&query_words, brain);
    println!(
        "[Query] Matched concepts: {:?}",
        matches.iter().map(|m| &m.concept_name).collect::<Vec<_>>()
    );

    if matches.is_empty() {
        return Answer {
            answer: format!("我还不了解\"{}\"相关的知识。要我学习一下吗？", query_words[0]),
            confidence: 0,
            paths: vec![],
            concepts: vec![],
            associations: None,
            elapsed_ms: 0,
        };
    }

    // Find paths from matched concepts
    let mut all_paths = Vec::new();
    let mut processed = HashSet::new();

    for m in &matches {
        if !processed.contains(&m.concept_name) {
            let paths = find_paths(&m.concept_name, brain, 2);
            all_paths.extend(paths);
            processed.insert(m.concept_name.clone());
        }
    }

    all_paths.dedup();
    aggregate_answer(&all_paths, brain, question)
}

/// Ask with enhanced features (Wikipedia integration optional)
pub fn ask(question: &str, brain: &Brain) -> Answer {
    let query_words = parse_query(question);
    let matches = find_matching_concepts(&query_words, brain);

    if matches.is_empty() {
        return Answer {
            answer: format!(
                "我暂时不太理解\"{}\"的含义。\n\n你可以试着问我一些其他问题，或者使用以下命令帮助我学习：\n• ./seed init \"概念名\" - 从网络学习\n• ./seed learn-file <文件> - 从文件学习",
                question
            ),
            confidence: 0,
            paths: vec![],
            concepts: vec![],
            associations: None,
            elapsed_ms: 0,
        };
    }

    // Get unique entities
    let unique_entities: HashSet<String> = matches.iter().map(|m| m.concept_name.clone()).collect();
    let unique_entities: Vec<String> = unique_entities.into_iter().collect();

    // Single concept: free association
    if unique_entities.len() == 1 {
        return answer_single_concept(&unique_entities[0], brain);
    }

    // Multiple concepts: path finding
    answer_multi_concept(&unique_entities, brain)
}

/// Generate answer for a single concept
fn answer_single_concept(concept: &str, brain: &Brain) -> Answer {
    // Get the description for the concept
    let description = brain.get_concept(concept)
        .and_then(|c| c.description);

    let relations = brain.get_relations_for_concept(concept);
    let mut connections: Vec<_> = relations
        .iter()
        .map(|r| {
            let target = if r.source == concept {
                &r.target
            } else {
                &r.source
            };
            (target.clone(), r.weight)
        })
        .collect();

    connections.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top5: Vec<_> = connections.iter().take(5).map(|(t, w)| (t.clone(), *w)).collect();

    let answer = response_generator::generate_single_concept_answer(concept, &top5, description.as_deref());

    Answer {
        answer,
        confidence: if !top5.is_empty() { (top5[0].1 * 100.0) as u32 } else { 0 },
        associations: Some(
            top5.iter()
                .map(|(t, w)| Association {
                    concept: t.clone(),
                    weight: *w,
                })
                .collect(),
        ),
        concepts: vec![concept.to_string()],
        paths: vec![],
        elapsed_ms: 0,
    }
}

/// Generate answer for multiple concepts
fn answer_multi_concept(entities: &[String], brain: &Brain) -> Answer {
    let mut all_paths = Vec::new();
    let mut all_path_details: Vec<(String, String, f64)> = Vec::new();

    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            if let Some(path_result) = find_best_path(&entities[i], &entities[j], brain) {
                for edge in &path_result.path_details {
                    all_path_details.push((edge.from.clone(), edge.to.clone(), edge.weight));
                }
                all_paths.push(path_result);
            }
        }
    }

    let answer = if all_paths.is_empty() {
        format!(
            "\"{}\"和\"{}\"是两个不同的概念。目前我还没有发现它们之间的直接关联。\n\n如果你能告诉我更多关于它们的关系，我可以更好地理解。",
            entities[0],
            entities.get(1).map(|s| s.as_str()).unwrap_or("其他概念")
        )
    } else {
        // Sort by total weight
        all_paths.sort_by(|a, b| {
            b.total_weight
                .partial_cmp(&a.total_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best = &all_paths[0];

        response_generator::generate_multi_concept_answer(
            entities,
            &best.path,
            &best
                .path_details
                .iter()
                .map(|e| (e.from.clone(), e.to.clone(), e.weight))
                .collect::<Vec<_>>(),
        )
    };

    let confidence = if !all_paths.is_empty() && !all_paths[0].path.is_empty() {
        (all_paths[0].total_weight * 100.0 / all_paths[0].path.len() as f64) as u32
    } else {
        0
    };

    Answer {
        answer,
        confidence,
        paths: all_paths.iter().take(3).map(|p| p.path.clone()).collect(),
        concepts: entities.to_vec(),
        associations: None,
        elapsed_ms: 0,
    }
}

// =============================================================================
// Data Types
// =============================================================================

#[derive(Debug, Clone, serde::Serialize)]
pub struct Match {
    pub word: String,
    pub concept_name: String,
    pub energy: f64,
    pub exact: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Answer {
    pub answer: String,
    pub confidence: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub concepts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associations: Option<Vec<Association>>,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Association {
    pub concept: String,
    pub weight: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PathResult {
    pub path: Vec<String>,
    pub path_details: Vec<PathEdge>,
    pub total_weight: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PathEdge {
    pub from: String,
    pub to: String,
    pub weight: f64,
}
