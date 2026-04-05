// IncrementalLearner - Hebbian Learning Engine
// Based on: "Neurons that fire together, wire together"

use crate::brain::Brain;
use crate::nlp::Tokenizer;
use std::time::Instant;

// Learning parameters
const WINDOW_SIZE: usize = 6;
const MIN_WEIGHT: f64 = 0.01;
const MIN_ENERGY: f64 = 0.1;
const ENERGY_PER_MENTION: f64 = 0.1;
const FOCUS_BOOST: f64 = 2.0;
const MAX_TEXT_LENGTH: usize = 50000;

pub struct IncrementalLearner {
    pub brain: Brain,
    pub brain_path: std::path::PathBuf,
    tokenizer: Tokenizer,
}

impl IncrementalLearner {
    pub fn new(brain_path: Option<std::path::PathBuf>) -> Self {
        let path = brain_path.unwrap_or_else(crate::brain::default_brain_path);
        let brain = Brain::load(&path);
        let tokenizer = crate::nlp::get_tokenizer();

        Self {
            brain,
            brain_path: path,
            tokenizer,
        }
    }

    pub fn load(&mut self) {
        self.brain = Brain::load(&self.brain_path);
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        self.brain.save(&self.brain_path)
    }

    /// Learn from text with optional focus concept (ontology anchoring)
    pub fn learn_from_text(&mut self, text: &str, focus_concept: Option<&str>) -> LearnResult {
        let start = Instant::now();

        // Preprocess text
        let text = preprocess_text(text);
        let tokens = self.tokenizer.tokenize(&text);

        println!("[Learner] Processing {} tokens, focus: {:?}", tokens.len(), focus_concept);

        let mut added_relations = 0u32;
        let mut updated_concepts = 0u32;

        // Sliding window processing
        for i in 0..tokens.len().saturating_sub(1) {
            let window: Vec<_> = tokens[i..].iter().take(WINDOW_SIZE).collect();

            // Calculate relations for each pair in window
            for j in 0..window.len() {
                for k in (j + 1)..window.len() {
                    let word_a = window[j];
                    let word_b = window[k];

                    // Skip invalid tokens
                    if !is_valid_token(word_a) || !is_valid_token(word_b) {
                        continue;
                    }

                    // Calculate distance decay
                    let distance = k - j;
                    let distance_decay = 1.0 / (1.0 + distance as f64 * 0.5);

                    // Focus concept boost
                    let focus_boost = calculate_focus_boost(word_a, word_b, focus_concept);

                    // Update relation
                    if self.update_relation(word_a, word_b, distance_decay, focus_boost) {
                        added_relations += 1;
                    }
                }
            }

            // Update energy for each token in window
            for token in &window {
                if is_valid_token(token) {
                    if self.update_concept_energy(token, focus_concept) {
                        updated_concepts += 1;
                    }
                }
            }
        }

        let elapsed = start.elapsed().as_millis();
        let performance = if elapsed < 100 { "excellent" }
            else if elapsed < 500 { "good" }
            else { "slow" };

        println!("[Learner] Completed in {}ms, relations: {}, concepts: {}",
            elapsed, added_relations, updated_concepts);

        LearnResult {
            success: true,
            tokens_processed: tokens.len(),
            relations_added: added_relations,
            concepts_updated: updated_concepts,
            elapsed_ms: elapsed,
            performance: performance.to_string(),
        }
    }

    /// Update relation weight using logarithmic growth
    fn update_relation(&mut self, source: &str, target: &str, distance_decay: f64, focus_boost: f64) -> bool {
        // Add or get relation
        self.brain.add_or_update_relation(source, target);

        // Get mutable reference
        if let Some(rel) = self.brain.get_relation_mut(source, target) {
            // Logarithmic growth
            rel.count += 1;
            let log_growth = (rel.count as f64 + 1.0).ln();

            // New weight = old weight + log_growth * distance_decay * focus_boost
            let weight_increment = log_growth * distance_decay * focus_boost * 0.1;
            rel.weight = (rel.weight + weight_increment).min(1.0);
            rel.last_updated = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            true
        } else {
            false
        }
    }

