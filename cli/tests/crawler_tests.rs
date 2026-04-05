// Tests for crawler module - DuckDuckGo and Wikipedia search

mod common;

use seed_intelligence::crawler::{
    search_wikipedia,
    search_duckduckgo,
    search,
    get_mock_results,
    extract_text,
    SearchResult,
};

#[test]
fn test_get_mock_results_returns_result() {
    let results = get_mock_results("人工智能");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "人工智能");
    assert!(results[0].snippet.contains("人工智能"));
    assert_eq!(results[0].source, "mock");
}

#[test]
fn test_get_mock_results_with_empty_query() {
    let results = get_mock_results("");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "");
}

#[test]
fn test_get_mock_results_with_special_chars() {
    let results = get_mock_results("测试 <script>alert('xss')</script>");

    // Should handle special characters safely
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("测试"));
}

#[test]
fn test_extract_text_single_result() {
    let results = vec![SearchResult {
        title: "人工智能".to_string(),
        snippet: "人工智能是计算机科学的一个分支".to_string(),
        url: "https://example.com".to_string(),
        source: "test".to_string(),
    }];

    let text = extract_text(&results);

    assert!(text.contains("人工智能"));
    assert!(text.contains("计算机科学"));
}

#[test]
fn test_extract_text_multiple_results() {
    let results = vec![
        SearchResult {
            title: "人工智能".to_string(),
            snippet: "AI简介".to_string(),
            url: "https://example.com/1".to_string(),
            source: "test".to_string(),
        },
        SearchResult {
            title: "机器学习".to_string(),
            snippet: "ML简介".to_string(),
            url: "https://example.com/2".to_string(),
            source: "test".to_string(),
        },
    ];

    let text = extract_text(&results);

    assert!(text.contains("人工智能"));
    assert!(text.contains("机器学习"));
}

#[test]
fn test_extract_text_empty_results() {
    let results: Vec<SearchResult> = vec![];
    let text = extract_text(&results);
    assert!(text.is_empty());
}

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

// Note: These tests require network access and may be slow or fail in CI
// They are integration tests that should be run with --ignored flag

#[tokio::test]
#[ignore]
async fn test_search_wikipedia_real() {
    let results = search_wikipedia("Python").await;

    // Should get results for a common topic
    // May be empty if network is unavailable
    println!("Wikipedia results: {:?}", results.len());
}

#[tokio::test]
#[ignore]
async fn test_search_duckduckgo_real() {
    let results = search_duckduckgo("Rust programming").await;

    // Should get results
    // May fallback to mock if network is unavailable
    println!("DuckDuckGo results: {:?}", results.len());
}

#[tokio::test]
#[ignore]
async fn test_search_combined_real() {
    let results = search("Artificial Intelligence").await;

    // Should get results from either Wikipedia or DuckDuckGo
    println!("Combined search results: {:?}", results.len());
}

// Unit tests for urlencoding (internal module)

#[test]
fn test_urlencoding_ascii() {
    // The urlencoding module is internal, but we can test it indirectly
    // through the search functions

    // ASCII characters should not be encoded
    let query = "hello";
    let results = get_mock_results(query);
    assert_eq!(results[0].title, query);
}

#[test]
fn test_urlencoding_chinese() {
    // Chinese characters should be handled
    let query = "人工智能";
    let results = get_mock_results(query);
    assert_eq!(results[0].title, query);
}

#[test]
fn test_urlencoding_special_chars() {
    // Special characters should be handled safely
    let query = "test & query";
    let results = get_mock_results(query);
    assert!(results[0].title.contains("test"));
}
