// Seed-Intelligence CLI - Main Entry Point
// Hebbian learning based embodied intelligence system

mod brain;
mod crawler;
mod inference;
mod learner;
mod nlp;
mod lm;
mod file_reader;
mod response_generator;
mod observer;

use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;

/// Seed-Intelligence CLI - Hebbian Learning AI
#[derive(Parser)]
#[command(name = "seed")]
#[command(about = "Seed-Intelligence: Hebbian learning based AI", long_about = None)]
struct Cli {
    /// Brain data path
    #[arg(short, long)]
    brain: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a concept and start learning
    Init {
        concept: String,
        /// Auto-learn related concepts
        #[arg(short, long, default_value = "true")]
        auto_learn: bool,
    },
    /// Learn a concept from web search
    Learn {
        concept: String,
    },
    /// Learn from text directly
    LearnText {
        text: String,
        /// Focus concept (ontology anchoring)
        focus: Option<String>,
    },
    /// Query the knowledge base (Q&A)
    Query {
        question: String,
    },
    /// Ask with enhanced features
    Ask {
        question: String,
    },
    /// Show knowledge graph stats
    Stats {
        /// Show per-book statistics
        #[arg(short, long)]
        by_book: bool,
    },
    /// Show full brain (knowledge graph)
    Brain,
    /// Clear knowledge base
    Clear,
    /// Get concept details
    Concept {
        name: String,
    },
    /// Get related concepts
    Related {
        name: String,
        /// Max depth for traversal
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },
    /// Generate text with local LM
    Generate {
        prompt: String,
        /// Max tokens to generate
        #[arg(short, long, default_value = "50")]
        max_tokens: usize,
    },
    /// Train local language model
    Train {
        text: String,
        /// Number of epochs
        #[arg(short, long, default_value = "3")]
        epochs: u32,
    },
    /// Learn from file (txt, epub, mobi, azw3, pdf)
    LearnFile {
        /// File path (txt, epub, mobi, azw3, pdf)
        file: PathBuf,
        /// Focus concept (ontology anchoring)
        focus: Option<String>,
        /// Lines per second (rate control)
        #[arg(short, long, default_value = "100")]
        rate: u32,
        /// Force re-learning even if already processed
        #[arg(short, long)]
        force: bool,
    },
    /// Train local language model from file
    TrainFile {
        /// File path (txt, epub, mobi, azw3)
        file: PathBuf,
        /// Number of epochs
        #[arg(short, long, default_value = "3")]
        epochs: u32,
    },
    /// Interactive REPL mode
    Repl,
    /// Observe and learn from environment (embodied intelligence mode)
    Observe {
        /// Sandbox directory to watch (default: ./agent_sandbox)
        #[arg(short, long)]
        sandbox: Option<PathBuf>,
    },
    /// List all learned books
    ListBooks {
        /// Filter by title (partial match)
        #[arg(short, long)]
        title: Option<String>,
    },
    /// Show what was learned from a specific book
    BookInfo {
        /// Book ID or title
        book: String,
    },
    /// Check if a file has been learned
    CheckFile {
        /// File path to check
        file: PathBuf,
    },
    /// Remove a book record (keeps concepts)
    RemoveBook {
        /// Book ID or title
        book: String,
    },
}

fn get_brain_path(brain_arg: Option<PathBuf>) -> PathBuf {
    if let Some(p) = brain_arg {
        return p;
    }
    crate::brain::default_brain_path()
}

async fn run_init(learner: &mut learner::IncrementalLearner, concept: &str, auto_learn: bool) {
    println!("🚀 Learning concept: {}", concept);

    // Search and learn
    let search_results = crawler::search(concept).await;
    println!("📊 Got {} search results", search_results.len());

    let texts: Vec<String> = search_results.iter()
        .map(|r| format!("{}: {}", r.title, r.snippet))
        .collect();
    let full_text = texts.join(" ");

    // Learn with focus
    let result = learner.learn_from_text(&full_text, Some(concept));
    println!("📚 Learning result: {:?}", result);

    // Store description from first search result (after concept is created)
    if let Some(first_result) = search_results.first() {
        learner.set_concept_description(concept, &first_result.snippet);
    }

    // Cleanup
    let cleanup = learner.cleanup(false);
    println!("🧹 Cleanup: {} relations, {} concepts pruned",
        cleanup.pruned_relations, cleanup.pruned_concepts);

    // Get stats
    let stats = learner.get_stats();
    println!("\n📈 Stats: {} concepts, {} relations",
        stats.total_concepts, stats.total_relations);

    if auto_learn && result.concepts_updated > 0 {
        println!("\n✨ Auto-learning enabled - learning related concepts...");
        // Note: For simplicity, we just show the top concepts
        if !stats.top_concepts.is_empty() {
            println!("Top concepts discovered:");
            for tc in stats.top_concepts.iter().take(5) {
                if tc.name != concept {
                    println!("  - {} (energy: {:.2})", tc.name, tc.energy);
                }
            }
        }
    }
}

