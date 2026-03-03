use async_trait::async_trait;
use std::collections::HashMap;
use tracing::info;
use zeroclaw::tools;

use super::CodingTool;

/// Terminal tool for command execution with allowlist/denylist
#[derive(Debug, Clone)]
pub struct TerminalTool {
    config: HashMap<String, String>,
    allowed_commands: Vec<String>,
    denied_commands: Vec<String>,
}

impl TerminalTool {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            allowed_commands: vec!["ls".to_string(), "pwd".to_string(), "echo".to_string(), "cat".to_string(), "grep".to_string(), "find".to_string()],
            denied_commands: vec!["rm".to_string(), "sudo".to_string(), "kill".to_string(), "shutdown".to_string()],
        }
    }
    
    fn is_command_allowed(&self, cmd: &str) -> bool {
        if self.denied_commands.contains(&cmd.to_string()) {
            return false;
        }
        self.allowed_commands.contains(&cmd.to_string())
    }
}

impl CodingTool for TerminalTool {
    fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    fn set_config(&mut self, config: HashMap<String, String>) {
        self.config = config;
    }
}

#[async_trait]
impl tools::Tool for TerminalTool {
    fn name(&self) -> &str {
        "terminal"
    }

    fn description(&self) -> &str {
        "Execute terminal commands with security restrictions."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute."
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;
            
        if !self.is_command_allowed(command) {
            return Ok(tools::ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Command '{}' is not allowed for security reasons", command)),
            });
        }
        
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        info!("[TOOL_CALL] TerminalTool executing in {}: {}", workspace, command);
        
        let output = match std::process::Command::new("sh")
            .current_dir(workspace)
            .arg("-c")
            .arg(command)
            .output() {
                Ok(output) => {
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).into_owned()
                    } else {
                        return Ok(tools::ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Command failed: {}", 
                                String::from_utf8_lossy(&output.stderr))),
                        });
                    }
                }
                Err(e) => {
                    return Ok(tools::ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to execute command: {}", e)),
                    });
                }
            };
        
        info!("[TOOL_SUCCESS] TerminalTool executed command: {}", command);
        Ok(tools::ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}
