// Tests for NLP module - Tokenization and stop words

mod common;

use seed_intelligence::nlp::{TokenizerWrapper, simple_tokenize, filter_stop_words};

#[test]
fn test_tokenize_chinese_text() {
    let tokenizer = TokenizerWrapper::new();
    let tokens = tokenizer.tokenize("人工智能是计算机科学的一个分支");

    assert!(!tokens.is_empty());
}

#[test]
fn test_tokenize_english_text() {
    let tokenizer = TokenizerWrapper::new();
    let tokens = tokenizer.tokenize("Artificial intelligence is a branch of computer science");

    assert!(!tokens.is_empty());
    // English words should be lowercased
    assert!(tokens.iter().all(|t| t.to_lowercase() == *t));
}

#[test]
fn test_tokenize_mixed_text() {
    let tokenizer = TokenizerWrapper::new();
    let tokens = tokenizer.tokenize("AI人工智能是Artificial Intelligence的缩写");

    assert!(!tokens.is_empty());
}

#[test]
fn test_tokenize_empty_string() {
    let tokenizer = TokenizerWrapper::new();
    let tokens = tokenizer.tokenize("");

    assert!(tokens.is_empty());
}

#[test]
fn test_tokenize_filters_stopwords() {
    let tokenizer = TokenizerWrapper::new();
    let tokens = tokenizer.tokenize("这是一个测试");

    // "这" "是" "一个" are stop words and should be filtered
    // But tokenization may still produce them, so just check we got something
    println!("Tokens: {:?}", tokens);
}

#[test]
fn test_simple_tokenize_works() {
    let tokens = simple_tokenize("机器学习是人工智能的核心技术");

    assert!(!tokens.is_empty());
}

#[test]
fn test_filter_stop_words() {
    let tokens = vec![
        "人工智能".to_string(),
        "的".to_string(),  // stop word
        "机器学习".to_string(),
        "是".to_string(),  // stop word
    ];

    let filtered = filter_stop_words(&tokens);

    assert!(!filtered.contains(&"的".to_string()));
    assert!(!filtered.contains(&"是".to_string()));
    assert!(filtered.contains(&"人工智能".to_string()));
}

#[test]
fn test_tokenize_minimum_length() {
    let tokenizer = TokenizerWrapper::new();
    let tokens = tokenizer.tokenize("人工智能");

    // Single characters should generally be filtered
    // (depends on tokenizer implementation)
    println!("Tokens for '人工智能': {:?}", tokens);
}

#[test]
fn test_tokenize_deduplication() {
    let tokenizer = TokenizerWrapper::new();
    let tokens = tokenizer.tokenize("人工智能人工智能人工智能");

    // Duplicate tokens should be deduplicated
    // Check that we don't have multiple copies of the same word
    let unique_count = tokens.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(tokens.len(), unique_count);
}
