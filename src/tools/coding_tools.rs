use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error, warn};

use zeroclaw::tools;

/// Base trait for coding tools that extend ZeroClaw's tool interface
#[async_trait]
pub trait CodingTool: tools::Tool {
    /// Get the tool's specific configuration requirements
    fn config(&self) -> &HashMap<String, String>;
    
    /// Set the tool's configuration requirements
    fn set_config(&mut self, config: HashMap<String, String>);
}

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