async fn run_learn(learner: &mut learner::IncrementalLearner, concept: &str) {
    run_init(learner, concept, false).await;
}

fn run_learn_text(learner: &mut learner::IncrementalLearner, text: &str, focus: Option<&str>) {
    let result = learner.learn_from_text(text, focus);
    println!("📚 Learning result: {:?}", result);

    let cleanup = learner.cleanup(false);
    println!("🧹 Cleanup: {} relations, {} concepts pruned",
        cleanup.pruned_relations, cleanup.pruned_concepts);
}

fn run_query(learner: &learner::IncrementalLearner, question: &str) {
    let answer = inference::query(question, &learner.brain);
    println!("\n🤖 {}\n", answer.answer);
    if answer.confidence > 0 {
        println!("置信度: {}%", answer.confidence);
    }
}

fn run_ask(learner: &learner::IncrementalLearner, question: &str) {
    let answer = inference::ask(question, &learner.brain);
    println!("\n🤖 {}\n", answer.answer);
    if answer.confidence > 0 {
        println!("置信度: {}%", answer.confidence);
    }
}

fn run_stats(learner: &learner::IncrementalLearner, by_book: bool) {
    let stats = learner.get_stats();
    println!("\n📊 Knowledge Graph Stats");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("总概念数: {}", stats.total_concepts);
    println!("总关系数: {}", stats.total_relations);
    println!("平均权重: {:.4}", stats.avg_weight);
    println!("平均能量: {:.4}", stats.avg_energy);

    if by_book {
        let books = learner.brain.get_all_books();
        if !books.is_empty() {
            println!("\n📚 Books Learned ({}):", books.len());
            for book in books.iter().take(20) {
                println!("  {} - {} concepts", book.title, book.total_concepts_learned);
            }
        }
    }

    println!("\n🔝 Top Concepts:");
    for tc in stats.top_concepts.iter().take(10) {
        println!("  {} (energy: {:.2}, count: {})", tc.name, tc.energy, tc.count);
    }
}

