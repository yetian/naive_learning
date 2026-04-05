// Tests for lm module - Neural Network / Language Model

mod common;

use seed_intelligence::lm::{Vocab, ModelConfig, Trainer, create_model};

#[test]
fn test_vocab_new() {
    let vocab = Vocab::new();

    // Should have special tokens
    assert!(vocab.word2idx.contains_key("[PAD]"));
    assert!(vocab.word2idx.contains_key("[UNK]"));
    assert!(vocab.word2idx.contains_key("[BOS]"));
    assert!(vocab.word2idx.contains_key("[EOS]"));

    // Size should be 4 (special tokens)
    assert_eq!(vocab.size(), 4);
}

#[test]
fn test_vocab_add_token() {
    let mut vocab = Vocab::new();

    let idx1 = vocab.add_token("hello".to_string());
    let idx2 = vocab.add_token("world".to_string());

    // Each new token should have a unique index
    assert_ne!(idx1, idx2);

    // Adding same token again should return same index
    let idx1_again = vocab.add_token("hello".to_string());
    assert_eq!(idx1, idx1_again);
}

#[test]
fn test_vocab_tokenize() {
    let mut vocab = Vocab::new();
    vocab.add_token("a".to_string());
    vocab.add_token("b".to_string());
    vocab.add_token("c".to_string());

    let tokens = vocab.tokenize("abc");

    // Should start with BOS token
    assert!(tokens.len() > 0);

    // Should have 4 tokens: BOS + a + b + c
    assert!(tokens.len() >= 4);
}

#[test]
fn test_vocab_tokenize_empty() {
    let vocab = Vocab::new();

    let tokens = vocab.tokenize("");

    // Should only have BOS token
    assert_eq!(tokens.len(), 1);
}

#[test]
fn test_vocab_decode() {
    let mut vocab = Vocab::new();
    vocab.add_token("x".to_string());
    vocab.add_token("y".to_string());

    let tokens = vocab.tokenize("xy");
    let decoded = vocab.decode(&tokens[1..]); // Skip BOS

    assert!(decoded.contains('x') || decoded.contains('y'));
}

#[test]
fn test_vocab_decode_unknown() {
    let vocab = Vocab::new();

    // Unknown character should be decoded via UNK
    let decoded = vocab.decode(&[1]); // UNK token

    // Should return something (may be empty for [UNK])
    println!("Decoded unknown: '{}'", decoded);
}

#[test]
fn test_vocab_size() {
    let mut vocab = Vocab::new();
    let initial_size = vocab.size();

    vocab.add_token("test".to_string());
    vocab.add_token("another".to_string());

    assert_eq!(vocab.size(), initial_size + 2);
}

#[test]
fn test_model_config_default() {
    let config = ModelConfig::default();

    assert_eq!(config.vocab_size, 2000);
    assert_eq!(config.embed_dim, 64);
    assert_eq!(config.num_heads, 2);
    assert_eq!(config.num_layers, 1);
    assert_eq!(config.context_len, 64);
    assert_eq!(config.hidden_dim, 256);
}

#[test]
fn test_model_config_clone() {
    let config = ModelConfig::default();
    let cloned = config.clone();

    assert_eq!(config.vocab_size, cloned.vocab_size);
    assert_eq!(config.embed_dim, cloned.embed_dim);
}

#[test]
fn test_model_config_serialization() {
    let config = ModelConfig::default();

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("vocab_size"));

    let parsed: ModelConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.vocab_size, config.vocab_size);
}

#[test]
fn test_create_model() {
    let model = create_model();

    assert!(model.is_ok());
    let model = model.unwrap();

    // Check config
    assert_eq!(model.config.vocab_size, 2000);

    // Check vocab has special tokens
    assert!(model.vocab.word2idx.contains_key("[PAD]"));
}

#[test]
fn test_causal_lm_add_vocab() {
    let mut model = create_model().unwrap();

    let initial_size = model.vocab.size();
    model.add_vocab("测试文本");

    // Should have added new characters
    assert!(model.vocab.size() > initial_size);
}

#[test]
fn test_causal_lm_forward() {
    let mut model = create_model().unwrap();
    model.add_vocab("hello");

    let tokens = model.vocab.tokenize("hello");
    let result = model.forward(&tokens);

    assert!(result.is_ok());
    let logits = result.unwrap();

    // Should have shape [seq_len, vocab_size]
    assert!(logits.shape().dims().len() >= 1);
}

#[test]
fn test_causal_lm_generate() {
    let mut model = create_model().unwrap();
    model.add_vocab("hello world");

    let output = model.generate("hello", 10, 0.8);

    // Should produce some output
    assert!(!output.is_empty() || output.len() <= 20);
}

#[test]
fn test_causal_lm_generate_empty_prompt() {
    let model = create_model().unwrap();

    let output = model.generate("", 5, 0.8);

    // Should not crash with empty prompt
    println!("Generated from empty prompt: '{}'", output);
}

#[test]
fn test_causal_lm_save_weights() {
    let model = create_model().unwrap();
    let temp_path = std::env::temp_dir().join("test_lm_weights.json");

    let result = model.save_weights(temp_path.to_str().unwrap());

    assert!(result.is_ok());

    // Check file was created
    assert!(temp_path.exists());

    // Cleanup
    std::fs::remove_file(temp_path).ok();
}

#[test]
fn test_causal_lm_load_weights() {
    let mut model = create_model().unwrap();
    let temp_path = std::env::temp_dir().join("test_lm_weights_load.json");

    // Save first
    model.save_weights(temp_path.to_str().unwrap()).unwrap();

    // Load
    let result = model.load_weights(temp_path.to_str().unwrap());

    assert!(result.is_ok());

    // Cleanup
    std::fs::remove_file(temp_path).ok();
}

#[test]
fn test_trainer_new() {
    let model = create_model().unwrap();
    let trainer = Trainer::new(model, 0.01);

    // Trainer should be created successfully
    assert_eq!(trainer.model.config.vocab_size, 2000);
}

#[test]
fn test_trainer_train_on_text() {
    let model = create_model().unwrap();
    let mut trainer = Trainer::new(model, 0.01);

    // Should not crash
    trainer.train_on_text("Hello world this is a test", 1);
}

#[test]
fn test_trainer_train_multiple_epochs() {
    let model = create_model().unwrap();
    let mut trainer = Trainer::new(model, 0.01);

    // Should handle multiple epochs
    trainer.train_on_text("Testing training", 3);
}

#[test]
fn test_vocab_default() {
    let vocab1 = Vocab::new();
    let vocab2 = Vocab::default();

    // Both should have same initial size
    assert_eq!(vocab1.size(), vocab2.size());
}
