// NLP Module - Using Jieba for Chinese Word Segmentation
// Provides better tokenization for Chinese and English

use lazy_static::lazy_static;
use std::collections::HashSet;

// Stop words (Chinese + English)
lazy_static! {
    static ref STOP_WORDS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        // Chinese stop words
        s.insert("的"); s.insert("是"); s.insert("在"); s.insert("了"); s.insert("和");
        s.insert("与"); s.insert("或"); s.insert("有"); s.insert("这"); s.insert("那");
        s.insert("个"); s.insert("一"); s.insert("不"); s.insert("也"); s.insert("都");
        s.insert("就"); s.insert("而"); s.insert("及"); s.insert("以"); s.insert("对");
        s.insert("可"); s.insert("能"); s.insert("会"); s.insert("被"); s.insert("于");
        s.insert("从"); s.insert("到"); s.insert("把"); s.insert("将"); s.insert("为");
        s.insert("但"); s.insert("却"); s.insert("又"); s.insert("如"); s.insert("因");
        s.insert("所"); s.insert("并"); s.insert("其"); s.insert("之"); s.insert("来");
        s.insert("去"); s.insert("上"); s.insert("下"); s.insert("中"); s.insert("大");
        s.insert("小"); s.insert("多"); s.insert("少"); s.insert("最"); s.insert("更");
        s.insert("很"); s.insert("太"); s.insert("过"); s.insert("要"); s.insert("该");
        s.insert("我们"); s.insert("你们"); s.insert("他们"); s.insert("她们");
        s.insert("它们"); s.insert("这个"); s.insert("那个"); s.insert("可以");
        s.insert("没有"); s.insert("这样"); s.insert("那样"); s.insert("自己");
        s.insert("已经"); s.insert("因为"); s.insert("所以"); s.insert("但是");
        s.insert("而且"); s.insert("或者"); s.insert("如果"); s.insert("虽然");
        s.insert("只是"); s.insert("就是"); s.insert("还是"); s.insert("应该");
        s.insert("需要"); s.insert("可能"); s.insert("关于");
        // More Chinese stop words
        s.insert("一个"); s.insert("一种"); s.insert("之"); s.insert("者");
        s.insert("着"); s.insert("过"); s.insert("得"); s.insert("地");
        s.insert("等"); s.insert("当"); s.insert("给"); s.insert("让");
        s.insert("使"); s.insert("比"); s.insert("由"); s.insert("此");
        // English stop words
        s.insert("the"); s.insert("a"); s.insert("an"); s.insert("is"); s.insert("are");
        s.insert("was"); s.insert("were"); s.insert("be"); s.insert("been"); s.insert("being");
        s.insert("have"); s.insert("has"); s.insert("had"); s.insert("do"); s.insert("does");
        s.insert("did"); s.insert("will"); s.insert("would"); s.insert("could"); s.insert("should");
        s.insert("may"); s.insert("might"); s.insert("must"); s.insert("shall"); s.insert("can");
        s.insert("need"); s.insert("dare"); s.insert("ought"); s.insert("used"); s.insert("to");
        s.insert("of"); s.insert("in"); s.insert("for"); s.insert("on"); s.insert("with");
        s.insert("at"); s.insert("by"); s.insert("from"); s.insert("as"); s.insert("into");
        s.insert("through"); s.insert("during"); s.insert("before"); s.insert("after");
        s.insert("above"); s.insert("below"); s.insert("between"); s.insert("under");
        s.insert("again"); s.insert("further"); s.insert("then"); s.insert("once");
        s.insert("and"); s.insert("but"); s.insert("or"); s.insert("nor"); s.insert("so");
        s.insert("yet"); s.insert("both"); s.insert("either"); s.insert("neither");
        s.insert("not"); s.insert("only"); s.insert("own"); s.insert("same"); s.insert("than");
        s.insert("too"); s.insert("very"); s.insert("just"); s.insert("also"); s.insert("now");
        s.insert("here"); s.insert("there"); s.insert("when"); s.insert("where"); s.insert("why");
        s.insert("how"); s.insert("all"); s.insert("each"); s.insert("every"); s.insert("few");
        s.insert("more"); s.insert("most"); s.insert("other"); s.insert("some"); s.insert("such");
        s.insert("no"); s.insert("any"); s.insert("what"); s.insert("which"); s.insert("who");
        s.insert("whom"); s.insert("this"); s.insert("that"); s.insert("these"); s.insert("those");
        s.insert("it"); s.insert("its"); s.insert("i"); s.insert("me"); s.insert("my");
        s.insert("we"); s.insert("our"); s.insert("you"); s.insert("your"); s.insert("he");
        s.insert("she"); s.insert("him"); s.insert("her"); s.insert("his");
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
