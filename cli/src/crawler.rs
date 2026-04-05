// Crawler - Wikipedia search and web content retrieval
//
// This module provides web search capabilities for learning.
// Automatically detects query language and searches the appropriate Wikipedia edition.

use serde::{Deserialize, Serialize};

// =============================================================================
// Data Structures
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
struct WikiResponse {
    #[serde(rename = "type")]
    page_type: Option<String>,
    #[serde(rename = "extract")]
    extract: Option<String>,
    title: Option<String>,
    #[serde(rename = "content_urls")]
    content_urls: Option<ContentUrls>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContentUrls {
    desktop: Option<DesktopUrl>,
}

#[derive(Debug, Clone, Deserialize)]
struct DesktopUrl {
    page: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub source: String,
    pub lang: String,
}

// =============================================================================
// Language Detection
// =============================================================================

/// Detected language type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Chinese,      // 中文 (Simplified/Traditional)
    Japanese,     // 日本語
    Korean,       // 한국어
    Russian,      // Русский
    Arabic,       // العربية
    Thai,         // ไทย
    Vietnamese,   // Tiếng Việt
    German,       // Deutsch
    French,       // Français
    Spanish,      // Español
    English,      // English (default)
}

impl Language {
    /// Get Wikipedia language code
    pub fn wiki_code(&self) -> &'static str {
        match self {
            Language::Chinese => "zh",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Russian => "ru",
            Language::Arabic => "ar",
            Language::Thai => "th",
            Language::Vietnamese => "vi",
            Language::German => "de",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::English => "en",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Chinese => "中文",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
            Language::Russian => "Русский",
            Language::Arabic => "العربية",
            Language::Thai => "ไทย",
            Language::Vietnamese => "Tiếng Việt",
            Language::German => "Deutsch",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::English => "English",
        }
    }
}

/// Detect the primary language of a query based on character analysis
pub fn detect_language(text: &str) -> Language {
    let mut chinese_count = 0;
    let mut japanese_only = 0;
    let mut korean_count = 0;
    let mut cyrillic_count = 0;
    let mut arabic_count = 0;
    let mut thai_count = 0;
    let mut vietnamese_chars = 0;
    let mut german_chars = 0;
    let mut french_chars = 0;
    let mut spanish_chars = 0;

    for c in text.chars() {
        let cp = c as u32;

        // CJK Unified Ideographs (common to Chinese/Japanese)
        if (0x4E00..=0x9FFF).contains(&cp) {
            chinese_count += 1;
        }
        // Hiragana and Katakana (Japanese only)
        else if (0x3040..=0x309F).contains(&cp) || (0x30A0..=0x30FF).contains(&cp) {
            japanese_only += 1;
        }
        // Hangul (Korean)
        else if (0xAC00..=0xD7AF).contains(&cp) || (0x1100..=0x11FF).contains(&cp) {
            korean_count += 1;
        }
        // Cyrillic (Russian, etc.)
        else if (0x0400..=0x04FF).contains(&cp) {
            cyrillic_count += 1;
        }
        // Arabic
        else if (0x0600..=0x06FF).contains(&cp) {
            arabic_count += 1;
        }
        // Thai
        else if (0x0E00..=0x0E7F).contains(&cp) {
            thai_count += 1;
        }
        // Vietnamese specific characters
        else if (0x1EA0..=0x1EF9).contains(&cp) {
            vietnamese_chars += 1;
        }
        // German specific: ä, ö, ü, ß, Ä, Ö, Ü
        else if matches!(c, 'ä' | 'ö' | 'ü' | 'ß' | 'Ä' | 'Ö' | 'Ü') {
            german_chars += 1;
        }
        // French specific: é, è, ê, ë, à, â, ù, û, ô, î, ç, œ
        else if matches!(c, 'é' | 'è' | 'ê' | 'ë' | 'à' | 'â' | 'ù' | 'û' | 'ô' | 'î' | 'ç' | 'œ') {
            french_chars += 1;
        }
        // Spanish specific: ñ, á, é, í, ó, ú, ü, ¿, ¡
        else if matches!(c, 'ñ' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü' | '¿' | '¡') {
            spanish_chars += 1;
        }
    }

    // Determine primary language (order matters - more specific first)
    if japanese_only > 0 {
        return Language::Japanese;
    }
    if korean_count > 0 {
        return Language::Korean;
    }
    if chinese_count > 0 {
        return Language::Chinese;
    }
    if cyrillic_count > 0 {
        return Language::Russian;
    }
    if arabic_count > 0 {
        return Language::Arabic;
    }
    if thai_count > 0 {
        return Language::Thai;
    }
    if vietnamese_chars > 0 {
        return Language::Vietnamese;
    }
    // German-specific characters (excluding those shared with French)
    if german_chars > french_chars && german_chars > 0 {
        return Language::German;
    }
    // French-specific characters
    if french_chars > spanish_chars && french_chars > 0 {
        return Language::French;
    }
    // Spanish-specific characters (ñ is very distinctive)
    if spanish_chars > 0 {
        return Language::Spanish;
    }

    Language::English
}

