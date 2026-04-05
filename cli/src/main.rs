// Seed-Intelligence CLI - Main Entry Point
// Hebbian learning based embodied intelligence system

mod brain;
mod crawler;
mod inference;
mod learner;
mod nlp;
mod lm;

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
    Stats,
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
    /// Interactive REPL mode
    Repl,
}

fn get_brain_path(brain_arg: Option<PathBuf>) -> PathBuf {
    if let Some(p) = brain_arg {
        return p;
    }

    // Use project data directory
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "seed-intelligence", "Seed-Intelligence") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).ok();
        return data_dir.join("brain.json");
    }

    PathBuf::from("brain.json")
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

    // Cleanup
    let cleanup = learner.cleanup(false);
    println!("🧹 Cleanup: {} relations, {} concepts pruned",
        cleanup.pruned_relations, cleanup.pruned_concepts);

    // Save
    if let Err(e) = learner.save() {
        eprintln!("Error saving brain: {}", e);
    }

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

    if let Err(e) = learner.save() {
        eprintln!("Error saving brain: {}", e);
    }
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

fn run_stats(learner: &learner::IncrementalLearner) {
    let stats = learner.get_stats();
    println!("\n📊 Knowledge Graph Stats");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("总概念数: {}", stats.total_concepts);
    println!("总关系数: {}", stats.total_relations);
    println!("平均权重: {:.4}", stats.avg_weight);
    println!("平均能量: {:.4}", stats.avg_energy);
    println!("\n🔝 Top Concepts:");
    for tc in stats.top_concepts.iter().take(10) {
        println!("  {} (energy: {:.2}, count: {})", tc.name, tc.energy, tc.count);
    }
}

fn run_brain(learner: &learner::IncrementalLearner) {
    println!("\n🧠 Knowledge Graph (brain.json)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("概念数: {}", learner.brain.total_concepts());
    println!("关系数: {}", learner.brain.total_relations());

    if !learner.brain.concepts.is_empty() {
        println!("\n📝 Concepts (first 20):");
        for (i, (name, c)) in learner.brain.concepts.iter().take(20).enumerate() {
            println!("  {}. {} (energy: {:.2}, count: {})", i + 1, name, c.energy, c.count);
        }
    }

    if !learner.brain.relations.is_empty() {
        println!("\n🔗 Relations (first 20):");
        for (i, r) in learner.brain.relations.values().take(20).enumerate() {
            println!("  {}. {} ↔ {} (weight: {:.2})", i + 1, r.source, r.target, r.weight);
        }
    }
}

fn run_clear(learner: &mut learner::IncrementalLearner) {
    learner.clear();
    if let Err(e) = learner.save() {
        eprintln!("Error saving brain: {}", e);
    }
    println!("✅ Knowledge base cleared!");
}

fn run_concept(learner: &learner::IncrementalLearner, name: &str) {
    if let Some(c) = learner.get_concept(name) {
        println!("\n📌 Concept: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
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
    for concept_name in learner.brain.concepts.keys() {
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

fn run_repl(learner: &mut learner::IncrementalLearner, brain_path: PathBuf) {
    println!("\n╔═══════════════════════════════════════════╗");
    println!("║     🌱 Seed-Intelligence REPL             ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  直接输入问题进行问答                       ║");
    println!("║  /help 查看所有命令                        ║");
    println!("║  /exit 或 Ctrl+C 退出                     ║");
    println!("╚═══════════════════════════════════════════╝\n");

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
            handle_command(input, learner, &brain_path);
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

fn handle_command(input: &str, learner: &mut learner::IncrementalLearner, brain_path: &PathBuf) {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).map(|s| *s).unwrap_or("");

    match cmd.as_str() {
        "/help" | "/h" | "help" => {
            print_help();
        }
        "/stats" => {
            run_stats(learner);
        }
        "/brain" => {
            run_brain(learner);
        }
        "/clear" => {
            run_clear(learner);
            // Reload
            learner.load();
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
                // Save after learning
                learner.save().ok();
            }
        }
        "/init" => {
            if args.is_empty() {
                println!("用法: /init <概念>");
            } else {
                // Use async runtime for init
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(run_init(learner, args, true));
            }
        }
        "/reload" => {
            learner.load();
            println!("✅ Brain reloaded from disk");
        }
        "/save" => {
            if let Err(e) = learner.save() {
                eprintln!("Error saving: {}", e);
            } else {
                println!("✅ Brain saved to {:?}", brain_path);
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
║    /reload          重新加载知识库         ║
║    /save            保存知识库             ║
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

    let mut learner = learner::IncrementalLearner::new(Some(brain_path.clone()));

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
        Some(Commands::Stats) => {
            run_stats(&learner);
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
        Some(Commands::Repl) => {
            run_repl(&mut learner, brain_path);
        }
        None => {
            // No command - start REPL
            run_repl(&mut learner, brain_path);
        }
    }
}
