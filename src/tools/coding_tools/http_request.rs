use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, error};
use zeroclaw::tools;

use super::CodingTool;

/// HttpRequestTool for making arbitrary HTTP requests
#[derive(Debug, Clone)]
pub struct HttpRequestTool {
    config: HashMap<String, String>,
}

impl HttpRequestTool {
    pub fn new() -> Self {
        Self { config: HashMap::new() }
    }
}

impl CodingTool for HttpRequestTool {
    fn config(&self) -> &HashMap<String, String> { &self.config }
    fn set_config(&mut self, config: HashMap<String, String>) { self.config = config; }
}

#[async_trait]
impl tools::Tool for HttpRequestTool {
    fn name(&self) -> &str { "http_request" }

    fn description(&self) -> &str {
        "Make an HTTP request (GET, POST, PUT, DELETE, PATCH) and return the response."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "method": { "type": "string", "description": "HTTP method (GET, POST, PUT, DELETE, PATCH).", "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"] },
                "url": { "type": "string", "description": "The URL to request." },
                "headers": { "type": "object", "description": "Optional headers as key-value pairs." },
                "body": { "type": "string", "description": "Optional request body." }
            },
            "required": ["method", "url"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult, anyhow::Error> {
        let method = args.get("method").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'method' parameter"))?;
        let url = args.get("url").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;

        info!("[TOOL_CALL] HttpRequestTool {} {}", method, url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => return Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Unsupported method: {}", method)) }),
        };

        if let Some(headers) = args.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val) = value.as_str() {
                    request = request.header(key.as_str(), val);
                }
            }
        }

        if let Some(body) = args.get("body").and_then(|v| v.as_str()) {
            request = request.body(body.to_string());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let headers: Vec<String> = response.headers().iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("(binary)")))
                    .collect();
                let body = response.text().await.unwrap_or_default();
                let truncated = if body.len() > 50000 { &body[..50000] } else { &body };

                let output = format!("Status: {}\nHeaders:\n{}\n\nBody:\n{}", status, headers.join("\n"), truncated);
                info!("[TOOL_SUCCESS] HttpRequestTool {} {} -> {}", method, url, status);
                Ok(tools::ToolResult { success: true, output, error: None })
            }
            Err(e) => {
                error!("[TOOL_ERROR] HttpRequestTool failed: {}", e);
                Ok(tools::ToolResult { success: false, output: String::new(), error: Some(format!("Request failed: {}", e)) })
            }
        }
    }
}