/// Get fallback languages to try if primary language fails
fn get_fallback_languages(primary: Language) -> Vec<Language> {
    match primary {
        Language::Chinese => vec![Language::English, Language::Japanese],
        Language::Japanese => vec![Language::English, Language::Chinese],
        Language::Korean => vec![Language::English],
        Language::Russian => vec![Language::English],
        Language::Arabic => vec![Language::English],
        Language::Thai => vec![Language::English],
        Language::Vietnamese => vec![Language::English],
        Language::German => vec![Language::English],
        Language::French => vec![Language::English],
        Language::Spanish => vec![Language::English],
        Language::English => vec![Language::Chinese],  // English fallback to Chinese for Asian topics
    }
}

// =============================================================================
// Wikipedia Search
// =============================================================================

/// Search Wikipedia for a query
///
/// Automatically detects the language of the query and searches
/// the appropriate Wikipedia edition first, then falls back to other languages.
pub async fn search_wikipedia(query: &str) -> Vec<SearchResult> {
    let detected_lang = detect_language(query);
    println!("[Crawler] Detected language: {} for query: {}", detected_lang.display_name(), query);

    let mut results = Vec::new();

    // Try primary language first
    if let Some(result) = fetch_wikipedia_page(query, detected_lang.wiki_code()).await {
        results.push(result);
        return results;
    }

    // Try fallback languages
    for fallback_lang in get_fallback_languages(detected_lang) {
        if let Some(result) = fetch_wikipedia_page(query, fallback_lang.wiki_code()).await {
            results.push(result);
            return results;
        }
    }

    // Last resort: try all major languages
    for lang_code in &["en", "zh", "ja", "ko", "ru", "ar", "th", "vi", "de", "fr", "es"] {
        if let Some(result) = fetch_wikipedia_page(query, lang_code).await {
            results.push(result);
            return results;
        }
    }

    results
}

/// Fetch a single Wikipedia page
async fn fetch_wikipedia_page(query: &str, lang: &str) -> Option<SearchResult> {
    let url = format!(
        "https://{}.wikipedia.org/api/rest_v1/page/summary/{}",
        lang,
        urlencoding::encode(query)
    );

    let client = reqwest::Client::builder()
        .user_agent("SeedIntelligence/0.1.0 (https://github.com/seed-intelligence; educational purpose)")
        .build()
        .ok()?;

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[Crawler] Wikipedia {} request failed: {}", lang, e);
            return None;
        }
    };

    if !response.status().is_success() {
        // Don't print 404 errors - they're expected for many queries
        if response.status() != 404 {
            eprintln!("[Crawler] Wikipedia {} returned status: {}", lang, response.status());
        }
        return None;
    }

    let data: WikiResponse = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[Crawler] Failed to parse Wikipedia {} response: {}", lang, e);
            return None;
        }
    };

    // Check if we have an extract
    let extract = data.extract?;

    // Skip disambiguation pages with minimal content
    if extract.len() < 20 && data.page_type.as_deref() == Some("disambiguation") {
        eprintln!("[Crawler] Skipping disambiguation page for: {}", query);
        return None;
    }

    let title = data.title.unwrap_or_else(|| query.to_string());
    let wiki_url = data.content_urls
        .and_then(|c| c.desktop)
        .and_then(|d| d.page)
        .unwrap_or_else(|| format!("https://{}.wikipedia.org/wiki/{}", lang, urlencoding::encode(query)));

    println!("[Crawler] Found article in {} Wikipedia: {}", lang, title);

    Some(SearchResult {
        title,
        snippet: extract,
        url: wiki_url,
        source: "wikipedia".to_string(),
        lang: lang.to_string(),
    })
}

// =============================================================================
// DuckDuckGo Instant Answer API
// =============================================================================

