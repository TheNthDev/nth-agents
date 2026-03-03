use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// ImageInfoTool for reading image metadata and basic properties
#[derive(Debug, Clone)]
pub struct ImageInfoTool {
    config: HashMap<String, String>,
}

impl ImageInfoTool {
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

impl CodingTool for ImageInfoTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for ImageInfoTool {
    fn name(&self) -> &str { "image_info" }

    fn description(&self) -> &str {
        "Get information about an image file (dimensions, format, file size)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the image file (relative to workspace root)." }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let path = args.get("path").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let validated_path = self.validate_path(path);
        info!("[TOOL_CALL] ImageInfoTool reading: {:?}", validated_path);

        let metadata = match fs::metadata(&validated_path).await {
            Ok(m) => m,
            Err(e) => {
                error!("[TOOL_ERROR] ImageInfoTool failed to read {:?}: {}", validated_path, e);
                return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Failed to read file: {}", e)) });
            }
        };

        let file_size = metadata.len();
        let extension = validated_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        let format_info = match extension.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif" =>
                format!("Format: {}", extension.to_uppercase()),
            _ => format!("Format: {} (unknown image type)", extension),
        };

        let output = format!("{}\nFile size: {} bytes\nPath: {}", format_info, file_size, path);
        info!("[TOOL_SUCCESS] ImageInfoTool read info for {:?}", validated_path);
        Ok(tools::ToolResult { success: true, output, error: None })
    }
}
