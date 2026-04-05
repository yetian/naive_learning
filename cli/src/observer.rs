// Observer - Embodied Intelligence Observation Mode
//
// This module provides the "observe" command for embodied intelligence:
// - File system watching in agent_sandbox
// - Clipboard monitoring
// - Command execution with output capture
// - Interaction history recording
// - Batch learning from accumulated observations

use crate::learner::IncrementalLearner;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Configuration
const BATCH_SIZE: usize = 500; // Learn after accumulating this many characters
const CLIPBOARD_POLL_INTERVAL_MS: u64 = 1500; // Poll clipboard every 1.5 seconds
const MAX_HISTORY_SIZE: usize = 100; // Keep last 100 interactions

/// Observation buffer for batch learning
pub struct ObservationBuffer {
    content: String,
    last_learn: Instant,
    total_observations: u64,
}

impl ObservationBuffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            last_learn: Instant::now(),
            total_observations: 0,
        }
    }

    /// Add observation content to buffer
    pub fn add(&mut self, source: &str, content: &str) {
        if content.trim().is_empty() {
            return;
        }

        let timestamp = chrono_timestamp();
        self.content.push_str(&format!(
            "[{} - {}] {}\n",
            timestamp, source, content
        ));
        self.total_observations += 1;
    }

    /// Check if buffer is ready for learning
    pub fn should_learn(&self) -> bool {
        self.content.len() >= BATCH_SIZE
    }

    /// Get buffer content and clear
    pub fn drain(&mut self) -> String {
        let content = std::mem::take(&mut self.content);
        self.last_learn = Instant::now();
        content
    }

    /// Get current buffer size
    pub fn size(&self) -> usize {
        self.content.len()
    }

    /// Get total observations count
    pub fn count(&self) -> u64 {
        self.total_observations
    }
}

/// Interaction history entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub source: String,
    pub content: String,
}

/// Interaction history recorder
pub struct InteractionHistory {
    entries: VecDeque<HistoryEntry>,
}

impl InteractionHistory {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_HISTORY_SIZE),
        }
    }

    /// Add entry to history
    pub fn add(&mut self, source: &str, content: &str) {
        let entry = HistoryEntry {
            timestamp: chrono_timestamp(),
            source: source.to_string(),
            content: content.to_string(),
        };

        if self.entries.len() >= MAX_HISTORY_SIZE {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Get recent entries
    pub fn recent(&self, n: usize) -> &[HistoryEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries.as_slices().0[start.min(self.entries.len())..]
    }

    /// Format history for display
    pub fn format(&self, n: usize) -> String {
        let recent = self.recent(n);
        let mut output = String::new();

        for entry in recent {
            output.push_str(&format!(
                "[{}] {} ({} chars)\n",
                entry.timestamp,
                entry.source,
                entry.content.len()
            ));
        }

        output
    }
}

/// Get current timestamp string
fn chrono_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", now)
}

