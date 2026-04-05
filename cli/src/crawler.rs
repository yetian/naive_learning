// Crawler - DuckDuckGo and Wikipedia search

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
struct WikiResponse {
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

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub source: String,
}

/// Search Wikipedia for a query
pub async fn search_wikipedia(query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // Try English Wikipedia first
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        urlencoding::encode(query)
    );

    if let Ok(response) = reqwest::get(&url).await {
        if let Ok(data) = response.json::<WikiResponse>().await {
            if let Some(extract) = data.extract {
                results.push(SearchResult {
                    title: data.title.unwrap_or_else(|| query.to_string()),
                    snippet: extract,
                    url: data.content_urls
                        .and_then(|c| c.desktop)
                        .and_then(|d| d.page)
                        .unwrap_or_else(|| format!("https://en.wikipedia.org/wiki/{}", urlencoding::encode(query))),
                    source: "wikipedia".to_string(),
                });
            }
        }
    }

    // Try Chinese Wikipedia if no results
    if results.is_empty() {
        let url = format!(
            "https://zh.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding::encode(query)
        );

        if let Ok(response) = reqwest::get(&url).await {
            if let Ok(data) = response.json::<WikiResponse>().await {
                if let Some(extract) = data.extract {
                    results.push(SearchResult {
                        title: data.title.unwrap_or_else(|| query.to_string()),
                        snippet: extract,
                        url: data.content_urls
                            .and_then(|c| c.desktop)
                            .and_then(|d| d.page)
                            .unwrap_or_else(|| format!("https://zh.wikipedia.org/wiki/{}", urlencoding::encode(query))),
                        source: "wikipedia".to_string(),
                    });
                }
            }
        }
    }

    results
}

/// Search using DuckDuckGo HTML
pub async fn search_duckduckgo(query: &str) -> Vec<SearchResult> {
    let client = reqwest::Client::new();

    let response = match client
        .get("https://html.duckduckgo.com/html/")
        .query(&[("q", query)])
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return get_mock_results(query),
    };

    let text = match response.text().await {
        Ok(t) => t,
        Err(_) => return get_mock_results(query),
    };

    let mut results = Vec::new();

    // Simple HTML parsing using regex
    let re = regex::Regex::new(r#"<a class="result__a"[^>]*href="([^"]*)"[^>]*>([^<]*)</a>.*?<a class="result__snippet"[^>]*>([^<]*)</a>"#).unwrap();

    for cap in re.captures_iter(&text) {
        if results.len() >= 5 {
            break;
        }

        let url = cap.get(1).map(|m| m.as_str()).unwrap_or("#");
        let title = cap.get(2).map(|m| m.as_str()).unwrap_or(query);
        let snippet = cap.get(3).map(|m| m.as_str()).unwrap_or("");

        results.push(SearchResult {
            title: title.to_string(),
            snippet: snippet.trim().to_string(),
            url: url.to_string(),
            source: "duckduckgo".to_string(),
        });
    }

    if results.is_empty() {
        get_mock_results(query)
    } else {
        results
    }
}

/// Combined search: Wikipedia first, then DuckDuckGo
pub async fn search(query: &str) -> Vec<SearchResult> {
    println!("[Crawler] Searching for: {}", query);

    // Try Wikipedia first
    let results = search_wikipedia(query).await;
    println!("[Crawler] Wikipedia results: {}", results.len());

    // Fallback to DuckDuckGo if no results
    if results.is_empty() {
        let ddg_results = search_duckduckgo(query).await;
        println!("[Crawler] DuckDuckGo results: {}", ddg_results.len());
        ddg_results
    } else {
        results
    }
}

/// Get mock results for fallback
pub fn get_mock_results(query: &str) -> Vec<SearchResult> {
    vec![SearchResult {
        title: query.to_string(),
        snippet: format!("{} 是一个概念。", query),
        url: "#".to_string(),
        source: "mock".to_string(),
    }]
}

/// Extract text from search results
pub fn extract_text(results: &[SearchResult]) -> String {
    results
        .iter()
        .map(|r| format!("{}: {}", r.title, r.snippet))
        .collect::<Vec<_>>()
        .join(" ")
}

// urlencoding crate re-export for convenience
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