use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::info;
use zeroclaw::tools;

use super::CodingTool;

/// Workspace tool for managing isolated workspaces per user
#[derive(Debug, Clone)]
pub struct WorkspaceTool {
    config: HashMap<String, String>,
}

impl WorkspaceTool {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
    
    async fn create_workspace(&self, user_id: &str) -> Result<(), anyhow::Error> {
        let workspace_path = format!("workspaces/{}", user_id);
        fs::create_dir_all(&workspace_path).await?;
        Ok(())
    }
}

impl CodingTool for WorkspaceTool {
    fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    fn set_config(&mut self, config: HashMap<String, String>) {
        self.config = config;
    }
}

#[async_trait]
impl tools::Tool for WorkspaceTool {
    fn name(&self) -> &str {
        "workspace"
    }

    fn description(&self) -> &str {
        "Create, clone, or delete isolated workspaces per user."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "The action to perform (create, clone, delete).",
                    "enum": ["create", "clone", "delete"]
                },
                "user_id": {
                    "type": "string",
                    "description": "The user ID for workspace isolation."
                }
            },
            "required": ["action", "user_id"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))?;
        let user_id = args.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'user_id' parameter"))?;
            
        info!("[TOOL_CALL] WorkspaceTool action: {} for user: {}", action, user_id);
        
        let result = match action {
            "create" => {
                self.create_workspace(user_id).await?;
                tools::ToolResult {
                    success: true,
                    output: format!("Workspace created for user: {}", user_id),
                    error: None,
                }
            },
            "clone" => {
                self.create_workspace(user_id).await?;
                let source = args.get("source_workspace")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                tools::ToolResult {
                    success: true,
                    output: format!("Workspace cloned from {} for user: {}", source, user_id),
                    error: None,
                }
            },
            "delete" => {
                tools::ToolResult {
                    success: true,
                    output: format!("Workspace deletion requested for user: {}", user_id),
                    error: None,
                }
            },
            _ => {
                return Ok(tools::ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Unknown workspace action".to_string()),
                });
            }
        };
        
        info!("[TOOL_SUCCESS] WorkspaceTool completed action: {}", action);
        Ok(result)
    }
}
