use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// FileEditTool for search-and-replace editing within workspace files
#[derive(Debug, Clone)]
pub struct FileEditTool {
    config: HashMap<String, String>,
}

impl FileEditTool {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }

    fn validate_path(&self, path: &str) -> std::path::PathBuf {
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        let base = std::path::PathBuf::from(workspace);
        let clean_path = path.trim_start_matches('/').replace("../", "").replace("..", "");
        let target = base.join(clean_path);
        if target.starts_with(&base) { target } else { base }
    }
}

impl CodingTool for FileEditTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for FileEditTool {
    fn name(&self) -> &str { "file_edit" }

    fn description(&self) -> &str {
        "Edit a file by replacing an exact string match with new content. The old_string must appear exactly once in the file."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file (relative to workspace root)." },
                "old_string": { "type": "string", "description": "The exact text to find (must appear exactly once)." },
                "new_string": { "type": "string", "description": "The replacement text (empty to delete)." }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let path = args.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let old_string = args.get("old_string").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'old_string' parameter"))?;
        let new_string = args.get("new_string").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'new_string' parameter"))?;

        if old_string.is_empty() {
            return Ok(tools::ToolResult { success: false, output: String::new(), error: Some("old_string must not be empty".into()) });
        }

        let validated_path = self.validate_path(path);
        info!("[TOOL_CALL] FileEditTool editing: {:?}", validated_path);

        let content = match fs::read_to_string(&validated_path).await {
            Ok(c) => c,
            Err(e) => {
                error!("[TOOL_ERROR] FileEditTool failed to read {:?}: {}", validated_path, e);
                return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Failed to read file: {}", e)) });
            }
        };

        let count = content.matches(old_string).count();
        if count == 0 {
            return Ok(tools::ToolResult { success: false, output: String::new(), error: Some("old_string not found in file".into()) });
        }
        if count > 1 {
            return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("old_string found {} times (must be exactly once)", count)) });
        }

        let new_content = content.replacen(old_string, new_string, 1);
        if let Err(e) = fs::write(&validated_path, &new_content).await {
            error!("[TOOL_ERROR] FileEditTool failed to write {:?}: {}", validated_path, e);
            return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Failed to write file: {}", e)) });
        }

        info!("[TOOL_SUCCESS] FileEditTool edited {:?}", validated_path);
        Ok(tools::ToolResult { success: true, output: format!("Successfully edited {}", path), error: None })
    }
}