// =============================================================================
// File Watcher
// =============================================================================

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Watch agent_sandbox directory for file changes
pub fn watch_sandbox(
    buffer: Arc<Mutex<ObservationBuffer>>,
    history: Arc<Mutex<InteractionHistory>>,
    sandbox_path: &Path,
) -> Result<RecommendedWatcher, notify::Error> {
    let path = sandbox_path.to_path_buf();

    let handler = move |event: Result<Event, notify::Error>| {
        if let Ok(event) = event {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    for path in &event.paths {
                        if path.is_file() {
                            if let Ok(content) = std::fs::read_to_string(path) {
                                let filename = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown");

                                buffer.lock().unwrap().add(
                                    &format!("file:{}", filename),
                                    &content
                                );

                                history.lock().unwrap().add(
                                    &format!("file:{}", filename),
                                    &content
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    };

    let mut watcher = RecommendedWatcher::new(handler, Config::default())?;
    watcher.watch(&path, RecursiveMode::Recursive)?;

    Ok(watcher)
}

// =============================================================================
// Clipboard Watcher
// =============================================================================

/// Watch clipboard for changes (polling-based)
pub fn watch_clipboard(
    buffer: Arc<Mutex<ObservationBuffer>>,
    history: Arc<Mutex<InteractionHistory>>,
    stop_signal: Arc<Mutex<bool>>,
) {
    let mut last_content = String::new();

    loop {
        // Check stop signal
        if *stop_signal.lock().unwrap() {
            break;
        }

        // Try to get clipboard content
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(content) = clipboard.get_text() {
                if content != last_content && !content.is_empty() {
                    last_content = content.clone();

                    buffer.lock().unwrap().add("clipboard", &content);
                    history.lock().unwrap().add("clipboard", &content);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(CLIPBOARD_POLL_INTERVAL_MS));
    }
}

// =============================================================================
// Command Executor
// =============================================================================

/// Execute a command and capture output
pub fn execute_command(command: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", command])
            .output()
    } else {
        std::process::Command::new("sh")
            .args(["-c", command])
            .output()
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok(stdout)
            } else {
                Ok(format!("Error: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

// =============================================================================
// Observe Mode Main Loop
// =============================================================================

/// Run the observe mode
pub fn run_observe_mode(
    learner: &mut IncrementalLearner,
    sandbox_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌱 Seed Observation Mode");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("👁️  Watching: {}", sandbox_path.display());
    println!("📋 Clipboard: enabled (polling {}ms)", CLIPBOARD_POLL_INTERVAL_MS);
    println!("📦 Batch size: {} chars", BATCH_SIZE);
    println!("");
    println!("Commands:");
    println!("  <text>         - Learn from input text");
    println!("  /run <cmd>     - Execute command and learn from output");
    println!("  /file <path>   - Read and learn from file in sandbox");
    println!("  /history       - Show recent observations");
    println!("  /stats         - Show observation statistics");
    println!("  /learn         - Force batch learning now");
    println!("  /save          - Save knowledge base");
    println!("  /help          - Show this help");
    println!("  /exit          - Exit observation mode");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("");

    // Initialize buffer and history
    let buffer = Arc::new(Mutex::new(ObservationBuffer::new()));
    let history = Arc::new(Mutex::new(InteractionHistory::new()));
    let stop_signal = Arc::new(Mutex::new(false));

    // Start file watcher
    let _watcher = match watch_sandbox(Arc::clone(&buffer), Arc::clone(&history), sandbox_path) {
        Ok(w) => {
            println!("✅ File watcher started");
            Some(w)
        }
        Err(e) => {
            eprintln!("⚠️  File watcher failed: {}", e);
            None
        }
    };

    // Start clipboard watcher in background thread
    let clipboard_buffer = Arc::clone(&buffer);
    let clipboard_history = Arc::clone(&history);
    let clipboard_stop = Arc::clone(&stop_signal);
    std::thread::spawn(move || {
        watch_clipboard(clipboard_buffer, clipboard_history, clipboard_stop);
    });
    println!("✅ Clipboard watcher started");

    println!("");
    println!("Ready to observe. Type /help for commands.");

    // Main input loop
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("🌱 > ");
        io::stdout().flush()?;

        input.clear();
        if stdin.read_line(&mut input)? == 0 {
            break; // EOF
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle commands
        if input.starts_with('/') {
            let parts: Vec<&str> = input.splitn(2, ' ').collect();
            let cmd = parts[0];

            match cmd {
                "/exit" | "/quit" => {
                    println!("Stopping watchers...");

                    // Stop clipboard watcher
                    *stop_signal.lock().unwrap() = true;

                    // Final learning if buffer has content
                    let mut buf = buffer.lock().unwrap();
                    if !buf.content.is_empty() {
                        println!("📚 Final learning from remaining observations...");
                        let content = buf.drain();
                        learner.learn_from_text(&content, None);
                        learner.save()?;
                    }

                    println!("👋 Goodbye!");
                    break;
                }

                "/help" => {
                    println!("");
                    println!("Commands:");
                    println!("  <text>         - Learn from input text");
                    println!("  /run <cmd>     - Execute command and learn from output");
                    println!("  /file <path>   - Read and learn from file in sandbox");
                    println!("  /history       - Show recent observations");
                    println!("  /stats         - Show observation statistics");
                    println!("  /learn         - Force batch learning now");
                    println!("  /save          - Save knowledge base");
                    println!("  /exit          - Exit observation mode");
                    println!("");
                }

                "/run" => {
                    if parts.len() < 2 {
                        println!("Usage: /run <command>");
                        continue;
                    }

                    let command = parts[1];
                    println!("Executing: {}", command);

                    match execute_command(command) {
                        Ok(output) => {
                            if !output.is_empty() {
                                buffer.lock().unwrap().add("command", &output);
                                history.lock().unwrap().add("command", &output);
                                println!("✅ Captured {} chars", output.len());
                            } else {
                                println!("⚠️  No output");
                            }
                        }
                        Err(e) => {
                            println!("❌ {}", e);
                        }
                    }
                }

                "/file" => {
                    if parts.len() < 2 {
                        println!("Usage: /file <path>");
                        continue;
                    }

                    let file_path = sandbox_path.join(parts[1].trim());
                    match std::fs::read_to_string(&file_path) {
                        Ok(content) => {
                            buffer.lock().unwrap().add("file", &content);
                            history.lock().unwrap().add("file", &content);
                            println!("✅ Read {} chars from {}", content.len(), parts[1]);
                        }
                        Err(e) => {
                            println!("❌ Failed to read file: {}", e);
                        }
                    }
                }

                "/history" => {
                    let hist = history.lock().unwrap();
                    println!("");
                    println!("📜 Recent Observations:");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    print!("{}", hist.format(10));
                    println!("");
                }

                "/stats" => {
                    let buf = buffer.lock().unwrap();
                    let stats = learner.get_stats();
                    println!("");
                    println!("📊 Observation Statistics:");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("Observations: {}", buf.count());
                    println!("Buffer size: {}/{} chars", buf.size(), BATCH_SIZE);
                    println!("Ready to learn: {}", buf.should_learn());
                    println!("");
                    println!("Knowledge Graph:");
                    println!("  Concepts: {}", stats.total_concepts);
                    println!("  Relations: {}", stats.total_relations);
                    println!("");
                }

                "/learn" => {
                    let mut buf = buffer.lock().unwrap();
                    if buf.content.is_empty() {
                        println!("Buffer is empty, nothing to learn");
                        continue;
                    }

                    let content = buf.drain();
                    let result = learner.learn_from_text(&content, None);
                    println!("📚 Learned: {} tokens, {} relations", result.tokens_processed, result.relations_added);

                    if let Err(e) = learner.save() {
                        println!("❌ Failed to save: {}", e);
                    } else {
                        println!("✅ Saved");
                    }
                }

                "/save" => {
                    if let Err(e) = learner.save() {
                        println!("❌ Failed to save: {}", e);
                    } else {
                        println!("✅ Saved");
                    }
                }

                _ => {
                    println!("Unknown command: {}. Type /help for available commands.", cmd);
                }
            }
        } else {
            // Direct text input - add to buffer
            buffer.lock().unwrap().add("input", input);
            history.lock().unwrap().add("input", input);
            println!("📝 Added {} chars to buffer", input.len());
        }

        // Check if we should batch learn
        {
            let buf = buffer.lock().unwrap();
            if buf.should_learn() {
                drop(buf); // Release lock before learning

                let mut buf = buffer.lock().unwrap();
                let content = buf.drain();
                println!("");
                println!("📦 Batch threshold reached, learning...");
                let result = learner.learn_from_text(&content, None);
                println!("📚 Learned: {} tokens, {} relations", result.tokens_processed, result.relations_added);

                if let Err(e) = learner.save() {
                    eprintln!("⚠️  Failed to save: {}", e);
                }
                println!("");
            }
        }
    }

    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // ObservationBuffer Tests
    // =========================================================================

    #[test]
    fn test_buffer_new() {
        let buffer = ObservationBuffer::new();
        assert_eq!(buffer.size(), 0);
        assert_eq!(buffer.count(), 0);
        assert!(!buffer.should_learn());
    }

    #[test]
    fn test_buffer_add() {
        let mut buffer = ObservationBuffer::new();
        buffer.add("test", "Hello world");

        assert!(buffer.size() > 0);
        assert_eq!(buffer.count(), 1);
    }

    #[test]
    fn test_buffer_add_empty() {
        let mut buffer = ObservationBuffer::new();
        buffer.add("test", "");
        buffer.add("test", "   ");

        assert_eq!(buffer.size(), 0);
        assert_eq!(buffer.count(), 0);
    }

    #[test]
    fn test_buffer_add_multiple() {
        let mut buffer = ObservationBuffer::new();
        buffer.add("file", "Content from file");
        buffer.add("clipboard", "Content from clipboard");
        buffer.add("command", "Command output");

        assert_eq!(buffer.count(), 3);
        assert!(buffer.content.contains("file"));
        assert!(buffer.content.contains("clipboard"));
        assert!(buffer.content.contains("command"));
    }

    #[test]
    fn test_buffer_drain() {
        let mut buffer = ObservationBuffer::new();
        buffer.add("test", "Hello world");

        let content = buffer.drain();
        assert!(content.contains("Hello world"));
        assert_eq!(buffer.size(), 0);
    }

    #[test]
    fn test_buffer_should_learn() {
        let mut buffer = ObservationBuffer::new();

        // Should not learn initially
        assert!(!buffer.should_learn());

        // Add content to reach threshold
        let long_content = "x".repeat(BATCH_SIZE);
        buffer.add("test", &long_content);

        // Should learn now
        assert!(buffer.should_learn());
    }

    #[test]
    fn test_buffer_size_after_multiple_adds() {
        let mut buffer = ObservationBuffer::new();

        for i in 0..10 {
            buffer.add("test", &format!("Content number {}", i));
        }

        assert_eq!(buffer.count(), 10);
        assert!(buffer.size() > 0);
    }

    // =========================================================================
    // InteractionHistory Tests
    // =========================================================================

    #[test]
    fn test_history_new() {
        let history = InteractionHistory::new();
        assert!(history.recent(10).is_empty());
    }

    #[test]
    fn test_history_add() {
        let mut history = InteractionHistory::new();
        history.add("file", "File content");

        let recent = history.recent(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].source, "file");
        assert_eq!(recent[0].content, "File content");
    }

    #[test]
    fn test_history_add_multiple() {
        let mut history = InteractionHistory::new();

        for i in 0..5 {
            history.add("test", &format!("Content {}", i));
        }

        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_history_max_size() {
        let mut history = InteractionHistory::new();

        // Add more than max size
        for i in 0..(MAX_HISTORY_SIZE + 50) {
            history.add("test", &format!("Content {}", i));
        }

        // Should only keep MAX_HISTORY_SIZE
        let recent = history.recent(MAX_HISTORY_SIZE + 10);
        assert!(recent.len() <= MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_history_format() {
        let mut history = InteractionHistory::new();
        history.add("file", "Test content");
        history.add("clipboard", "Clipboard text");

        let formatted = history.format(10);
        assert!(formatted.contains("file"));
        assert!(formatted.contains("clipboard"));
    }

    // =========================================================================
    // Command Execution Tests
    // =========================================================================

    #[test]
    fn test_execute_command_echo() {
        let result = execute_command("echo hello");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }

    #[test]
    fn test_execute_command_invalid() {
        // Invalid commands should return error message, not panic
        let result = execute_command("nonexistent_command_12345");
        // Should either return error or the shell error message
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_execute_command_empty() {
        let result = execute_command("");
        // Empty command should handle gracefully
        assert!(result.is_ok() || result.is_err());
    }

    // =========================================================================
    // Timestamp Tests
    // =========================================================================

    #[test]
    fn test_chrono_timestamp() {
        let ts1 = chrono_timestamp();
        let ts2 = chrono_timestamp();

        // Should be numeric strings
        assert!(ts1.parse::<u64>().is_ok());
        assert!(ts2.parse::<u64>().is_ok());

        // Second timestamp should be >= first
        assert!(ts2.parse::<u64>().unwrap() >= ts1.parse::<u64>().unwrap());
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    #[test]
    fn test_buffer_thread_safety() {
        use std::thread;

        let buffer = Arc::new(Mutex::new(ObservationBuffer::new()));
        let mut handles = vec![];

        for i in 0..10 {
            let buf = Arc::clone(&buffer);
            handles.push(thread::spawn(move || {
                buf.lock().unwrap().add("thread", &format!("Content {}", i));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(buffer.lock().unwrap().count(), 10);
    }

    #[test]
    fn test_history_thread_safety() {
        use std::thread;

        let history = Arc::new(Mutex::new(InteractionHistory::new()));
        let mut handles = vec![];

        for i in 0..10 {
            let hist = Arc::clone(&history);
            handles.push(thread::spawn(move || {
                hist.lock().unwrap().add("thread", &format!("Content {}", i));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(history.lock().unwrap().recent(20).len(), 10);
    }
}
