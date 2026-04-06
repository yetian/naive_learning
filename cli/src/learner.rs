// IncrementalLearner - Hebbian Learning Engine
// Based on: "Neurons that fire together, wire together"

use crate::brain::Brain;
use crate::nlp::Tokenizer;
use std::path::PathBuf;
use std::time::Instant;
use std::collections::{HashMap, HashSet};

// Learning parameters
const WINDOW_SIZE: usize = 6;
const MIN_WEIGHT: f64 = 0.01;
const MIN_ENERGY: f64 = 0.1;
const ENERGY_PER_MENTION: f64 = 0.1;
const FOCUS_BOOST: f64 = 2.0;
const MAX_TEXT_LENGTH: usize = 50000;

/// In-memory batch for accumulating updates before writing to DB
#[derive(Default)]
pub struct LearningBatch {
    /// Concept name -> (energy_delta, count_delta)
    pub concepts: HashMap<String, (f64, u32)>,
    /// Relation key (source|||target) -> (weight_delta, count_delta, source, target)
    pub relations: HashMap<String, (f64, u32, String, String)>,
}

/// Create a normalized key for a relation (order-independent)
fn make_relation_key(a: &str, b: &str) -> String {
    if a < b {
        format!("{}|||{}", a, b)
    } else {
        format!("{}|||{}", b, a)
    }
}

pub struct IncrementalLearner {
    pub brain: Brain,
    pub brain_path: PathBuf,
    tokenizer: Tokenizer,
    /// Current book ID being learned (for tracking)
    current_book_id: Option<i64>,
}

impl IncrementalLearner {
    pub fn new(brain_path: Option<PathBuf>) -> Self {
        let path = brain_path.unwrap_or_else(crate::brain::default_brain_path);
        let brain = Brain::new(&path).expect("Failed to initialize brain database");
        let tokenizer = crate::nlp::get_tokenizer();

        Self {
            brain,
            brain_path: path,
            tokenizer,
            current_book_id: None,
        }
    }

    /// Initialize with auto-migration from JSON
    pub fn init() -> Self {
        let brain = crate::brain::init_brain().expect("Failed to initialize brain");
        let tokenizer = crate::nlp::get_tokenizer();
        let brain_path = crate::brain::default_brain_path();

        Self {
            brain,
            brain_path,
            tokenizer,
            current_book_id: None,
        }
    }

    /// Start learning from a book (sets current book context)
    pub fn start_book(&mut self, book_id: i64) {
        self.current_book_id = Some(book_id);
    }

    /// Stop learning from current book
    pub fn end_book(&mut self) {
        self.current_book_id = None;
    }

    /// Get current book ID
    pub fn current_book_id(&self) -> Option<i64> {
        self.current_book_id
    }

    /// Learn from text with optional focus concept (ontology anchoring)
    pub fn learn_from_text(&mut self, text: &str, focus_concept: Option<&str>) -> LearnResult {
        let start = Instant::now();

        // Preprocess text
        let text = preprocess_text(text);
        let tokens = self.tokenizer.tokenize(&text);

        println!("[Learner] Processing {} tokens, focus: {:?}", tokens.len(), focus_concept);

        // Use in-memory batch to accumulate updates
        let mut batch = LearningBatch::default();
        let mut concepts_in_batch = HashSet::new();

        // Ensure focus concept exists (for multi-word concepts)
        if let Some(focus) = focus_concept {
            batch.concepts.entry(focus.to_string())
                .and_modify(|(e, c)| { *e += ENERGY_PER_MENTION * 2.0; *c += 1; })
                .or_insert((ENERGY_PER_MENTION * 2.0, 1));
            concepts_in_batch.insert(focus.to_string());
        }

        // Sliding window processing - all in memory
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

                    // Calculate weight delta
                    let weight_delta = distance_decay * focus_boost * 0.1;

                    // Create normalized relation key
                    let key = make_relation_key(word_a, word_b);
                    let (source, target) = if word_a < word_b {
                        (word_a.to_string(), word_b.to_string())
                    } else {
                        (word_b.to_string(), word_a.to_string())
                    };

                    batch.relations.entry(key)
                        .and_modify(|(w, c, _, _)| { *w += weight_delta; *c += 1; })
                        .or_insert((weight_delta, 1, source, target));
                }
            }

            // Accumulate energy updates in memory
            for token in &window {
                if is_valid_token(token) {
                    let mut energy_delta = ENERGY_PER_MENTION;

                    // Focus concept bonus
                    if let Some(focus) = focus_concept {
                        if token.to_lowercase().contains(&focus.to_lowercase()) {
                            energy_delta += ENERGY_PER_MENTION;
                        }
                    }

                    batch.concepts.entry(token.to_string())
                        .and_modify(|(e, c)| { *e += energy_delta; *c += 1; })
                        .or_insert((energy_delta, 1));

                    concepts_in_batch.insert(token.to_string());
                }
            }
        }

        // Single transaction for all database writes
        let (added_relations, updated_concepts) = self.brain.apply_batch(batch);

        // Track book concepts
        for concept in concepts_in_batch {
            self.track_book_concept(&concept);
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

    /// Track concept in current book
    fn track_book_concept(&mut self, concept_name: &str) {
        if let Some(book_id) = self.current_book_id {
            self.brain.track_book_concept(book_id, concept_name);
        }
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
        let concepts = self.brain.get_all_concepts();
        let relations = self.brain.get_all_relations();

        let avg_weight = if relations.is_empty() {
            0.0
        } else {
            relations.values().map(|r| r.weight).sum::<f64>() / relations.len() as f64
        };

        let avg_energy = if concepts.is_empty() {
            0.0
        } else {
            concepts.values().map(|c| c.energy).sum::<f64>() / concepts.len() as f64
        };

        let mut top_concepts: Vec<_> = concepts.iter()
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
    pub fn get_concept(&self, name: &str) -> Option<crate::brain::Concept> {
        self.brain.get_concept(name)
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

        let relations = self.brain.get_relations_for_concept(current);
        for rel in &relations {
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

    /// Set description for a concept (useful for storing Wikipedia summaries)
    pub fn set_concept_description(&mut self, name: &str, description: &str) {
        self.brain.set_concept_description(name, description);
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
