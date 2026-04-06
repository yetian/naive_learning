// Brain - Knowledge Graph with SQLite Storage
//
// This module defines the core data structures for the knowledge graph:
// - Concept: A node representing a learned concept with energy and metadata
// - Relation: An edge connecting two concepts with a weight
// - Book: A source file that has been learned
// - Brain: The knowledge graph container using SQLite backend

use rusqlite::{Connection, params, Row};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// Data Structures (for JSON migration and in-memory operations)
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
    /// Short description/definition of the concept
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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

/// Book record for tracking learned sources
#[derive(Debug, Clone)]
pub struct Book {
    pub id: i64,
    pub file_hash: String,
    pub file_path: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub file_size: i64,
    pub processed_at: i64,
    pub total_concepts_learned: i64,
}

/// Book metadata for learning
#[derive(Debug, Clone, Default)]
pub struct BookMetadata {
    pub title: String,
    pub author: Option<String>,
    pub format: String,
}

/// Metadata for the brain (JSON legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMeta {
    #[serde(rename = "totalConcepts")]
    pub total_concepts: u32,
    #[serde(rename = "totalRelations")]
    pub total_relations: u32,
    #[serde(rename = "totalLearnCount")]
    pub total_learn_count: u32,
}

/// Legacy JSON brain structure (for migration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyBrain {
    pub version: String,
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<String>,
    pub concepts: HashMap<String, Concept>,
    pub relations: HashMap<String, Relation>,
    pub meta: BrainMeta,
}

// =============================================================================
// Brain Implementation (SQLite Backend)
// =============================================================================

/// Knowledge graph (brain) with SQLite backend
pub struct Brain {
    conn: Connection,
    db_path: PathBuf,
}

impl Brain {
    /// Create or open brain database
    pub fn new(db_path: &PathBuf) -> Result<Self, String> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let mut brain = Self {
            conn,
            db_path: db_path.clone(),
        };

