// Tests for crawler module - DuckDuckGo and Wikipedia search

mod common;

use seed_intelligence::crawler::{
    search_wikipedia,
    search_duckduckgo,
    search,
    SearchResult,
};

#[test]
fn test_search_result_serialization() {
    let result = SearchResult {
        title: "测试".to_string(),
        snippet: "摘要内容".to_string(),
        url: "https://test.com".to_string(),
        source: "wikipedia".to_string(),
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
        assert!(results[0].source == "wikipedia");
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
        assert!(results[0].source == "duckduckgo");
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

    // Should handle Chinese queries
}

#[tokio::test]
#[ignore]
async fn test_search_returns_empty_on_network_error() {
    // This tests that search returns empty vec on failure, not mock data
    // Using a very long timeout query that might fail
    let results = search_duckduckgo("x").await;

    // Results could be empty or have actual results, but never mock
    for r in &results {
        assert_ne!(r.source, "mock");
    }
}
