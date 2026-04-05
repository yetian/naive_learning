// Brain - Knowledge Graph Data Structures

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Knowledge graph (brain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brain {
    pub version: String,
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<String>,
    pub concepts: HashMap<String, Concept>,
    pub relations: HashMap<String, Relation>,
    pub meta: BrainMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMeta {
    #[serde(rename = "totalConcepts")]
    pub total_concepts: u32,
    #[serde(rename = "totalRelations")]
    pub total_relations: u32,
    #[serde(rename = "totalLearnCount")]
    pub total_learn_count: u32,
}

impl Default for Brain {
    fn default() -> Self {
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
}

impl Brain {
    pub fn load(path: &PathBuf) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(brain) = serde_json::from_str(&data) {
                return brain;
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn get_or_create_concept(&mut self, name: &str) -> &mut Concept {
        let now = iso_now();
        self.concepts.entry(name.to_string()).or_insert_with(|| Concept {
            energy: 0.1,
            count: 1,
            first_seen: now.clone(),
            last_seen: now,
        })
    }

    pub fn find_relation(&self, source: &str, target: &str) -> Option<&Relation> {
        let key = make_relation_key(source, target);
        self.relations.values().find(|r| {
            let k = make_relation_key(&r.source, &r.target);
            k == key
        })
    }

    pub fn find_relation_mut(&mut self, source: &str, target: &str) -> Option<&mut Relation> {
        let key = make_relation_key(source, target);
        self.relations.values_mut().find(|r| {
            let k = make_relation_key(&r.source, &r.target);
            k == key
        })
    }

    /// Find existing relation by source and target
    pub fn get_relation(&self, source: &str, target: &str) -> Option<&Relation> {
        let key = make_relation_key(source, target);
        self.relations.values().find(|r| {
            let k = make_relation_key(&r.source, &r.target);
            k == key
        })
    }

    /// Get all relations involving a concept
    pub fn get_relations_for_concept(&self, concept: &str) -> Vec<&Relation> {
        self.relations.values()
            .filter(|r| r.source.as_str() == concept || r.target.as_str() == concept)
            .collect()
    }

    /// Add or update a relation, return the relation ID
    pub fn add_or_update_relation(&mut self, source: &str, target: &str) {
        let key = make_relation_key(source, target);
        let id = format!("rel_{}", key.replace("|||", "_"));

        // Check if exists
        let exists = self.relations.values().any(|r| {
            make_relation_key(&r.source, &r.target) == key
        });

        if !exists {
            let rel = Relation {
                id: id.clone(),
                source: source.to_string(),
                target: target.to_string(),
                weight: 0.0,
                count: 0,
                last_updated: now_millis(),
            };
            self.relations.insert(id, rel);
        }
    }

    /// Get a mutable relation by source and target
    pub fn get_relation_mut(&mut self, source: &str, target: &str) -> Option<&mut Relation> {
        let key = make_relation_key(source, target);
        self.relations.values_mut().find(|r| {
            let k = make_relation_key(&r.source, &r.target);
            k == key
        })
    }

    pub fn cleanup(&mut self, min_weight: f64, min_energy: f64, aggressive: bool) -> (u32, u32) {
        let mut pruned_relations = 0u32;
        let mut pruned_concepts = 0u32;

        // Find connected concepts
        let mut connected = HashSet::new();
        for rel in self.relations.values() {
            connected.insert(rel.source.clone());
            connected.insert(rel.target.clone());
        }

        // Decay weights first
        for r in self.relations.values_mut() {
            r.weight *= 0.95;
            r.last_updated = now_millis();
        }

        // Collect IDs to delete
        let relations_to_delete: Vec<String> = self.relations.iter()
            .filter(|(_, r)| r.weight < min_weight)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &relations_to_delete {
            self.relations.remove(id);
            pruned_relations += 1;
        }

        // Decay concept energy
        for c in self.concepts.values_mut() {
            c.energy *= 0.95;
            c.last_seen = iso_now();
        }

        // Collect concepts to delete
        let concepts_to_delete: Vec<String> = self.concepts.iter()
            .filter(|(name, c)| {
                c.energy < min_energy ||
                (aggressive && !connected.contains(*name) && !self.relations.is_empty())
            })
            .map(|(name, _)| name.clone())
            .collect();

        for name in &concepts_to_delete {
            self.concepts.remove(name);
            pruned_concepts += 1;
        }

        (pruned_relations, pruned_concepts)
    }

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

    pub fn total_concepts(&self) -> usize {
        self.concepts.len()
    }

    pub fn total_relations(&self) -> usize {
        self.relations.len()
    }
}

fn make_relation_key(source: &str, target: &str) -> String {
    let mut parts = vec![source.to_string(), target.to_string()];
    parts.sort();
    parts.join("|||")
}

fn iso_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    format!("{}", secs)
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn default_brain_path() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "seed-intelligence", "Seed-Intelligence") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).ok();
        data_dir.join("brain.json")
    } else {
        PathBuf::from("brain.json")
    }
}