        brain.create_schema()?;
        Ok(brain)
    }

    /// Create database schema
    fn create_schema(&self) -> Result<(), String> {
        self.conn.execute_batch(
            r#"
            -- Books table
            CREATE TABLE IF NOT EXISTS books (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_hash TEXT UNIQUE NOT NULL,
                file_path TEXT,
                title TEXT NOT NULL,
                author TEXT,
                format TEXT NOT NULL,
                file_size INTEGER DEFAULT 0,
                processed_at INTEGER NOT NULL,
                total_concepts_learned INTEGER DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_books_hash ON books(file_hash);
            CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);

            -- Concepts table
            CREATE TABLE IF NOT EXISTS concepts (
                name TEXT PRIMARY KEY,
                energy REAL NOT NULL DEFAULT 0.1,
                count INTEGER NOT NULL DEFAULT 1,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                description TEXT
            );

            -- Relations table
            CREATE TABLE IF NOT EXISTS relations (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                weight REAL NOT NULL DEFAULT 0.0,
                count INTEGER NOT NULL DEFAULT 0,
                last_updated INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_relations_source ON relations(source);
            CREATE INDEX IF NOT EXISTS idx_relations_target ON relations(target);
            CREATE INDEX IF NOT EXISTS idx_relations_weight ON relations(weight);

            -- Book-Concept associations
            CREATE TABLE IF NOT EXISTS book_concepts (
                book_id INTEGER NOT NULL,
                concept_name TEXT NOT NULL,
                mention_count INTEGER DEFAULT 1,
                PRIMARY KEY (book_id, concept_name),
                FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
                FOREIGN KEY (concept_name) REFERENCES concepts(name) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_book_concepts_book ON book_concepts(book_id);
            CREATE INDEX IF NOT EXISTS idx_book_concepts_concept ON book_concepts(concept_name);

            -- Brain metadata
            CREATE TABLE IF NOT EXISTS brain_meta (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            "#
        ).map_err(|e| format!("Failed to create schema: {}", e))?;

        Ok(())
    }

    /// Get database path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Get existing concept or create a new one
    pub fn get_or_create_concept(&mut self, name: &str) -> Concept {
        let now = current_timestamp();

        // Try to get existing
        let existing: Option<Concept> = self.conn.query_row(
            "SELECT energy, count, first_seen, last_seen, description FROM concepts WHERE name = ?1",
            params![name],
            |row| Ok(Concept {
                energy: row.get(0)?,
                count: row.get(1)?,
                first_seen: row.get(2)?,
                last_seen: row.get(3)?,
                description: row.get(4)?,
            })
        ).ok();

        match existing {
            Some(concept) => concept,
            None => {
                // Create new concept
                self.conn.execute(
                    "INSERT INTO concepts (name, energy, count, first_seen, last_seen, description)
                     VALUES (?1, 0.1, 1, ?2, ?2, NULL)",
                    params![name, now]
                ).ok();

                Concept {
                    energy: 0.1,
                    count: 1,
                    first_seen: now.clone(),
                    last_seen: now,
                    description: None,
                }
            }
        }
    }

    /// Update concept after learning
    pub fn update_concept(&mut self, name: &str, energy_delta: f64, count_delta: u32) {
        let now = current_timestamp();
        self.conn.execute(
            "UPDATE concepts SET energy = energy + ?1, count = count + ?2, last_seen = ?3 WHERE name = ?4",
            params![energy_delta, count_delta, now, name]
        ).ok();
    }

    /// Apply batch updates efficiently using a single transaction
    /// Returns (relations_updated, concepts_updated)
    pub fn apply_batch(&mut self, batch: crate::learner::LearningBatch) -> (u32, u32) {
        let tx = self.conn.transaction().expect("Failed to start transaction");
        let now = current_timestamp();
        let now_ms = current_millis();

        let mut relations_count = 0u32;
        let mut concepts_count = 0u32;

        // Batch insert/update concepts using UPSERT
        for (name, (energy_delta, count_delta)) in &batch.concepts {
            tx.execute(
                "INSERT INTO concepts (name, energy, count, first_seen, last_seen, description)
                 VALUES (?1, ?2, ?3, ?4, ?4, NULL)
                 ON CONFLICT(name) DO UPDATE SET
                    energy = energy + excluded.energy,
                    count = count + excluded.count,
                    last_seen = excluded.last_seen",
                params![name, *energy_delta, *count_delta, now]
            ).ok();
            concepts_count += 1;
        }

        // Batch insert/update relations using UPSERT
        for (_, (weight_delta, count_delta, source, target)) in &batch.relations {
            let id = format!("rel_{}_{}", source, target);
            tx.execute(
                "INSERT INTO relations (id, source, target, weight, count, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                    weight = MIN(weight + excluded.weight, 1.0),
                    count = count + excluded.count,
                    last_updated = excluded.last_updated",
                params![id, source, target, *weight_delta, *count_delta, now_ms]
            ).ok();
            relations_count += 1;
        }

        tx.commit().expect("Failed to commit transaction");

        (relations_count, concepts_count)
    }

    /// Set the description for a concept
    pub fn set_concept_description(&mut self, name: &str, description: &str) {
        if description.is_empty() {
            return;
        }

        // Only update if new description is longer
        self.conn.execute(
            "UPDATE concepts SET description = ?1
             WHERE name = ?2 AND (description IS NULL OR LENGTH(?1) > LENGTH(description))",
            params![description, name]
        ).ok();
    }

    /// Get a concept by name
    pub fn get_concept(&self, name: &str) -> Option<Concept> {
        self.conn.query_row(
            "SELECT energy, count, first_seen, last_seen, description FROM concepts WHERE name = ?1",
            params![name],
            |row| Ok(Concept {
                energy: row.get(0)?,
                count: row.get(1)?,
                first_seen: row.get(2)?,
                last_seen: row.get(3)?,
                description: row.get(4)?,
            })
        ).ok()
    }

    /// Get all concepts as a HashMap
    pub fn get_all_concepts(&self) -> HashMap<String, Concept> {
        let mut concepts = HashMap::new();

        let stmt = self.conn.prepare(
            "SELECT name, energy, count, first_seen, last_seen, description FROM concepts"
        );

        if let Ok(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    Concept {
                        energy: row.get(1)?,
                        count: row.get(2)?,
                        first_seen: row.get(3)?,
                        last_seen: row.get(4)?,
                        description: row.get(5)?,
                    }
                ))
            });

            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    concepts.insert(row.0, row.1);
                }
            }
        }

        concepts
    }

    /// Add a new relation if it doesn't exist, returns relation id
    pub fn add_or_update_relation(&mut self, source: &str, target: &str) -> String {
        let key = make_relation_key(source, target);
        let id = format!("rel_{}", key.replace("|||", "_"));

        // Check if exists
        let exists: bool = self.conn.query_row(
            "SELECT 1 FROM relations WHERE id = ?1",
            params![&id],
            |_| Ok(true)
        ).unwrap_or(false);

        if !exists {
            self.conn.execute(
                "INSERT INTO relations (id, source, target, weight, count, last_updated)
                 VALUES (?1, ?2, ?3, 0.0, 0, ?4)",
                params![&id, source, target, current_millis()]
            ).ok();
        }

        id
    }

    /// Get a relation by source and target
    pub fn get_relation_mut(&mut self, source: &str, target: &str) -> Option<Relation> {
        let key = make_relation_key(source, target);
        let id = format!("rel_{}", key.replace("|||", "_"));

        self.conn.query_row(
            "SELECT id, source, target, weight, count, last_updated FROM relations WHERE id = ?1",
            params![&id],
            |row| Ok(Relation {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                weight: row.get(3)?,
                count: row.get(4)?,
                last_updated: row.get(5)?,
            })
        ).ok()
    }

    /// Update relation weight and count
    pub fn update_relation(&mut self, id: &str, weight_delta: f64, count_delta: u32) {
        self.conn.execute(
            "UPDATE relations SET weight = MIN(weight + ?1, 1.0), count = count + ?2, last_updated = ?3 WHERE id = ?4",
            params![weight_delta, count_delta, current_millis(), id]
        ).ok();
    }

    /// Get all relations for a concept (indexed query)
    pub fn get_relations_for_concept(&self, concept: &str) -> Vec<Relation> {
        let mut relations = Vec::new();

        let stmt = self.conn.prepare(
            "SELECT id, source, target, weight, count, last_updated FROM relations
             WHERE source = ?1 OR target = ?1"
        );

        if let Ok(mut stmt) = stmt {
            let rows = stmt.query_map(params![concept], |row| {
                Ok(Relation {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    weight: row.get(3)?,
                    count: row.get(4)?,
                    last_updated: row.get(5)?,
                })
            });

            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    relations.push(row);
                }
            }
        }

        relations
    }

    /// Get all relations as a HashMap
    pub fn get_all_relations(&self) -> HashMap<String, Relation> {
        let mut relations = HashMap::new();

        let stmt = self.conn.prepare(
            "SELECT id, source, target, weight, count, last_updated FROM relations"
        );

        if let Ok(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    Relation {
                        id: row.get(0)?,
                        source: row.get(1)?,
                        target: row.get(2)?,
                        weight: row.get(3)?,
                        count: row.get(4)?,
                        last_updated: row.get(5)?,
                    }
                ))
            });

            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    relations.insert(row.0, row.1);
                }
            }
        }

        relations
    }

    /// Cleanup weak relations and low-energy concepts
    pub fn cleanup(&mut self, min_weight: f64, min_energy: f64, aggressive: bool) -> (u32, u32) {
        let now_ts = current_timestamp();
        let now_ms = current_millis();

        // Apply decay to relations
        self.conn.execute("UPDATE relations SET weight = weight * 0.95, last_updated = ?1", params![now_ms]).ok();

        // Delete weak relations
        let pruned_relations = self.conn.execute(
            "DELETE FROM relations WHERE weight < ?1",
            params![min_weight]
        ).unwrap_or(0) as u32;

        // Apply decay to concepts
        self.conn.execute("UPDATE concepts SET energy = energy * 0.95, last_seen = ?1", params![now_ts]).ok();

        // Get connected concepts after relation cleanup
        let connected: HashSet<String> = {
            let mut set = HashSet::new();
            if let Ok(mut stmt) = self.conn.prepare("SELECT DISTINCT source, target FROM relations") {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) {
                    for row in rows.flatten() {
                        set.insert(row.0);
                        set.insert(row.1);
                    }
                }
            }
            set
        };

        // Delete low-energy concepts (and disconnected if aggressive)
        let pruned_concepts = if aggressive && !connected.is_empty() {
            self.conn.execute(
                "DELETE FROM concepts WHERE energy < ?1 OR (name NOT IN (SELECT source FROM relations UNION SELECT target FROM relations) AND (SELECT COUNT(*) FROM relations) > 0)",
                params![min_energy]
            ).unwrap_or(0) as u32
        } else {
            self.conn.execute(
                "DELETE FROM concepts WHERE energy < ?1",
                params![min_energy]
            ).unwrap_or(0) as u32
        };

        (pruned_relations, pruned_concepts)
    }

    /// Clear all concepts and relations
    pub fn clear(&mut self) {
        self.conn.execute("DELETE FROM book_concepts", []).ok();
        self.conn.execute("DELETE FROM relations", []).ok();
        self.conn.execute("DELETE FROM concepts", []).ok();
        self.conn.execute("DELETE FROM books", []).ok();
        self.conn.execute("DELETE FROM brain_meta", []).ok();
    }

    /// Get total number of concepts
    pub fn total_concepts(&self) -> usize {
        self.conn.query_row("SELECT COUNT(*) FROM concepts", [], |row| row.get::<_, i64>(0))
            .unwrap_or(0) as usize
    }

    /// Get total number of relations
    pub fn total_relations(&self) -> usize {
        self.conn.query_row("SELECT COUNT(*) FROM relations", [], |row| row.get::<_, i64>(0))
            .unwrap_or(0) as usize
    }

    // =========================================================================
    // Book Management
    // =========================================================================

    /// Check if a book has been learned by hash
    pub fn has_book(&self, file_hash: &str) -> Option<Book> {
        self.conn.query_row(
            "SELECT id, file_hash, file_path, title, author, format, file_size, processed_at, total_concepts_learned
             FROM books WHERE file_hash = ?1",
            params![file_hash],
            |row| Ok(Book {
                id: row.get(0)?,
                file_hash: row.get(1)?,
                file_path: row.get(2)?,
                title: row.get(3)?,
                author: row.get(4)?,
                format: row.get(5)?,
                file_size: row.get(6)?,
                processed_at: row.get(7)?,
                total_concepts_learned: row.get(8)?,
            })
        ).ok()
    }

    /// Add a new book record
    pub fn add_book(&mut self, file_hash: &str, file_path: &str, metadata: &BookMetadata, file_size: i64) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn.execute(
            "INSERT INTO books (file_hash, file_path, title, author, format, file_size, processed_at, total_concepts_learned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
            params![file_hash, file_path, metadata.title, metadata.author, metadata.format, file_size, now]
        ).ok();

        self.conn.last_insert_rowid()
    }

    /// Update book's concept count
    pub fn update_book_concept_count(&mut self, book_id: i64, count: i64) {
        self.conn.execute(
            "UPDATE books SET total_concepts_learned = ?1 WHERE id = ?2",
            params![count, book_id]
        ).ok();
    }

    /// Get all books
    pub fn get_all_books(&self) -> Vec<Book> {
        let mut books = Vec::new();

        let stmt = self.conn.prepare(
            "SELECT id, file_hash, file_path, title, author, format, file_size, processed_at, total_concepts_learned
             FROM books ORDER BY processed_at DESC"
        );

        if let Ok(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                Ok(Book {
                    id: row.get(0)?,
                    file_hash: row.get(1)?,
                    file_path: row.get(2)?,
                    title: row.get(3)?,
                    author: row.get(4)?,
                    format: row.get(5)?,
                    file_size: row.get(6)?,
                    processed_at: row.get(7)?,
                    total_concepts_learned: row.get(8)?,
                })
            });

            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    books.push(row);
                }
            }
        }

        books
    }

    /// Get book by ID or title
    pub fn get_book(&self, id_or_title: &str) -> Option<Book> {
        // Try as ID first
        if let Ok(id) = id_or_title.parse::<i64>() {
            return self.conn.query_row(
                "SELECT id, file_hash, file_path, title, author, format, file_size, processed_at, total_concepts_learned
                 FROM books WHERE id = ?1",
                params![id],
                |row| Ok(Book {
                    id: row.get(0)?,
                    file_hash: row.get(1)?,
                    file_path: row.get(2)?,
                    title: row.get(3)?,
                    author: row.get(4)?,
                    format: row.get(5)?,
                    file_size: row.get(6)?,
                    processed_at: row.get(7)?,
                    total_concepts_learned: row.get(8)?,
                })
            ).ok();
        }

        // Try as title (partial match)
        self.conn.query_row(
            "SELECT id, file_hash, file_path, title, author, format, file_size, processed_at, total_concepts_learned
             FROM books WHERE title LIKE ?1",
            params![format!("%{}%", id_or_title)],
            |row| Ok(Book {
                id: row.get(0)?,
                file_hash: row.get(1)?,
                file_path: row.get(2)?,
                title: row.get(3)?,
                author: row.get(4)?,
                format: row.get(5)?,
                file_size: row.get(6)?,
                processed_at: row.get(7)?,
                total_concepts_learned: row.get(8)?,
            })
        ).ok()
    }

    /// Remove a book (keeps concepts)
    pub fn remove_book(&mut self, book_id: i64) -> bool {
        let removed = self.conn.execute(
            "DELETE FROM books WHERE id = ?1",
            params![book_id]
        ).unwrap_or(0);

        removed > 0
    }

    /// Track concept from book
    pub fn track_book_concept(&mut self, book_id: i64, concept_name: &str) {
        self.conn.execute(
            "INSERT OR IGNORE INTO book_concepts (book_id, concept_name, mention_count) VALUES (?1, ?2, 1)",
            params![book_id, concept_name]
        ).ok();

        self.conn.execute(
            "UPDATE book_concepts SET mention_count = mention_count + 1 WHERE book_id = ?1 AND concept_name = ?2",
            params![book_id, concept_name]
        ).ok();
    }

    /// Get concepts learned from a book
    pub fn get_book_concepts(&self, book_id: i64) -> Vec<(String, i32)> {
        let mut concepts = Vec::new();

        let stmt = self.conn.prepare(
            "SELECT concept_name, mention_count FROM book_concepts WHERE book_id = ?1 ORDER BY mention_count DESC"
        );

        if let Ok(mut stmt) = stmt {
            let rows = stmt.query_map(params![book_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            });

            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    concepts.push(row);
                }
            }
        }

        concepts
    }

    // =========================================================================
    // Migration
    // =========================================================================

    /// Migrate from legacy JSON format
    pub fn migrate_from_json(&mut self, json_path: &PathBuf) -> Result<(), String> {
        let content = fs::read_to_string(json_path)
            .map_err(|e| format!("Failed to read JSON: {}", e))?;

        let legacy: LegacyBrain = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let tx = self.conn.transaction()
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        // Migrate concepts
        for (name, concept) in &legacy.concepts {
            tx.execute(
                "INSERT OR IGNORE INTO concepts (name, energy, count, first_seen, last_seen, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![name, concept.energy, concept.count, &concept.first_seen,
                        &concept.last_seen, &concept.description]
            ).map_err(|e| format!("Failed to insert concept: {}", e))?;
        }

        // Migrate relations
        for (_, relation) in &legacy.relations {
            tx.execute(
                "INSERT OR IGNORE INTO relations (id, source, target, weight, count, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![&relation.id, &relation.source, &relation.target,
                        relation.weight, relation.count, relation.last_updated]
            ).map_err(|e| format!("Failed to insert relation: {}", e))?;
        }

        // Set metadata
        tx.execute(
            "INSERT OR REPLACE INTO brain_meta (key, value) VALUES ('version', ?1)",
            params![&legacy.version]
        ).map_err(|e| format!("Failed to set metadata: {}", e))?;

        tx.commit().map_err(|e| format!("Failed to commit transaction: {}", e))?;

        // Backup the old JSON file
        let backup_path = json_path.with_extension("json.backup");
        fs::rename(json_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;

        Ok(())
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
pub fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_default()
}

