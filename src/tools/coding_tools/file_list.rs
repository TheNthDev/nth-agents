use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// File list tool for listing directory contents with filtering
#[derive(Debug, Clone)]
pub struct FileListTool {
    config: HashMap<String, String>,
}

impl FileListTool {
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
        
        if target.starts_with(&base) {
            target
        } else {
            base
        }
    }
}

impl CodingTool for FileListTool {
    fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    fn set_config(&mut self, config: HashMap<String, String>) {
        self.config = config;
    }
}

#[async_trait]
impl tools::Tool for FileListTool {
    fn name(&self) -> &str {
        "file_list"
    }

    fn description(&self) -> &str {
        "List directory contents with filtering capabilities."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to list (relative to workspace root)."
                },
                "filter": {
                    "type": "string",
                    "description": "Filter files by extension or pattern (e.g., '.rs', '*.md')."
                }
            },
            "required": []
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
            
        let dir_path = self.validate_path(path);
        
        info!("[TOOL_CALL] FileListTool listing: {:?}", dir_path);
        
        let mut entries = Vec::new();
        let mut dir = match fs::read_dir(&dir_path).await {
            Ok(d) => d,
            Err(e) => {
                error!("[TOOL_ERROR] FileListTool failed to list {:?}: {}", dir_path, e);
                return Ok(tools::ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to list directory: {}", e)),
                });
            }
        };
        
        while let Some(entry) = dir.next_entry().await? {
            let entry_path = entry.path();
            if let Ok(meta) = entry.metadata().await {
                let file_type = if meta.is_dir() { "directory" } else { "file" };
                let entry_str = entry_path.to_string_lossy().to_string();
                
                // Apply filter if specified
                let should_include = match args.get("filter") {
                    Some(filter) => {
                        if let Some(filter_str) = filter.as_str() {
                            if filter_str.starts_with('.') {
                                entry_str.ends_with(filter_str)
                            } else {
                                entry_str.contains(filter_str)
                            }
                        } else {
                            true
                        }
                    }
                    None => true,
                };
                
                if should_include {
                    entries.push(format!("{} ({})", entry_str, file_type));
                }
            }
        }
        
        let output = if entries.is_empty() {
            "No files found.".to_string()
        } else {
            format!("Files in directory:\n{}", entries.join("\n"))
        };
        
        info!("[TOOL_SUCCESS] FileListTool returned {} entries", entries.len());
        Ok(tools::ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}
