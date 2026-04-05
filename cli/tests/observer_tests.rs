// Integration tests for observer module

use seed_intelligence::observer::{ObservationBuffer, InteractionHistory, execute_command};

#[test]
fn test_observation_buffer_basic() {
    let mut buffer = ObservationBuffer::new();
    assert_eq!(buffer.size(), 0);
    assert_eq!(buffer.count(), 0);

    buffer.add("test", "Hello world");
    assert!(buffer.size() > 0);
    assert_eq!(buffer.count(), 1);
}

#[test]
fn test_observation_buffer_empty_content() {
    let mut buffer = ObservationBuffer::new();
    buffer.add("test", "");
    buffer.add("test", "   ");

    assert_eq!(buffer.size(), 0);
    assert_eq!(buffer.count(), 0);
}

#[test]
fn test_observation_buffer_drain() {
    let mut buffer = ObservationBuffer::new();
    buffer.add("source1", "Content 1");
    buffer.add("source2", "Content 2");

    let content = buffer.drain();
    assert!(content.contains("source1"));
    assert!(content.contains("source2"));
    assert_eq!(buffer.size(), 0);
}

#[test]
fn test_interaction_history_basic() {
    let mut history = InteractionHistory::new();
    assert!(history.recent(10).is_empty());

    history.add("file", "File content");
    let recent = history.recent(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].source, "file");
}

#[test]
fn test_interaction_history_multiple() {
    let mut history = InteractionHistory::new();

    for i in 0..5 {
        history.add("test", &format!("Content {}", i));
    }

    let recent = history.recent(3);
    assert_eq!(recent.len(), 3);
}

#[test]
fn test_execute_command_simple() {
    let result = execute_command("echo test");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("test"));
}

#[test]
fn test_buffer_accumulation() {
    let mut buffer = ObservationBuffer::new();

    // Add multiple observations
    buffer.add("file", "First observation");
    buffer.add("clipboard", "Second observation");
    buffer.add("command", "Third observation");

    assert_eq!(buffer.count(), 3);

    // Content should contain all sources
    let content = buffer.drain();
    assert!(content.contains("file"));
    assert!(content.contains("clipboard"));
    assert!(content.contains("command"));
}

#[test]
fn test_history_format() {
    let mut history = InteractionHistory::new();
    history.add("file", "Test content");

    let formatted = history.format(10);
    assert!(formatted.contains("file"));
}
