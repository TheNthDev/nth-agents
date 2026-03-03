use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// WebFetchTool for fetching web page content
#[derive(Debug, Clone)]
pub struct WebFetchTool {
    config: HashMap<String, String>,
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self { config: HashMap::new() }
    }
}

impl CodingTool for WebFetchTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for WebFetchTool {
    fn name(&self) -> &str { "web_fetch" }

    fn description(&self) -> &str {
        "Fetch the text content of a web page by URL. Returns the page body as plain text (HTML tags stripped)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to fetch." },
                "max_length": { "type": "integer", "description": "Maximum response length in characters (default: 50000)." }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let url = args.get("url").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
        let max_length = args.get("max_length").and_then(|v| v.as_u64()).unwrap_or(50000) as usize;

        info!("[TOOL_CALL] WebFetchTool fetching: {}", url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        match client.get(url).send().await {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("HTTP {}", status)) });
                }
                let body = response.text().await.unwrap_or_default();
                // Simple HTML tag stripping
                let text = body
                    .replace("<script", "\n<script").replace("</script>", "</script>\n")
                    .split('<').flat_map(|s| s.split('>').skip(1)).collect::<Vec<_>>().join(" ")
                    .split_whitespace().collect::<Vec<_>>().join(" ");
                let truncated = if text.len() > max_length { &text[..max_length] } else { &text };
                info!("[TOOL_SUCCESS] WebFetchTool fetched {} chars from {}", truncated.len(), url);
                Ok(tools::ToolResult { success: true, output: truncated.to_string(), error: None })
            }
            Err(e) => {
                error!("[TOOL_ERROR] WebFetchTool failed: {}", e);
                Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Fetch failed: {}", e)) })
            }
        }
    }
}
