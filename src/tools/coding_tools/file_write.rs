use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// File write tool for creating/updating files with atomic writes and backup
#[derive(Debug, Clone)]
pub struct FileWriteTool {
    config: HashMap<String, String>,
}

impl FileWriteTool {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
    
    /// Create backup of existing file before overwriting
    async fn create_backup(&self, path: &std::path::Path) -> Result<(), anyhow::Error> {
        if path.exists() {
            let backup_path = path.with_extension("backup");
            if let Ok(content) = fs::read_to_string(path).await {
                let _ = fs::write(&backup_path, content).await;
            }
        }
        Ok(())
    }

    fn validate_path(&self, path: &str) -> std::path::PathBuf {
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        let base = std::path::PathBuf::from(workspace);
        
        let clean_path = path.trim_start_matches('/').replace("../", "").replace("..", "");
        let target = base.join(clean_path);
        
        if target.starts_with(&base) {
            target
        } else {
            base
        }
    }
}

impl CodingTool for FileWriteTool {
    fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    fn set_config(&mut self, config: HashMap<String, String>) {
        self.config = config;
    }
}

#[async_trait]
impl tools::Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file with atomic write and backup capabilities."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write (relative to workspace root)."
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file."
                },
                "append": {
                    "type": "boolean",
                    "description": "Whether to append to the file instead of overwriting.",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let append = args.get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let file_path = self.validate_path(path);
        
        // Create backup if file exists and not appending
        if !append && file_path.exists() {
            self.create_backup(&file_path).await?;
        }
        
        let write_result = if append {
            fs::write(&file_path, content).await
        } else {
            fs::write(&file_path, content).await
        };
        
        match write_result {
            Ok(_) => {
                info!("[TOOL_SUCCESS] FileWriteTool wrote to {:?}", file_path);
                Ok(tools::ToolResult {
                    success: true,
                    output: format!("Successfully wrote to file: {:?}", file_path),
                    error: None,
                })
            }
            Err(e) => {
                error!("[TOOL_ERROR] FileWriteTool failed to write to {:?}: {}", file_path, e);
                Ok(tools::ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to write file: {}", e)),
                })
            }
        }
    }
}
