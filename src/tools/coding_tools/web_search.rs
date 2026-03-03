use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, warn};
use zeroclaw::tools;

use super::CodingTool;

/// WebSearchTool for performing web searches
#[derive(Debug, Clone)]
pub struct WebSearchTool {
    config: HashMap<String, String>,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self { config: HashMap::new() }
    }
}

impl CodingTool for WebSearchTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for WebSearchTool {
    fn name(&self) -> &str { "web_search" }

    fn description(&self) -> &str {
        "Search the web using a query string. Returns search results with titles, URLs, and snippets."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "The search query." },
                "max_results": { "type": "integer", "description": "Maximum number of results (default: 5, max: 10)." }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let query = args.get("query").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;
        let max_results = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(5).min(10) as usize;

        info!("[TOOL_CALL] WebSearchTool searching: '{}'", query);

        // Check for Brave API key in config or environment
        let brave_key = self.config.get("brave_api_key")
            .cloned()
            .or_else(|| std::env::var("BRAVE_API_KEY").ok());

        if let Some(api_key) = brave_key {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()?;
            let resp = client.get("https://api.search.brave.com/res/v1/web/search")
                .header("X-Subscription-Token", &api_key)
                .query(&[("q", query), ("count", &max_results.to_string())])
                .send().await;

            match resp {
                Ok(response) if response.status().is_success() => {
                    let body: serde_json::Value = response.json().await.unwrap_or_default();
                    let mut results = Vec::new();
                    if let Some(web) = body.get("web").and_then(|w| w.get("results")).and_then(|r| r.as_array()) {
                        for item in web.iter().take(max_results) {
                            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("(no title)");
                            let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            let desc = item.get("description").and_then(|v| v.as_str()).unwrap_or("");
                            results.push(format!("• {} \n  {}\n  {}", title, url, desc));
                        }
                    }
                    let output = if results.is_empty() {
                        format!("No results found for '{}'.", query)
                    } else {
                        format!("Search results for '{}':\n\n{}", query, results.join("\n\n"))
                    };
                    info!("[TOOL_SUCCESS] WebSearchTool returned {} results", results.len());
                    return Ok(tools::ToolResult { success: true, output, error: None });
                }
                Ok(response) => {
                    warn!("[TOOL_WARN] WebSearchTool Brave API returned {}", response.status());
                }
                Err(e) => {
                    warn!("[TOOL_WARN] WebSearchTool Brave API failed: {}", e);
                }
            }
        }

        // Fallback: inform user no search provider is configured
        Ok(tools::ToolResult {
            success: false,
            output: String::new(),
            error: Some("No web search provider configured. Set BRAVE_API_KEY environment variable for Brave Search.".into()),
        })
    }
}
