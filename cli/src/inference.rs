// Inference Engine - Graph-based Q&A using path aggregation

use crate::brain::Brain;
use crate::nlp::{filter_stop_words, tokenize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Parse user question into keywords
pub fn parse_query(question: &str) -> Vec<String> {
    let tokens = tokenize(question);
    filter_stop_words(&tokens)
}

/// Find matching concepts in brain
pub fn find_matching_concepts(query_words: &[String], brain: &Brain) -> Vec<Match> {
    let mut matches = Vec::new();
    let mut matched_names = HashSet::new();

    for word in query_words {
        let lower_word = word.to_lowercase();

        // Exact match (priority)
        if let Some(concept) = brain.concepts.get(word) {
            if !matched_names.contains(word) {
                matches.push(Match {
                    word: word.clone(),
                    concept_name: word.clone(),
                    energy: concept.energy,
                    exact: true,
                });
                matched_names.insert(word.clone());
            }
        }

        // Fuzzy match (only if no exact match)
        if !matched_names.contains(word) {
            for (concept_name, concept) in &brain.concepts {
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
                }
            }
        }
    }

    matches
}

/// Build adjacency list from brain
fn build_adjacency(brain: &Brain) -> HashMap<String, Vec<(String, f64)>> {
    let mut adj: HashMap<String, Vec<(String, f64)>> = HashMap::new();

    for rel in brain.relations.values() {
        adj.entry(rel.source.clone()).or_default()
            .push((rel.target.clone(), rel.weight));
        adj.entry(rel.target.clone()).or_default()
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

/// Dijkstra to find best path between two nodes
pub fn dijkstra(start: &str, end: &str, brain: &Brain) -> Option<Vec<String>> {
    let adj = build_adjacency(brain);

    let mut dist: HashMap<String, f64> = HashMap::new();
    let mut prev: HashMap<String, Option<String>> = HashMap::new();
    let mut visited = HashSet::new();

    // Initialize
    for node in brain.concepts.keys() {
        dist.insert(node.clone(), f64::NEG_INFINITY);
    }
    dist.insert(start.to_string(), 0.0);

    // Priority queue (using simple Vec and sort)
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

    let mut path = Vec::new();
    let mut current = Some(end.to_string());

    while let Some(node) = current {
        path.push(node.clone());
        current = prev.get(&node).and_then(|p| p.clone());
    }

    path.reverse();
    if path.first() == Some(&start.to_string()) {
        Some(path)
    } else {
        None
    }
}

/// Find best path between two nodes with details
pub fn find_best_path(node_a: &str, node_b: &str, brain: &Brain) -> Option<PathResult> {
    let path = dijkstra(node_a, node_b, brain)?;

    let mut path_details = Vec::new();
    let mut total_weight = 0.0;

    for i in 0..path.len() - 1 {
        let source = &path[i];
        let target = &path[i + 1];

        // Find the edge
        for rel in brain.relations.values() {
            if (rel.source.as_str() == source.as_str() && rel.target.as_str() == target.as_str())
                || (rel.source.as_str() == target.as_str() && rel.target.as_str() == source.as_str())
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

/// Aggregate paths into answer
pub fn aggregate_answer(paths: &[Vec<String>], brain: &Brain, _question: &str) -> Answer {
    if paths.is_empty() {
        return Answer {
            answer: "我的知识库中还没有关于这个概念的信息。".to_string(),
            confidence: 0,
            paths: vec![],
            concepts: vec![],
            associations: None,
            elapsed_ms: 0,
        };
    }

    // Collect all concepts from paths
    let mut all_concepts = HashSet::new();
    let mut all_relations = Vec::new();

    for path in paths {
        for i in 0..path.len() - 1 {
            all_concepts.insert(&path[i]);
            all_concepts.insert(&path[i + 1]);

            // Find relation
            for rel in brain.relations.values() {
                if (rel.source.as_str() == path[i].as_str() && rel.target.as_str() == path[i + 1].as_str())
                    || (rel.source.as_str() == path[i + 1].as_str() && rel.target.as_str() == path[i].as_str())
                {
                    all_relations.push(rel);
                    break;
                }
            }
        }
    }

    let main_concepts: Vec<_> = all_concepts.iter().take(5).cloned().collect();

    let answer = if main_concepts.len() == 1 {
        let concept = brain.concepts.get(*main_concepts.first().unwrap());
        if let Some(c) = concept {
            format!(
                "关于\"{}\"，据我所知：\n这是一个重要概念，能量值为 {:.2}，出现在 {} 个上下文中。",
                main_concepts.first().unwrap(),
                c.energy,
                c.count
            )
        } else {
            format!("关于\"{}\"，这是一个重要的概念。", main_concepts.first().unwrap())
        }
    } else {
        let concepts_str = main_concepts.iter().take(3).map(|s| s.to_string()).collect::<Vec<_>>().join("、");
        let mut ans = format!("根据我的知识图谱，{} 等概念相互关联。\n\n", concepts_str);

        if !all_relations.is_empty() {
            let mut sorted_rels: Vec<_> = all_relations.iter().collect();
            sorted_rels.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));

            ans += "它们的关系：\n";
            for r in sorted_rels.iter().take(3) {
                ans += &format!(
                    "• {} ↔ {} (关联度: {:.0}%)\n",
                    r.source,
                    r.target,
                    r.weight * 100.0
                );
            }
        }

        ans
    };

    // Calculate confidence
    let avg_weight = if all_relations.is_empty() {
        0.5
    } else {
        all_relations.iter().map(|r| r.weight).sum::<f64>() / all_relations.len() as f64
    };

    Answer {
        answer,
        confidence: (avg_weight * 100.0) as u32,
        paths: paths.iter().take(5).map(|p| p.clone()).collect(),
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
    println!("[Query] Matched concepts: {:?}", matches.iter().map(|m| &m.concept_name).collect::<Vec<_>>());

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

    // Deduplicate
    all_paths.dedup();

    aggregate_answer(&all_paths, brain, question)
}

/// Ask with enhanced features (Wikipedia integration optional)
pub fn ask(question: &str, brain: &Brain) -> Answer {
    let query_words = parse_query(question);
    let matches = find_matching_concepts(&query_words, brain);

    if matches.is_empty() {
        return Answer {
            answer: format!("我的知识库中还没有关于\"{}\"的信息。要我学习一下吗？", question),
            confidence: 0,
            paths: vec![],
            concepts: vec![],
            associations: None,
            elapsed_ms: 0,
        };
    }

    // Get unique entities
    let unique_entities: Vec<_> = matches.iter().map(|m| m.concept_name.clone()).collect();
    let unique_entities: Vec<_> = unique_entities.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();

    // Single concept: free association
    if unique_entities.len() == 1 {
        let concept = &unique_entities[0];

        // Find top connections
        let mut connections: Vec<_> = brain.relations.iter()
            .filter(|(_, r)| r.source.as_str() == concept.as_str() || r.target.as_str() == concept.as_str())
            .map(|(_, r)| {
                let target = if r.source.as_str() == concept.as_str() { &r.target } else { &r.source };
                (target.clone(), r.weight)
            })
            .collect();

        connections.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top5: Vec<_> = connections.iter().take(5).collect();

        let answer = if top5.is_empty() {
            format!("关于\"{}\"，我只知道这一个概念，还没有发现它与其他概念的关联。", concept)
        } else {
            let mut ans = format!("关于\"{}\"，我联想到以下概念：\n", concept);
            for (i, (target, weight)) in top5.iter().enumerate() {
                ans += &format!("{}. {} (关联度: {:.0}%)\n", i + 1, target, weight * 100.0);
            }
            ans
        };

        return Answer {
            answer,
            confidence: if !top5.is_empty() { (top5[0].1 * 100.0) as u32 } else { 0 },
            associations: Some(top5.iter().map(|(t, w)| Association {
                concept: (*t).clone(),
                weight: *w,
            }).collect()),
            concepts: vec![concept.clone()],
            paths: vec![],
            elapsed_ms: 0,
        };
    }

    // Multiple concepts: path finding
    let mut all_paths = Vec::new();

    for i in 0..unique_entities.len() {
        for j in (i + 1)..unique_entities.len() {
            let node_a = &unique_entities[i];
            let node_b = &unique_entities[j];

            if let Some(path_result) = find_best_path(node_a, node_b, brain) {
                all_paths.push(PathResult {
                    path: path_result.path,
                    path_details: path_result.path_details,
                    total_weight: path_result.total_weight,
                });
            }
        }
    }

    let answer = if all_paths.is_empty() {
        format!("我找到了概念: {}，但它们之间还没有建立关联路径。",
            unique_entities.join("、"))
    } else {
        // Sort by total weight
        all_paths.sort_by(|a, b| b.total_weight.partial_cmp(&a.total_weight).unwrap_or(std::cmp::Ordering::Equal));
        let best = &all_paths[0];

        let mut ans = "我找到了逻辑链：\n".to_string();
        let mut current_node = &best.path[0];

        for edge in &best.path_details {
            ans += &format!(
                "• {} (权重 {:.0}%) -> {}\n",
                current_node,
                edge.weight * 100.0,
                edge.to
            );
            current_node = &edge.to;
        }

        ans += &format!("\n总关联度: {:.0}%", best.total_weight * 100.0);
        ans
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
        concepts: unique_entities,
        associations: None,
        elapsed_ms: 0,
    }
}

// Types
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