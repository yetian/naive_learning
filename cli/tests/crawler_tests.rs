// Tests for crawler module - DuckDuckGo and Wikipedia search

mod common;

use seed_intelligence::crawler::{
    search_wikipedia,
    search_duckduckgo,
    search,
    SearchResult,
    detect_language,
    Language,
};

#[test]
fn test_detect_language_chinese() {
    assert_eq!(detect_language("人工智能"), Language::Chinese);
    assert_eq!(detect_language("机器学习"), Language::Chinese);
}

#[test]
fn test_detect_language_japanese() {
    assert_eq!(detect_language("こんにちは"), Language::Japanese);
}

#[test]
fn test_detect_language_korean() {
    assert_eq!(detect_language("인공지능"), Language::Korean);
}

#[test]
fn test_detect_language_english() {
    assert_eq!(detect_language("Artificial Intelligence"), Language::English);
}

#[test]
fn test_search_result_serialization() {
    let result = SearchResult {
        title: "测试".to_string(),
        snippet: "摘要内容".to_string(),
        url: "https://test.com".to_string(),
        source: "wikipedia".to_string(),
        lang: "zh".to_string(),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("测试"));
    assert!(json.contains("wikipedia"));

    let parsed: SearchResult = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, result.title);
    assert_eq!(parsed.url, result.url);
}

#[test]
fn test_search_result_clone() {
    let result = SearchResult {
        title: "原始标题".to_string(),
        snippet: "原始摘要".to_string(),
        url: "https://original.com".to_string(),
        source: "test".to_string(),
        lang: "en".to_string(),
    };

    let cloned = result.clone();
    assert_eq!(cloned.title, result.title);
    assert_eq!(cloned.snippet, result.snippet);
}

#[test]
fn test_search_result_debug() {
    let result = SearchResult {
        title: "测试".to_string(),
        snippet: "内容".to_string(),
        url: "https://test.com".to_string(),
        source: "wikipedia".to_string(),
        lang: "zh".to_string(),
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("SearchResult"));
}

// Integration tests that require network access
// Run with: cargo test --test crawler_tests -- --ignored

#[tokio::test]
#[ignore]
async fn test_search_wikipedia_real() {
    let results = search_wikipedia("Python").await;
    println!("Wikipedia results for 'Python': {:?}", results.len());

    // Should get results for a common topic
    if !results.is_empty() {
        assert_eq!(results[0].source, "wikipedia");
        assert!(!results[0].snippet.is_empty());
    }
}

#[tokio::test]
#[ignore]
async fn test_search_duckduckgo_real() {
    let results = search_duckduckgo("Rust programming").await;
    println!("DuckDuckGo results for 'Rust programming': {:?}", results.len());

    // Should get results
    if !results.is_empty() {
        assert_eq!(results[0].source, "duckduckgo");
    }
}

#[tokio::test]
#[ignore]
async fn test_search_combined_real() {
    let results = search("Artificial Intelligence").await;
    println!("Combined search results: {:?}", results.len());

    // Should get results from either Wikipedia or DuckDuckGo
}

#[tokio::test]
#[ignore]
async fn test_search_chinese() {
    let results = search("人工智能").await;
    println!("Search results for '人工智能': {:?}", results.len());

    // Should handle Chinese queries and return Chinese Wikipedia
    if !results.is_empty() {
        println!("Language: {}", results[0].lang);
    }
}

#[tokio::test]
#[ignore]
async fn test_search_japanese() {
    let results = search("人工知能").await;
    println!("Search results for '人工知能': {:?}", results.len());

    // Should handle Japanese queries
    if !results.is_empty() {
        println!("Language: {}", results[0].lang);
    }
}

#[tokio::test]
#[ignore]
async fn test_search_korean() {
    let results = search("인공지능").await;
    println!("Search results for '인공지능': {:?}", results.len());

    // Should handle Korean queries
    if !results.is_empty() {
        println!("Language: {}", results[0].lang);
    }
}
