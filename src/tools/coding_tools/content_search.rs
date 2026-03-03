use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::info;
use zeroclaw::tools;

use super::CodingTool;

/// ContentSearchTool for searching text content within workspace files (grep-like)
#[derive(Debug, Clone)]
pub struct ContentSearchTool {
    config: HashMap<String, String>,
}

impl ContentSearchTool {
    pub fn new() -> Self {
        Self { config: HashMap::new() }
    }

    fn workspace_path(&self) -> std::path::PathBuf {
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        std::path::PathBuf::from(workspace)
    }
}

impl CodingTool for ContentSearchTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for ContentSearchTool {
    fn name(&self) -> &str { "content_search" }

    fn description(&self) -> &str {
        "Search for text content within workspace files. Returns matching lines with file paths and line numbers."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "The text to search for." },
                "file_pattern": { "type": "string", "description": "Optional glob pattern to limit search (e.g., '**/*.rs'). Defaults to '**/*'." }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let query = args.get("query").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;
        let file_pattern = args.get("file_pattern").and_then(|v| v.as_str()).unwrap_or("**/*");

        let workspace = self.workspace_path();
        let full_pattern = workspace.join(file_pattern);
        info!("[TOOL_CALL] ContentSearchTool searching for '{}' in {:?}", query, full_pattern);

        let mut results = Vec::new();
        let max_results = 100;

        if let Ok(paths) = glob::glob(full_pattern.to_str().unwrap_or("")) {
            for entry in paths.flatten() {
                if !entry.is_file() { continue; }
                if let Ok(content) = fs::read_to_string(&entry).await {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains(query) {
                            let rel = entry.strip_prefix(&workspace).unwrap_or(&entry);
                            results.push(format!("{}:{}: {}", rel.display(), line_num + 1, line.trim()));
                            if results.len() >= max_results { break; }
                        }
                    }
                }
                if results.len() >= max_results { break; }
            }
        }

        let output = if results.is_empty() {
            format!("No matches found for '{}'.", query)
        } else {
            format!("Found {} match(es):\n{}", results.len(), results.join("\n"))
        };

        info!("[TOOL_SUCCESS] ContentSearchTool found {} matches", results.len());
        Ok(tools::ToolResult { success: true, output, error: None })
    }
}