/// Get current time as milliseconds since Unix epoch
pub fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Get the default brain database path
pub fn default_brain_path() -> PathBuf {
    directories::ProjectDirs::from("com", "seed-intelligence", "Seed-Intelligence")
        .map(|proj_dirs| {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            data_dir.join("brain.db")
        })
        .unwrap_or_else(|| PathBuf::from("brain.db"))
}

/// Get the legacy JSON brain path (for migration check)
pub fn legacy_brain_path() -> PathBuf {
    directories::ProjectDirs::from("com", "seed-intelligence", "Seed-Intelligence")
        .map(|proj_dirs| {
            let data_dir = proj_dirs.data_dir();
            data_dir.join("brain.json")
        })
        .unwrap_or_else(|| PathBuf::from("brain.json"))
}

/// Initialize brain with automatic migration from JSON if needed
pub fn init_brain() -> Result<Brain, String> {
    let db_path = default_brain_path();
    let json_path = legacy_brain_path();

    // Check for legacy JSON to migrate BEFORE creating database
    let should_migrate = json_path.exists() && !db_path.exists();

    let mut brain = Brain::new(&db_path)?;

    if should_migrate {
        println!("Migrating brain.json to SQLite...");
        brain.migrate_from_json(&json_path)?;
        println!("Migration complete! Backup saved as brain.json.backup");
    }

    Ok(brain)
}
