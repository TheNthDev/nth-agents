use async_trait::async_trait;
use std::collections::HashMap;
use tracing::info;
use zeroclaw::tools;

use super::CodingTool;

/// Git tool for common git operations with optional write mode for push/commit
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

    fn write_mode_enabled(&self) -> bool {
        self.config.get("git_write_mode").map(|v| v == "true").unwrap_or(false)
    }

    fn is_allowed_command(&self, command: &str) -> bool {
        let read_commands = ["status", "diff", "log", "branch", "checkout", "add"];
        let write_commands = ["commit", "push", "pull", "merge", "rebase", "stash", "clone"];
        if read_commands.contains(&command) {
            return true;
        }
        if write_commands.contains(&command) {
            return self.write_mode_enabled();
        }
        false
    }

    fn run_git(&self, workspace: &str, git_args: &[&str]) -> Result<String, tools::ToolResult> {
        let output = std::process::Command::new("git")
            .current_dir(workspace)
            .args(git_args)
            .output()
            .map_err(|e| tools::ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to run git: {}", e)),
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(tools::ToolResult {
                success: false,
                output: String::new(),
                error: Some(String::from_utf8_lossy(&output.stderr).into_owned()),
            })
        }
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
        "Execute git commands in the agent's workspace. Read-only commands (status, diff, log, branch, checkout, add) are always available. Write commands (commit, push, pull, merge, rebase, stash) require git_write_mode to be enabled in the agent configuration."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The git subcommand to run: status, diff, log, branch, checkout, add, commit, push, pull, merge, rebase, stash."
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Additional arguments for the git command. E.g. for commit: [\"-m\", \"my message\"]. For push: [\"origin\", \"main\"]. For checkout: [\"-b\", \"new-branch\"]."
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

        let extra_args: Vec<String> = args.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        if !self.is_allowed_command(command) {
            let write_commands = ["commit", "push", "pull", "merge", "rebase", "stash"];
            if write_commands.contains(&command) {
                return Ok(tools::ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Command '{}' requires write mode. Enable it by setting 'git_write_mode: true' in your agent configuration tools list as 'git_write'.",
                        command
                    )),
                });
            }
            return Ok(tools::ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Command '{}' is not supported.", command)),
            });
        }

        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        info!("[TOOL_CALL] GitTool executing in {}: git {} {:?}", workspace, command, extra_args);

        let mut git_args = vec![command];
        let extra_refs: Vec<&str> = extra_args.iter().map(|s| s.as_str()).collect();
        git_args.extend_from_slice(&extra_refs);

        let result = self.run_git(workspace, &git_args);

        match result {
            Ok(output) => {
                info!("[TOOL_SUCCESS] GitTool executed: git {} {:?}", command, extra_args);
                Ok(tools::ToolResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            Err(tool_result) => Ok(tool_result),
        }
    }
}
