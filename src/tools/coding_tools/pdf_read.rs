use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// PdfReadTool for extracting text content from PDF files in the workspace
#[derive(Debug, Clone)]
pub struct PdfReadTool {
    config: HashMap<String, String>,
}

impl PdfReadTool {
    pub fn new() -> Self {
        Self { config: HashMap::new() }
    }

    fn validate_path(&self, path: &str) -> std::path::PathBuf {
        let workspace = self.config.get("workspace").map(|s| s.as_str()).unwrap_or("workspaces/default");
        let base = std::path::PathBuf::from(workspace);
        let clean_path = path.trim_start_matches('/').replace("../", "").replace("..", "");
        let target = base.join(clean_path);
        if target.starts_with(&base) { target } else { base }
    }
}

impl CodingTool for PdfReadTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for PdfReadTool {
    fn name(&self) -> &str { "pdf_read" }

    fn description(&self) -> &str {
        "Extract text content from a PDF file in the workspace."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the PDF file (relative to workspace root)." }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let path = args.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let validated_path = self.validate_path(path);
        info!("[TOOL_CALL] PdfReadTool reading: {:?}", validated_path);

        // Read the file bytes
        let bytes = match fs::read(&validated_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("[TOOL_ERROR] PdfReadTool failed to read {:?}: {}", validated_path, e);
                return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Failed to read file: {}", e)) });
            }
        };

        // Basic text extraction: try to read readable text from PDF bytes
        // For full PDF parsing, consider adding the pdf_extract crate
        let text = String::from_utf8_lossy(&bytes);
        // Extract readable ASCII segments from the PDF
        let readable: String = text.split(|c: char| c.is_control() && c != '\n' && c != '\t')
            .filter(|s| s.len() > 3)
            .collect::<Vec<_>>()
            .join(" ");

        if readable.trim().is_empty() {
            Ok(tools::ToolResult { success: false, output: String::new(), error: Some("Could not extract readable text from PDF. The file may be image-based.".into()) })
        } else {
            info!("[TOOL_SUCCESS] PdfReadTool extracted {} chars from {:?}", readable.len(), validated_path);
            Ok(tools::ToolResult { success: true, output: readable, error: None })
        }
    }
}