fn run_brain(learner: &learner::IncrementalLearner) {
    println!("\n🧠 Knowledge Graph (SQLite)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("概念数: {}", learner.brain.total_concepts());
    println!("关系数: {}", learner.brain.total_relations());

    let concepts = learner.brain.get_all_concepts();
    if !concepts.is_empty() {
        println!("\n📝 Concepts (first 20):");
        for (i, (name, c)) in concepts.iter().take(20).enumerate() {
            println!("  {}. {} (energy: {:.2}, count: {})", i + 1, name, c.energy, c.count);
        }
    }

    let relations = learner.brain.get_all_relations();
    if !relations.is_empty() {
        println!("\n🔗 Relations (first 20):");
        for (i, r) in relations.values().take(20).enumerate() {
            println!("  {}. {} ↔ {} (weight: {:.2})", i + 1, r.source, r.target, r.weight);
        }
    }
}

fn run_clear(learner: &mut learner::IncrementalLearner) {
    learner.clear();
    println!("✅ Knowledge base cleared!");
}

fn run_concept(learner: &learner::IncrementalLearner, name: &str) {
    if let Some(c) = learner.get_concept(name) {
        println!("\n📌 Concept: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        if let Some(ref desc) = c.description {
            println!("📝 {}", desc);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        println!("能量: {:.4}", c.energy);
        println!("出现次数: {}", c.count);
        println!("首次出现: {}", c.first_seen);
        println!("最后出现: {}", c.last_seen);
    } else {
        println!("❌ Concept '{}' not found", name);
    }
}

fn run_related(learner: &learner::IncrementalLearner, name: &str, depth: usize) {
    let related = learner.get_related_concepts(name, depth);
    if related.is_empty() {
        println!("❌ No related concepts found for '{}'", name);
    } else {
        println!("\n🔗 Related concepts to '{}':", name);
        for (i, (target, weight)) in related.iter().take(10).enumerate() {
            println!("  {}. {} (关联度: {:.0}%)", i + 1, target, weight * 100.0);
        }
    }
}

fn run_generate(learner: &learner::IncrementalLearner, prompt: &str, max_tokens: usize) {
    // Check if we have a trained model
    let model_path = learner.brain_path.parent()
        .map(|p| p.join("lm_weights.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("lm_weights.json"));

    // Create model
    let mut model = match lm::create_model() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ Failed to create language model: {}", e);
            return;
        }
    };

    // Try to load existing weights
    if model_path.exists() {
        if let Err(e) = model.load_weights(model_path.to_str().unwrap()) {
            println!("⚠️ Could not load model weights: {}", e);
        }
    }

    // Build vocabulary from brain concepts
    let concepts = learner.brain.get_all_concepts();
    for concept_name in concepts.keys() {
        model.add_vocab(concept_name);
    }
    model.add_vocab(prompt);

    println!("\n🤖 Generating text...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Prompt: {}", prompt);

    let output = model.generate(prompt, max_tokens, 0.8);
    println!("\nGenerated: {}", output);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn run_train(learner: &learner::IncrementalLearner, text: &str, epochs: u32) {
    let model_path = learner.brain_path.parent()
        .map(|p| p.join("lm_weights.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("lm_weights.json"));

    println!("\n🧠 Training Language Model");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Text length: {} chars", text.len());
    println!("Epochs: {}", epochs);

    // Create model
    let model = match lm::create_model() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ Failed to create language model: {}", e);
            return;
        }
    };

    // Create trainer
    let mut trainer = lm::Trainer::new(model, 0.01);

    // Train
    trainer.train_on_text(text, epochs);

    // Save weights
    if let Err(e) = trainer.model.save_weights(model_path.to_str().unwrap()) {
        eprintln!("❌ Failed to save model weights: {}", e);
    } else {
        println!("✅ Model weights saved to {:?}", model_path);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn run_learn_file(learner: &mut learner::IncrementalLearner, file_path: &PathBuf, focus: Option<&str>, rate: u32, force: bool) {
    println!("\n📚 Learning from file: {:?}", file_path);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Get file metadata
    let metadata = file_reader::extract_book_metadata(file_path);
    let file_size = file_reader::get_file_size(file_path).unwrap_or(0);

    println!("📖 Title: {}", metadata.title);
    if let Some(ref author) = metadata.author {
        println!("👤 Author: {}", author);
    }
    println!("📄 Format: {}", metadata.format);
    println!("📊 Size: {} bytes", file_size);

    // Compute hash while streaming
    println!("\n🔍 Computing file hash...");

    let batch_size = 10;
    let delay_ms = 1000 / rate.max(1);

    // First pass: compute hash and check if already learned
    let result = file_reader::stream_read_file_with_hash(file_path, batch_size, |_| {});

    match result {
        Ok((_, hash, _)) => {
            println!("🔑 File hash: {}...", &hash[..16]);

            // Check if already learned
            if let Some(existing_book) = learner.brain.has_book(&hash) {
                if !force {
                    println!("\n⚠️  This file has already been learned!");
                    println!("   Book: {} (ID: {})", existing_book.title, existing_book.id);
                    println!("   Learned on: {}", format_timestamp(existing_book.processed_at));
                    println!("   Concepts learned: {}", existing_book.total_concepts_learned);
                    println!("\n   Use --force to re-learn this file.");
                    return;
                }
                println!("\n⚠️  Re-learning previously learned file (--force)");
            }

            // Create book record
            let book_id = learner.brain.add_book(
                &hash,
                file_path.to_str().unwrap_or(""),
                &metadata,
                file_size
            );

            println!("📝 Book ID: {}", book_id);

            // Start book context for learning
            learner.start_book(book_id);

            // Second pass: actual learning
            let mut concepts_count = 0i64;
            let learn_result = file_reader::stream_read_file(file_path, batch_size, |text| {
                let result = learner.learn_from_text(text, focus);
                concepts_count += result.concepts_updated as i64;

                // Rate control
                std::thread::sleep(std::time::Duration::from_millis(delay_ms as u64));
            });

            // End book context
            learner.end_book();

            match learn_result {
                Ok((lines, temp_file)) => {
                    // Cleanup temp file
                    file_reader::cleanup_temp_file(temp_file.as_ref());

                    // Update book concept count
                    learner.brain.update_book_concept_count(book_id, concepts_count);

                    // Final cleanup
                    let cleanup = learner.cleanup(false);

                    println!("\n✅ Learning complete!");
                    println!("   Lines processed: {}", lines);
                    println!("   Concepts learned: {}", concepts_count);
                    println!("   Pruned: {} relations, {} concepts", cleanup.pruned_relations, cleanup.pruned_concepts);

                    let stats = learner.get_stats();
                    println!("   Total concepts: {}", stats.total_concepts);
                    println!("   Total relations: {}", stats.total_relations);
                }
                Err(e) => {
                    eprintln!("❌ Error reading file: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error processing file: {}", e);
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn run_train_file(learner: &learner::IncrementalLearner, file_path: &PathBuf, epochs: u32) {
    let model_path = learner.brain_path.parent()
        .map(|p| p.join("lm_weights.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("lm_weights.json"));

    println!("\n🧠 Training Language Model from file");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {:?}", file_path);
    println!("Epochs: {}", epochs);

    // Read file content
    let result = file_reader::read_file(file_path);

    match result {
        Ok((text, temp_file)) => {
            // Cleanup temp file
            file_reader::cleanup_temp_file(temp_file.as_ref());

            println!("Text length: {} chars", text.len());

            // Create model
            let model = match lm::create_model() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("❌ Failed to create language model: {}", e);
                    return;
                }
            };

            // Create trainer
            let mut trainer = lm::Trainer::new(model, 0.01);

            // Train
            trainer.train_on_text(&text, epochs);

            // Save weights
            if let Err(e) = trainer.model.save_weights(model_path.to_str().unwrap()) {
                eprintln!("❌ Failed to save model weights: {}", e);
            } else {
                println!("✅ Model weights saved to {:?}", model_path);
            }
        }
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn run_observe(learner: &mut learner::IncrementalLearner, sandbox: Option<PathBuf>) {
    let sandbox_path = sandbox.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.join("agent_sandbox"))
            .unwrap_or_else(|_| PathBuf::from("./agent_sandbox"))
    });

    // Create sandbox directory if it doesn't exist
    if !sandbox_path.exists() {
        std::fs::create_dir_all(&sandbox_path).ok();
    }

    if let Err(e) = observer::run_observe_mode(learner, &sandbox_path) {
        eprintln!("❌ Observation mode error: {}", e);
    }
}

fn run_list_books(learner: &learner::IncrementalLearner, title_filter: Option<&str>) {
    let books = learner.brain.get_all_books();

    let filtered: Vec<_> = if let Some(title) = title_filter {
        books.into_iter().filter(|b| b.title.to_lowercase().contains(&title.to_lowercase())).collect()
    } else {
        books
    };

    if filtered.is_empty() {
        println!("📚 No books found");
        return;
    }

    println!("\n📚 Learned Books ({})", filtered.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:<4} {:<30} {:<20} {:<8} {:<10}", "ID", "Title", "Author", "Format", "Concepts");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for book in filtered {
        let author = book.author.as_deref().unwrap_or("-");
        let title = if book.title.len() > 28 { format!("{}...", &book.title[..25]) } else { book.title.clone() };
        let author_display = if author.len() > 18 { format!("{}...", &author[..15]) } else { author.to_string() };

        println!("{:<4} {:<30} {:<20} {:<8} {:<10}",
            book.id, title, author_display, book.format, book.total_concepts_learned);
    }
}

fn run_book_info(learner: &learner::IncrementalLearner, id_or_title: &str) {
    if let Some(book) = learner.brain.get_book(id_or_title) {
        println!("\n📖 Book: {}", book.title);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("ID: {}", book.id);
        if let Some(ref author) = book.author {
            println!("Author: {}", author);
        }
        println!("Format: {}", book.format);
        println!("File size: {} bytes", book.file_size);
        println!("Processed: {}", format_timestamp(book.processed_at));
        println!("Concepts learned: {}", book.total_concepts_learned);

        // Show concepts from this book
        let concepts = learner.brain.get_book_concepts(book.id);
        if !concepts.is_empty() {
            println!("\n📝 Top concepts from this book:");
            for (i, (name, count)) in concepts.iter().take(20).enumerate() {
                println!("  {}. {} (mentions: {})", i + 1, name, count);
            }
        }
    } else {
        println!("❌ Book '{}' not found", id_or_title);
    }
}

fn run_check_file(learner: &learner::IncrementalLearner, file_path: &PathBuf) {
    println!("\n🔍 Checking file: {:?}", file_path);

    // Get file metadata
    let metadata = file_reader::extract_book_metadata(file_path);
    println!("📖 Title: {}", metadata.title);

    // Compute hash
    match file_reader::compute_file_hash(file_path) {
        Ok(hash) => {
            println!("🔑 Hash: {}...", &hash[..16]);

            if let Some(book) = learner.brain.has_book(&hash) {
                println!("\n✅ Already learned");
                println!("   Book: {} (ID: {})", book.title, book.id);
                println!("   Learned on: {}", format_timestamp(book.processed_at));
            } else {
                println!("\n❌ Not yet learned");
            }
        }
        Err(e) => {
            eprintln!("❌ Error computing hash: {}", e);
        }
    }
}

fn run_remove_book(learner: &mut learner::IncrementalLearner, id_or_title: &str) {
    if let Some(book) = learner.brain.get_book(id_or_title) {
        let book_id = book.id;
        let title = book.title.clone();

        if learner.brain.remove_book(book_id) {
            println!("✅ Removed book: {} (ID: {})", title, book_id);
            println!("   Note: Concepts learned from this book are preserved.");
        } else {
            println!("❌ Failed to remove book");
        }
    } else {
        println!("❌ Book '{}' not found", id_or_title);
    }
}

fn format_timestamp(ts: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dt = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(ts as u64);
    let datetime: chrono::DateTime<chrono::Local> = dt.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn run_repl(learner: &mut learner::IncrementalLearner, brain_path: PathBuf) {
    println!("\n╔═══════════════════════════════════════════╗");
    println!("║     🌱 Seed-Intelligence REPL             ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  直接输入问题进行问答                     ║");
    println!("║  /help 查看所有命令                       ║");
    println!("║  /exit 或 Ctrl+C 退出                     ║");
    println!("╚═══════════════════════════════════════════╝\n");

    // Get a handle to the current runtime
    let rt = tokio::runtime::Handle::current();

    loop {
        print!("> ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "/exit" || input == "/quit" || input == "exit" || input == "quit" {
            println!("👋 再见!");
            break;
        }

        if input.starts_with('/') {
            handle_command(input, learner, &brain_path, &rt);
        } else {
            // Treat as question
            let answer = inference::ask(input, &learner.brain);
            println!("\n🤖 {}\n", answer.answer);
            if answer.confidence > 0 {
                println!("置信度: {}%\n", answer.confidence);
            }
        }
    }
}

fn handle_command(input: &str, learner: &mut learner::IncrementalLearner, brain_path: &PathBuf, rt: &tokio::runtime::Handle) {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).map(|s| *s).unwrap_or("");

    match cmd.as_str() {
        "/help" | "/h" | "help" => {
            print_help();
        }
        "/stats" => {
            run_stats(learner, false);
        }
        "/brain" => {
            run_brain(learner);
        }
        "/clear" => {
            run_clear(learner);
        }
        "/concept" | "/c" => {
            if args.is_empty() {
                println!("用法: /concept <概念名>");
            } else {
                run_concept(learner, args);
            }
        }
        "/related" | "/r" => {
            if args.is_empty() {
                println!("用法: /related <概念名> [深度]");
            } else {
                let parts: Vec<&str> = args.splitn(2, ' ').collect();
                let depth = parts.get(1).and_then(|d| d.parse().ok()).unwrap_or(2);
                run_related(learner, parts[0], depth);
            }
        }
        "/learn" => {
            if args.is_empty() {
                println!("用法: /learn <概念>");
            } else {
                // For now, just learn from the concept name directly
                run_learn_text(learner, args, Some(args));
            }
        }
        "/learn-text" | "/lt" => {
            if args.is_empty() {
                println!("用法: /learn-text <文本>");
            } else {
                run_learn_text(learner, args, None);
            }
        }
        "/init" => {
            if args.is_empty() {
                println!("用法: /init <概念>");
            } else {
                // Use block_in_place to allow blocking within async context
                let concept = args.to_string();
                tokio::task::block_in_place(|| {
                    rt.block_on(run_init(learner, &concept, true));
                });
            }
        }
        "/books" => {
            run_list_books(learner, if args.is_empty() { None } else { Some(args) });
        }
        "/book-info" | "/book" => {
            if args.is_empty() {
                println!("用法: /book-info <书名或ID>");
            } else {
                run_book_info(learner, args);
            }
        }
        "/exit" | "/quit" => {
            println!("👋 再见!");
            std::process::exit(0);
        }
        _ => {
            println!("未知命令: {} - 输入 /help 查看所有命令", cmd);
        }
    }
}

fn print_help() {
    println!("
╔═══════════════════════════════════════════╗
║              🌱 Seed-Intelligence          ║
║              命令帮助                      ║
╠═══════════════════════════════════════════╣
║  问答命令:                                 ║
║    直接输入问题 → 进行问答                 ║
║                                             ║
║  管理命令:                                 ║
║    /help, /h        显示帮助               ║
║    /stats           查看统计信息           ║
║    /brain           查看完整知识图谱       ║
║    /clear           清空知识库             ║
║                                             ║
║  学习命令:                                 ║
║    /init <概念>     初始化并学习概念       ║
║    /learn <概念>    学习概念               ║
║    /learn-text <文本> 从文本学习           ║
║                                             ║
║  查询命令:                                 ║
║    /concept <名称>  查看概念详情           ║
║    /related <名称>  查看相关概念           ║
║                                             ║
║  书籍命令:                                 ║
║    /books [标题]    列出学习的书籍         ║
║    /book-info <ID>  查看书籍详情           ║
║                                             ║
║  退出:                                     ║
║    /exit, /quit     退出程序               ║
╚═══════════════════════════════════════════╝
");
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let brain_path = get_brain_path(cli.brain);

    println!("📂 Brain path: {:?}", brain_path);

    // Use init() for auto-migration support
    let mut learner = learner::IncrementalLearner::init();

    match cli.command {
        Some(Commands::Init { concept, auto_learn }) => {
            run_init(&mut learner, &concept, auto_learn).await;
        }
        Some(Commands::Learn { concept }) => {
            run_learn(&mut learner, &concept).await;
        }
        Some(Commands::LearnText { text, focus }) => {
            run_learn_text(&mut learner, &text, focus.as_deref());
        }
        Some(Commands::Query { question }) => {
            run_query(&learner, &question);
        }
        Some(Commands::Ask { question }) => {
            run_ask(&learner, &question);
        }
        Some(Commands::Stats { by_book }) => {
            run_stats(&learner, by_book);
        }
        Some(Commands::Brain) => {
            run_brain(&learner);
        }
        Some(Commands::Clear) => {
            run_clear(&mut learner);
        }
        Some(Commands::Concept { name }) => {
            run_concept(&learner, &name);
        }
        Some(Commands::Related { name, depth }) => {
            run_related(&learner, &name, depth);
        }
        Some(Commands::Generate { prompt, max_tokens }) => {
            run_generate(&learner, &prompt, max_tokens);
        }
        Some(Commands::Train { text, epochs }) => {
            run_train(&learner, &text, epochs);
        }
        Some(Commands::LearnFile { file, focus, rate, force }) => {
            run_learn_file(&mut learner, &file, focus.as_deref(), rate, force);
        }
        Some(Commands::TrainFile { file, epochs }) => {
            run_train_file(&learner, &file, epochs);
        }
        Some(Commands::Repl) => {
            run_repl(&mut learner, brain_path);
        }
        Some(Commands::Observe { sandbox }) => {
            run_observe(&mut learner, sandbox);
        }
        Some(Commands::ListBooks { title }) => {
            run_list_books(&learner, title.as_deref());
        }
        Some(Commands::BookInfo { book }) => {
            run_book_info(&learner, &book);
        }
        Some(Commands::CheckFile { file }) => {
            run_check_file(&learner, &file);
        }
        Some(Commands::RemoveBook { book }) => {
            run_remove_book(&mut learner, &book);
        }
        None => {
            // No command - start REPL
            run_repl(&mut learner, brain_path);
        }
    }
}
