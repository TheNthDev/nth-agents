use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// GlobSearchTool for finding files matching a glob pattern within the workspace
#[derive(Debug, Clone)]
pub struct GlobSearchTool {
    config: HashMap<String, String>,
}

impl GlobSearchTool {
    pub fn new() -> Self {
        Self { config: HashMap::new() }
    }

    fn workspace_path(&self) -> std::path::PathBuf {
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        std::path::PathBuf::from(workspace)
    }
}

impl CodingTool for GlobSearchTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for GlobSearchTool {
    fn name(&self) -> &str { "glob_search" }

    fn description(&self) -> &str {
        "Search for files matching a glob pattern within the workspace (e.g., '**/*.rs', 'src/**/*.json')."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Glob pattern to match files (e.g., '**/*.rs')." }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let pattern = args.get("pattern").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pattern' parameter"))?;

        let workspace = self.workspace_path();
        let full_pattern = workspace.join(pattern);
        info!("[TOOL_CALL] GlobSearchTool searching: {:?}", full_pattern);

        let mut matches = Vec::new();
        match glob::glob(full_pattern.to_str().unwrap_or("")) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if let Ok(rel) = entry.strip_prefix(&workspace) {
                        matches.push(rel.display().to_string());
                    }
                }
            }
            Err(e) => {
                error!("[TOOL_ERROR] GlobSearchTool invalid pattern: {}", e);
                return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Invalid glob pattern: {}", e)) });
            }
        }

        let output = if matches.is_empty() {
            "No files matched the pattern.".to_string()
        } else {
            format!("Found {} file(s):\n{}", matches.len(), matches.join("\n"))
        };

        info!("[TOOL_SUCCESS] GlobSearchTool found {} matches", matches.len());
        Ok(tools::ToolResult { success: true, output, error: None })
    }
}
