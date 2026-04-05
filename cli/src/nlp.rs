// NLP Module - Using Jieba for Chinese Word Segmentation
// Provides better tokenization for Chinese and English

use lazy_static::lazy_static;
use std::collections::HashSet;

// Stop words loaded from stop-words crate
lazy_static! {
    static ref STOP_WORDS: HashSet<String> = {
        let mut s: HashSet<String> = HashSet::new();

        // Load Chinese stop words from stop-words crate
        let zh_stopwords = stop_words::get(stop_words::LANGUAGE::Chinese);
        for word in zh_stopwords {
            s.insert(word.to_string());
        }

        // Load English stop words from stop-words crate
        let en_stopwords = stop_words::get(stop_words::LANGUAGE::English);
        for word in en_stopwords {
            s.insert(word.to_lowercase());
        }

        // Add some additional common Chinese stop words not in the library
        let extra_zh = [
            "个", "一", "不", "也", "都", "就", "而", "及", "以", "对",
            "可", "能", "会", "被", "于", "从", "到", "把", "将", "为",
            "但", "却", "又", "如", "因", "所", "并", "其", "之", "来",
            "去", "上", "下", "中", "大", "小", "多", "少", "最", "更",
            "很", "太", "过", "要", "该", "它", "此", "则", "着", "过",
            "得", "地", "等", "当", "给", "让", "使", "比", "由", "出",
        ];
        for word in extra_zh {
            s.insert(word.to_string());
        }

        s
    };
}

/// Tokenizer using Jieba for Chinese segmentation
pub struct TokenizerWrapper {
    jieba: jieba_rs::Jieba,
}

impl TokenizerWrapper {
    /// Create a new tokenizer
    pub fn new() -> Self {
        let mut jieba = jieba_rs::Jieba::new();

        // Load custom dictionary from file if it exists
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "seed-intelligence", "Seed-Intelligence") {
            let dict_path = proj_dirs.data_dir().join("custom_dict.txt");
            if dict_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&dict_path) {
                    for line in content.lines() {
                        let word = line.split_whitespace().next().unwrap_or("");
                        if !word.is_empty() && !word.starts_with('#') {
                            jieba.add_word(word, None, None);
                        }
                    }
                    println!("[NLP] Loaded custom dictionary from {:?}", dict_path);
                }
            }
        }

        Self { jieba }
    }

    /// Tokenize text using Jieba for Chinese and regex for English
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return vec![];
        }

        let mut tokens = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Use Jieba for Chinese word segmentation
        let words = self.jieba.cut(text, false); // cut_all=false for more accurate segmentation

        for word in words {
            let word = word.trim();
            if word.is_empty() {
                continue;
            }

            // Skip single characters (unless they're meaningful)
            if word.len() < 2 && word.chars().all(|c| c >= '\u{4e00}' && c <= '\u{9fff}') {
                continue;
            }

            // Skip stop words
            if STOP_WORDS.contains(word) {
                continue;
            }

            // For English words, lowercase and check length
            if word.chars().all(|c| c.is_ascii_alphabetic()) {
                let word_lower = word.to_lowercase();
                if word_lower.len() < 2 || STOP_WORDS.contains(word_lower.as_str()) {
                    continue;
                }
                if !seen.contains(&word_lower) {
                    seen.insert(word_lower.clone());
                    tokens.push(word_lower);
                }
            } else {
                // Chinese or mixed - keep original
                if !seen.contains(word) {
                    seen.insert(word.to_string());
                    tokens.push(word.to_string());
                }
            }
        }

        tokens
    }

    /// Tokenize and filter stop words
    pub fn tokenize_filtered(&self, text: &str) -> Vec<String> {
        self.tokenize(text)
    }

    /// Get token IDs for model input (character-level)
    pub fn encode(&self, text: &str) -> Vec<u32> {
        text.chars().map(|c| c as u32).collect()
    }

    /// Decode token IDs back to text
    pub fn decode(&self, ids: &[u32]) -> String {
        ids.iter()
            .filter_map(|&id| char::from_u32(id))
            .collect()
    }
}

impl Default for TokenizerWrapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple word-level tokenization (fallback without Jieba instance)
pub fn simple_tokenize(text: &str) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    let mut tokens = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Use global jieba instance
    let jieba = jieba_rs::Jieba::new();
    let words = jieba.cut(text, false);

    for word in words {
        let word = word.trim();
        if word.is_empty() || word.len() < 2 {
            continue;
        }

        if STOP_WORDS.contains(word) {
            continue;
        }

        // For English words, lowercase
        if word.chars().all(|c| c.is_ascii_alphabetic()) {
            let word_lower = word.to_lowercase();
            if word_lower.len() >= 2 && !STOP_WORDS.contains(word_lower.as_str()) && !seen.contains(&word_lower) {
                seen.insert(word_lower.clone());
                tokens.push(word_lower);
            }
        } else if !seen.contains(word) {
            seen.insert(word.to_string());
            tokens.push(word.to_string());
        }
    }

    tokens
}

/// Filter stop words from tokens
pub fn filter_stop_words(tokens: &[String]) -> Vec<String> {
    tokens
        .iter()
        .filter(|t| t.len() >= 2 && !STOP_WORDS.contains(t.as_str()))
        .cloned()
        .collect()
}

/// Get a global tokenizer instance
pub fn get_tokenizer() -> TokenizerWrapper {
    TokenizerWrapper::new()
}

// Re-exports for compatibility
pub type Tokenizer = TokenizerWrapper;
pub use simple_tokenize as tokenize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokenizer = get_tokenizer();
        let tokens = tokenizer.tokenize("人工智能是计算机科学的一个分支");
        println!("Tokens: {:?}", tokens);
        assert!(!tokens.is_empty());
        // Should contain "人工智能" as a whole word
        assert!(tokens.contains(&"人工智能".to_string()) || tokens.contains(&"计算机科学".to_string()));
    }

    #[test]
    fn test_chinese_segmentation() {
        let tokenizer = get_tokenizer();
        let tokens = tokenizer.tokenize("机器学习是人工智能的核心技术之一");
        println!("Tokens: {:?}", tokens);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_filter() {
        let tokens = vec!["水".to_string(), "的".to_string(), "是".to_string(), "重要".to_string()];
        let filtered = filter_stop_words(&tokens);
        assert!(!filtered.contains(&"的".to_string()));
    }
}
