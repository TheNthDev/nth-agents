use async_trait::async_trait;
use std::collections::HashMap;
use tracing::info;
use zeroclaw::tools;

use super::CodingTool;

/// Git tool for common git operations with safety checks
#[derive(Debug, Clone)]
pub struct GitTool {
    config: HashMap<String, String>,
}

impl GitTool {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
    
    fn is_allowed_command(&self, command: &str) -> bool {
        let allowed_commands = ["status", "diff", "log", "commit", "branch", "checkout"];
        allowed_commands.contains(&command)
    }
}

impl CodingTool for GitTool {
    fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    fn set_config(&mut self, config: HashMap<String, String>) {
        self.config = config;
    }
}

#[async_trait]
impl tools::Tool for GitTool {
    fn name(&self) -> &str {
        "git_tool"
    }

    fn description(&self) -> &str {
        "Execute git commands with safety checks."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The git command to execute (status, diff, log, commit, branch)."
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;
            
        if !self.is_allowed_command(command) {
            return Ok(tools::ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Command '{}' is not allowed for security reasons", command)),
            });
        }
        
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        info!("[TOOL_CALL] GitTool executing in {}: git {}", workspace, command);
        
        let output = match command {
            "status" => {
                let output = std::process::Command::new("git")
                    .current_dir(workspace)
                    .arg("status")
                    .output()?;
            
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).into_owned()
                } else {
                    return Ok(tools::ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Git status failed: {}", 
                            String::from_utf8_lossy(&output.stderr))),
                    });
                }
            }
            "diff" => {
                let output = std::process::Command::new("git")
                    .current_dir(workspace)
                    .arg("diff")
                    .output()?;
            
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).into_owned()
                } else {
                    return Ok(tools::ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Git diff failed: {}", 
                            String::from_utf8_lossy(&output.stderr))),
                    });
                }
            }
            "log" => {
                let output = std::process::Command::new("git")
                    .current_dir(workspace)
                    .arg("log")
                    .arg("--oneline")
                    .output()?;
            
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).into_owned()
                } else {
                    return Ok(tools::ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Git log failed: {}", 
                            String::from_utf8_lossy(&output.stderr))),
                    });
                }
            }
            "commit" => {
                "Git commit tool executed (would commit changes in a real system)".to_string()
            }
            "branch" => {
                let output = std::process::Command::new("git")
                    .current_dir(workspace)
                    .arg("branch")
                    .output()?;
            
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).into_owned()
                } else {
                    return Ok(tools::ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Git branch failed: {}", 
                            String::from_utf8_lossy(&output.stderr))),
                    });
                }
            }
            "checkout" => {
                "Git checkout tool executed (would switch branches in a real system)".to_string()
            }
            _ => {
                return Ok(tools::ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Unknown git command".to_string()),
                });
            }
        };
        
        info!("[TOOL_SUCCESS] GitTool executed command: {}", command);
        Ok(tools::ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}
