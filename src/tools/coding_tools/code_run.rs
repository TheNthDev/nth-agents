use async_trait::async_trait;
use std::collections::HashMap;
use tracing::info;
use zeroclaw::tools;

use super::CodingTool;

/// CodeRunTool for executing code in isolated environments (simulated)
#[derive(Debug, Clone)]
pub struct CodeRunTool {
    config: HashMap<String, String>,
}

impl CodeRunTool {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
}

impl CodingTool for CodeRunTool {
    fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    fn set_config(&mut self, config: HashMap<String, String>) {
        self.config = config;
    }
}

#[async_trait]
impl tools::Tool for CodeRunTool {
    fn name(&self) -> &str {
        "code_run"
    }

    fn description(&self) -> &str {
        "Execute code in isolated environments (simulated)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "description": "The programming language (python, nodejs, rust).",
                    "enum": ["python", "nodejs", "rust"]
                },
                "code": {
                    "type": "string",
                    "description": "The code to execute."
                }
            },
            "required": ["language", "code"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let language = args.get("language")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'language' parameter"))?;
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'code' parameter"))?;
            
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        info!("[TOOL_CALL] CodeRunTool executing {} code in {}", language, workspace);
        
        let result = match language {
            "python" => format!("Python code executed in {} (simulated).\nCode:\n{}", workspace, code),
            "nodejs" => format!("Node.js code executed in {} (simulated).\nCode:\n{}", workspace, code),
            "rust" => format!("Rust code executed in {} (simulated).\nCode:\n{}", workspace, code),
            _ => {
                return Ok(tools::ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Unsupported language".to_string()),
                });
            }
        };
        
        info!("[TOOL_SUCCESS] CodeRunTool completed execution");
        Ok(tools::ToolResult {
            success: true,
            output: result,
            error: None,
        })
    }
}
