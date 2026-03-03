use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// File read tool for reading file contents with path validation to prevent directory traversal
#[derive(Debug, Clone)]
pub struct FileReadTool {
    config: HashMap<String, String>,
}

impl FileReadTool {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
    
    /// Validate that the file path is within allowed boundaries to prevent directory traversal
    fn validate_path(&self, path: &str) -> std::path::PathBuf {
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        let base = std::path::PathBuf::from(workspace);
        
        // Clean up the input path to prevent traversal
        let clean_path = path.trim_start_matches('/').replace("../", "").replace("..", "");
        let target = base.join(clean_path);
        
        // Ensure path doesn't escape workspace
        if target.starts_with(&base) {
            target
        } else {
            base
        }
    }
}

impl CodingTool for FileReadTool {
    fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    fn set_config(&mut self, config: HashMap<String, String>) {
        self.config = config;
    }
}

#[async_trait]
impl tools::Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read file contents with path validation to prevent directory traversal attacks."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read (relative to workspace root)."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
            
        let validated_path = self.validate_path(path);
        
        info!("[TOOL_CALL] FileReadTool reading: {:?}", validated_path);
        
        match fs::read_to_string(&validated_path).await {
            Ok(content) => {
                info!("[TOOL_SUCCESS] FileReadTool read {} bytes from {:?}", content.len(), validated_path);
                Ok(tools::ToolResult {
                    success: true,
                    output: content,
                    error: None,
                })
            }
            Err(e) => {
                error!("[TOOL_ERROR] FileReadTool failed to read {:?}: {}", validated_path, e);
                Ok(tools::ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to read file: {}", e)),
                })
            }
        }
    }
}