    /// Update concept energy
    fn update_concept_energy(&mut self, token: &str, focus_concept: Option<&str>) -> bool {
        let concept = self.brain.get_or_create_concept(token);

        // Add energy
        concept.energy += ENERGY_PER_MENTION;
        concept.count += 1;
        concept.last_seen = format!("{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0));

        // Focus concept bonus
        if let Some(focus) = focus_concept {
            if token.to_lowercase().contains(&focus.to_lowercase()) {
                concept.energy += ENERGY_PER_MENTION;
            }
        }

        true
    }

    /// Cleanup - decay and prune weak connections
    pub fn cleanup(&mut self, aggressive: bool) -> CleanupResult {
        let (pruned_relations, pruned_concepts) =
            self.brain.cleanup(MIN_WEIGHT, MIN_ENERGY, aggressive);

        println!("[Learner] Cleanup: pruned {} relations, {} concepts",
            pruned_relations, pruned_concepts);

        CleanupResult {
            pruned_relations,
            pruned_concepts,
            remaining_relations: self.brain.total_relations() as u32,
            remaining_concepts: self.brain.total_concepts() as u32,
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> Stats {
        let avg_weight = if self.brain.relations.is_empty() {
            0.0
        } else {
            self.brain.relations.values().map(|r| r.weight).sum::<f64>()
                / self.brain.relations.len() as f64
        };

        let avg_energy = if self.brain.concepts.is_empty() {
            0.0
        } else {
            self.brain.concepts.values().map(|c| c.energy).sum::<f64>()
                / self.brain.concepts.len() as f64
        };

        let mut top_concepts: Vec<_> = self.brain.concepts.iter()
            .map(|(name, c)| (name.clone(), c.energy, c.count))
            .collect();
        top_concepts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_concepts: Vec<_> = top_concepts.iter().take(10)
            .map(|(name, energy, count)| TopConcept {
                name: name.clone(),
                energy: *energy,
                count: *count,
            })
            .collect();

        Stats {
            total_concepts: self.brain.total_concepts() as u32,
            total_relations: self.brain.total_relations() as u32,
            avg_weight,
            avg_energy,
            top_concepts,
        }
    }

    /// Get concept info
    pub fn get_concept(&self, name: &str) -> Option<&crate::brain::Concept> {
        self.brain.concepts.get(name)
    }

    /// Get related concepts (graph traversal)
    pub fn get_related_concepts(&self, name: &str, max_depth: usize) -> Vec<(String, f64)> {
        let mut related = std::collections::HashMap::new();
        let mut visited = std::collections::HashSet::new();

        self.traverse(name, 0, max_depth, &mut visited, &mut related);

        let mut items: Vec<_> = related.into_iter().collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items
    }

    fn traverse(
        &self,
        current: &str,
        depth: usize,
        max_depth: usize,
        visited: &mut std::collections::HashSet<String>,
        related: &mut std::collections::HashMap<String, f64>,
    ) {
        if depth > max_depth || visited.contains(current) {
            return;
        }
        visited.insert(current.to_string());

        for rel in self.brain.relations.values() {
            let neighbor = if rel.source == current {
                Some(rel.target.as_str())
            } else if rel.target == current {
                Some(rel.source.as_str())
            } else {
                None
            };

            if let Some(n) = neighbor {
                if !visited.contains(n) {
                    related.insert(n.to_string(), rel.weight);
                    self.traverse(n, depth + 1, max_depth, visited, related);
                }
            }
        }
    }

    /// Clear knowledge base
    pub fn clear(&mut self) {
        self.brain.clear();
    }
}

/// Preprocess text
fn preprocess_text(text: &str) -> String {
    let text = text.trim();
    let text = text.replace(|c: char| c.is_whitespace(), " ");
    if text.len() > MAX_TEXT_LENGTH {
        text.chars().take(MAX_TEXT_LENGTH).collect()
    } else {
        text.to_string()
    }
}

/// Check if token is valid
fn is_valid_token(token: &str) -> bool {
    if token.len() < 2 {
        return false;
    }

    // Filter punctuation using Unicode categories
    for c in token.chars() {
        // Skip common punctuation
        if c.is_whitespace() {
            continue;
        }
        // Check if it's a CJK punctuation
        let cp = c as u32;
        if (0x3000..=0x303F).contains(&cp) || // CJK Symbols and Punctuation
           (0xFF00..=0xFFEF).contains(&cp) || // Halfwidth and Fullwidth Forms
           c.is_ascii_punctuation() {
            return false;
        }
    }

    // Filter stop words
    let stop_words = ["的", "是", "在", "了", "和", "与", "或", "有", "这", "那",
        "the", "is", "are", "was", "been", "being", "have", "has", "and", "but",
        "它", "其", "此", "彼", "各", "每", "某"];

    !stop_words.contains(&token.to_lowercase().as_str())
}

/// Calculate focus boost
fn calculate_focus_boost(word_a: &str, word_b: &str, focus_concept: Option<&str>) -> f64 {
    if let Some(focus) = focus_concept {
        let focus = focus.to_lowercase();
        let a_match = word_a.to_lowercase().contains(&focus);
        let b_match = word_b.to_lowercase().contains(&focus);

        if a_match && b_match {
            FOCUS_BOOST
        } else if a_match || b_match {
            FOCUS_BOOST.sqrt()
        } else {
            1.0
        }
    } else {
        1.0
    }
}

// Result types
#[derive(Debug, Clone, serde::Serialize)]
pub struct LearnResult {
    pub success: bool,
    pub tokens_processed: usize,
    pub relations_added: u32,
    pub concepts_updated: u32,
    pub elapsed_ms: u128,
    pub performance: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CleanupResult {
    pub pruned_relations: u32,
    pub pruned_concepts: u32,
    pub remaining_relations: u32,
    pub remaining_concepts: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Stats {
    pub total_concepts: u32,
    pub total_relations: u32,
    pub avg_weight: f64,
    pub avg_energy: f64,
    pub top_concepts: Vec<TopConcept>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TopConcept {
    pub name: String,
    pub energy: f64,
    pub count: u32,
}