// Crawler - Wikipedia search and web content retrieval
//
// This module provides web search capabilities for learning.
// Due to bot protection on many search engines, we primarily rely on
// Wikipedia's API which is more automation-friendly.

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
}

// =============================================================================
// Wikipedia Search
// =============================================================================

/// Search Wikipedia for a query
///
/// Tries English Wikipedia first, then Chinese Wikipedia.
/// Handles disambiguation pages by returning them with minimal content.
pub async fn search_wikipedia(query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // Try English Wikipedia first
    if let Some(result) = fetch_wikipedia_page(query, "en").await {
        results.push(result);
    }

    // Try Chinese Wikipedia if no English results
    if results.is_empty() {
        if let Some(result) = fetch_wikipedia_page(query, "zh").await {
            results.push(result);
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
        eprintln!("[Crawler] Wikipedia {} returned status: {}", lang, response.status());
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

    Some(SearchResult {
        title,
        snippet: extract,
        url: wiki_url,
        source: "wikipedia".to_string(),
    })
}

// =============================================================================
// Alternative: DuckDuckGo Instant Answer API
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
            results.push(SearchResult {
                title: data.heading.clone().unwrap_or_else(|| query.to_string()),
                snippet: abstract_text,
                url: data.abstract_url.clone().unwrap_or_default(),
                source: "duckduckgo".to_string(),
            });
        }
    }

    // Add related topics
    for topic in data.related_topics.iter().take(3) {
        if let (Some(text), Some(url)) = (&topic.text, &topic.first_url) {
            if !text.is_empty() {
                let title = text.split(" - ").next().unwrap_or(query).to_string();
                results.push(SearchResult {
                    title,
                    snippet: text.clone(),
                    url: url.clone(),
                    source: "duckduckgo".to_string(),
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
        eprintln!("[Crawler] Try a more specific term like 'Python_(programming_language)' for Wikipedia");
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