/// Search using DuckDuckGo Instant Answer API
///
/// This uses the DuckDuckGo Instant Answer API which is more automation-friendly
/// than the HTML search. It returns definitions and abstracts from Wikipedia and
/// other sources.
pub async fn search_duckduckgo(query: &str) -> Vec<SearchResult> {
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
        urlencoding::encode(query)
    );

    let client = match reqwest::Client::builder()
        .user_agent("SeedIntelligence/0.1.0 (https://github.com/seed-intelligence; educational purpose)")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Crawler] Failed to create HTTP client: {}", e);
            return vec![];
        }
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[Crawler] DuckDuckGo API request failed: {}", e);
            return vec![];
        }
    };

    let data: DDGResponse = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[Crawler] Failed to parse DuckDuckGo response: {}", e);
            return vec![];
        }
    };

    let mut results = Vec::new();

    // Add abstract if available
    if let Some(abstract_text) = data.abstract_text {
        if !abstract_text.is_empty() {
            let lang = detect_language(&abstract_text);
            results.push(SearchResult {
                title: data.heading.clone().unwrap_or_else(|| query.to_string()),
                snippet: abstract_text,
                url: data.abstract_url.clone().unwrap_or_default(),
                source: "duckduckgo".to_string(),
                lang: lang.wiki_code().to_string(),
            });
        }
    }

    // Add related topics
    for topic in data.related_topics.iter().take(3) {
        if let (Some(text), Some(url)) = (&topic.text, &topic.first_url) {
            if !text.is_empty() {
                let title = text.split(" - ").next().unwrap_or(query).to_string();
                let lang = detect_language(text);
                results.push(SearchResult {
                    title,
                    snippet: text.clone(),
                    url: url.clone(),
                    source: "duckduckgo".to_string(),
                    lang: lang.wiki_code().to_string(),
                });
            }
        }
    }

    results
}

#[derive(Debug, Clone, Deserialize)]
struct DDGResponse {
    #[serde(rename = "Abstract")]
    abstract_text: Option<String>,
    #[serde(rename = "AbstractURL")]
    abstract_url: Option<String>,
    #[serde(rename = "Heading")]
    heading: Option<String>,
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<RelatedTopic>,
}

#[derive(Debug, Clone, Deserialize)]
struct RelatedTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
}

// =============================================================================
// Combined Search
// =============================================================================

/// Combined search: tries multiple sources
pub async fn search(query: &str) -> Vec<SearchResult> {
    println!("[Crawler] Searching for: {}", query);

    // Try Wikipedia first (most reliable)
    let mut results = search_wikipedia(query).await;
    println!("[Crawler] Wikipedia results: {}", results.len());

    // If Wikipedia had no results, try DuckDuckGo Instant Answer API
    if results.is_empty() {
        let ddg_results = search_duckduckgo(query).await;
        println!("[Crawler] DuckDuckGo results: {}", ddg_results.len());
        results = ddg_results;
    }

    // Log if we got no results at all
    if results.is_empty() {
        eprintln!("[Crawler] No results found for query: '{}'", query);
        eprintln!("[Crawler] Suggestions:");
        eprintln!("[Crawler]   - Try the exact Wikipedia page title (e.g., 'Python_(programming_language)')");
        eprintln!("[Crawler]   - For Chinese, try '人工智能' or '机器学习'");
        eprintln!("[Crawler]   - For Japanese, try '人工知能'");
    }

    results
}

// =============================================================================
// URL Encoding Helper
// =============================================================================

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    encoded.push(c);
                }
                _ => {
                    for b in c.to_string().as_bytes() {
                        encoded.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        encoded
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_chinese() {
        assert_eq!(detect_language("人工智能"), Language::Chinese);
        assert_eq!(detect_language("机器学习"), Language::Chinese);
        assert_eq!(detect_language("深度學習"), Language::Chinese); // Traditional
    }

    #[test]
    fn test_detect_language_japanese() {
        // Japanese has Hiragana/Katakana mixed with Kanji
        assert_eq!(detect_language("人工知能は"), Language::Japanese);
        assert_eq!(detect_language("こんにちは"), Language::Japanese); // Hiragana only
        assert_eq!(detect_language("プログラミング"), Language::Japanese); // Katakana
    }

    #[test]
    fn test_detect_language_korean() {
        assert_eq!(detect_language("인공지능"), Language::Korean);
        assert_eq!(detect_language("머신러닝"), Language::Korean);
    }

    #[test]
    fn test_detect_language_russian() {
        assert_eq!(detect_language("Искусственный интеллект"), Language::Russian);
    }

    #[test]
    fn test_detect_language_arabic() {
        assert_eq!(detect_language("الذكاء الاصطناعي"), Language::Arabic);
    }

    #[test]
    fn test_detect_language_english() {
        assert_eq!(detect_language("Artificial Intelligence"), Language::English);
        assert_eq!(detect_language("machine learning"), Language::English);
    }

    #[test]
    fn test_language_wiki_code() {
        assert_eq!(Language::Chinese.wiki_code(), "zh");
        assert_eq!(Language::Japanese.wiki_code(), "ja");
        assert_eq!(Language::Korean.wiki_code(), "ko");
        assert_eq!(Language::English.wiki_code(), "en");
    }
}
