// Brain - Knowledge Graph Data Structures
//
// This module defines the core data structures for the knowledge graph:
// - Concept: A node representing a learned concept with energy and metadata
// - Relation: An edge connecting two concepts with a weight
// - Brain: The knowledge graph container with concepts and relations

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// Data Structures
// =============================================================================

/// Knowledge graph concept node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub energy: f64,
    pub count: u32,
    #[serde(rename = "firstSeen")]
    pub first_seen: String,
    #[serde(rename = "lastSeen")]
    pub last_seen: String,
}

/// Knowledge graph relation edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub count: u32,
    #[serde(rename = "last_updated")]
    pub last_updated: u64,
}

/// Metadata for the brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMeta {
    #[serde(rename = "totalConcepts")]
    pub total_concepts: u32,
    #[serde(rename = "totalRelations")]
    pub total_relations: u32,
    #[serde(rename = "totalLearnCount")]
    pub total_learn_count: u32,
}

/// Knowledge graph (brain) - the main data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brain {
    pub version: String,
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<String>,
    pub concepts: HashMap<String, Concept>,
    pub relations: HashMap<String, Relation>,
    pub meta: BrainMeta,
}

// =============================================================================
// Brain Implementation
// =============================================================================

impl Default for Brain {
    fn default() -> Self {
        Self::new()
    }
}

impl Brain {
    /// Create a new empty brain
    pub fn new() -> Self {
        Self {
            version: "2.0".to_string(),
            last_update: None,
            concepts: HashMap::new(),
            relations: HashMap::new(),
            meta: BrainMeta {
                total_concepts: 0,
                total_relations: 0,
                total_learn_count: 0,
            },
        }
    }

    /// Load brain from file, returns default if file doesn't exist or is invalid
    pub fn load(path: &PathBuf) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    /// Save brain to file
    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// Get existing concept or create a new one
    pub fn get_or_create_concept(&mut self, name: &str) -> &mut Concept {
        let now = current_timestamp();
        self.concepts.entry(name.to_string()).or_insert_with(|| Concept {
            energy: 0.1,
            count: 1,
            first_seen: now.clone(),
            last_seen: now,
        })
    }

    /// Find a relation between two concepts
    #[allow(dead_code)]
    pub fn get_relation(&self, source: &str, target: &str) -> Option<&Relation> {
        let key = make_relation_key(source, target);
        self.relations.values().find(|r| {
            make_relation_key(&r.source, &r.target) == key
        })
    }

    /// Find a mutable relation between two concepts
    pub fn get_relation_mut(&mut self, source: &str, target: &str) -> Option<&mut Relation> {
        let key = make_relation_key(source, target);
        self.relations.values_mut().find(|r| {
            make_relation_key(&r.source, &r.target) == key
        })
    }

    /// Get all relations involving a concept
    #[allow(dead_code)]
    pub fn get_relations_for_concept(&self, concept: &str) -> Vec<&Relation> {
        self.relations
            .values()
            .filter(|r| r.source == concept || r.target == concept)
            .collect()
    }

    /// Add a new relation if it doesn't exist
    pub fn add_or_update_relation(&mut self, source: &str, target: &str) {
        let key = make_relation_key(source, target);
        let exists = self.relations.values().any(|r| {
            make_relation_key(&r.source, &r.target) == key
        });

        if !exists {
            let id = format!("rel_{}", key.replace("|||", "_"));
            self.relations.insert(id.clone(), Relation {
                id,
                source: source.to_string(),
                target: target.to_string(),
                weight: 0.0,
                count: 0,
                last_updated: current_millis(),
            });
        }
    }

    /// Cleanup weak relations and low-energy concepts
    pub fn cleanup(&mut self, min_weight: f64, min_energy: f64, aggressive: bool) -> (u32, u32) {
        let (pruned_relations, connected) = self.cleanup_relations(min_weight);
        let pruned_concepts = self.cleanup_concepts(min_energy, aggressive, &connected);
        (pruned_relations, pruned_concepts)
    }

    /// Clear all concepts and relations
    pub fn clear(&mut self) {
        self.concepts.clear();
        self.relations.clear();
        self.meta = BrainMeta {
            total_concepts: 0,
            total_relations: 0,
            total_learn_count: 0,
        };
        self.last_update = None;
    }

    /// Get total number of concepts
    pub fn total_concepts(&self) -> usize {
        self.concepts.len()
    }

    /// Get total number of relations
    pub fn total_relations(&self) -> usize {
        self.relations.len()
    }

    // -------------------------------------------------------------------------
    // Private helper methods
    // -------------------------------------------------------------------------

    /// Cleanup weak relations, returns (pruned_count, connected_concepts)
    fn cleanup_relations(&mut self, min_weight: f64) -> (u32, HashSet<String>) {
        // Find connected concepts
        let mut connected: HashSet<String> = HashSet::new();
        for rel in self.relations.values() {
            connected.insert(rel.source.clone());
            connected.insert(rel.target.clone());
        }

        // Apply decay
        for r in self.relations.values_mut() {
            r.weight *= 0.95;
            r.last_updated = current_millis();
        }

        // Remove weak relations
        let to_delete: Vec<String> = self.relations
            .iter()
            .filter(|(_, r)| r.weight < min_weight)
            .map(|(id, _)| id.clone())
            .collect();

        let pruned = to_delete.len() as u32;
        for id in &to_delete {
            self.relations.remove(id);
        }

        (pruned, connected)
    }

    /// Cleanup low-energy concepts
    fn cleanup_concepts(&mut self, min_energy: f64, aggressive: bool, connected: &HashSet<String>) -> u32 {
        // Apply decay
        for c in self.concepts.values_mut() {
            c.energy *= 0.95;
            c.last_seen = current_timestamp();
        }

        // Remove low-energy or unconnected concepts
        let to_delete: Vec<String> = self.concepts
            .iter()
            .filter(|(name, c)| {
                c.energy < min_energy ||
                (aggressive && !connected.contains(*name) && !self.relations.is_empty())
            })
            .map(|(name, _)| name.clone())
            .collect();

        let pruned = to_delete.len() as u32;
        for name in &to_delete {
            self.concepts.remove(name);
        }

        pruned
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a normalized key for a relation (order-independent)
fn make_relation_key(source: &str, target: &str) -> String {
    let mut parts = [source, target];
    parts.sort();
    parts.join("|||")
}

/// Get current timestamp as seconds since Unix epoch
fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_default()
}

/// Get current time as milliseconds since Unix epoch
fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Get the default brain data path
pub fn default_brain_path() -> PathBuf {
    directories::ProjectDirs::from("com", "seed-intelligence", "Seed-Intelligence")
        .map(|proj_dirs| {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            data_dir.join("brain.json")
        })
        .unwrap_or_else(|| PathBuf::from("brain.json"))
}
