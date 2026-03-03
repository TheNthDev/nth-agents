use async_trait::async_trait;
use tracing::{info, warn, error};
use zeroclaw::tools;
use anyhow::Result;
use urlencoding::encode;

// Import coding tools module
pub mod coding_tools;

// Re-export coding tools for convenient access
pub use coding_tools::{
    CodingTool, FileReadTool, FileWriteTool, FileListTool,
    GitTool, TerminalTool, WorkspaceTool, CodeRunTool,
    FileEditTool, GlobSearchTool, ContentSearchTool,
    WebFetchTool, WebSearchTool, HttpRequestTool,
    PdfReadTool, ImageInfoTool,
};



/// A simple weather tool to demonstrate LLM tool integration
pub struct WeatherTool {
    api_key: Option<String>,
}

impl WeatherTool {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl tools::Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get the current weather for a specific city."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city to get the weather for."
                }
            },
            "required": ["city"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<tools::ToolResult> {
        let city = args.get("city")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        info!("[TOOL_CALL] WeatherTool for city: {}", city);
        
        // Use user-provided API key if available, otherwise check environment
        let api_key = self.api_key.clone().or_else(|| std::env::var("OPENWEATHERMAP_API_KEY").ok());

        if let Some(api_key) = api_key {
            // Geocoding step (OWM 3.0 recommends using lat/lon)
            let geo_url = format!("https://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}", encode(city), api_key);
            
            match reqwest::get(&geo_url).await {
                Ok(geo_resp) if geo_resp.status().is_success() => {
                    let geo_json: serde_json::Value = geo_resp.json().await?;
                    if let Some(location) = geo_json.get(0) {
                        let lat = location["lat"].as_f64().unwrap_or(0.0);
                        let lon = location["lon"].as_f64().unwrap_or(0.0);
                        
                        // OWM Current Weather API
                        let url = format!("https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=metric&appid={}", lat, lon, api_key);
                        
                        // Log a masked version for debugging
                        let masked_key = if api_key.len() > 8 {
                            format!("{}...{}", &api_key[..4], &api_key[api_key.len()-4..])
                        } else {
                            "****".to_string()
                        };
                        info!("[TOOL_DEBUG] Calling OpenWeatherMap 2.5 API: https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}", lat, lon, masked_key);

                        match reqwest::get(&url).await {
                            Ok(resp) => {
                                let status = resp.status();
                                if status.is_success() {
                                    match resp.json::<serde_json::Value>().await {
                                        Ok(json) => {
                                            let temp = json["main"]["temp"].as_f64().unwrap_or(0.0);
                                            let description = json["weather"][0]["description"].as_str().unwrap_or("unknown");
                                            info!("[TOOL_SUCCESS] WeatherTool result: {}°C, {}", temp, description);
                                            return Ok(tools::ToolResult {
                                                success: true,
                                                output: format!("The current weather in {} is {}, with a temperature of {}°C.", city, description, temp),
                                                error: None,
                                            });
                                        }
                                        Err(e) => {
                                            error!("[TOOL_ERROR] Failed to parse weather data: {}", e);
                                            return Ok(tools::ToolResult {
                                                success: false,
                                                output: String::new(),
                                                error: Some(format!("Failed to parse weather data: {}", e)),
                                            });
                                        }
                                    }
                                } else {
                                    warn!("[TOOL_ERROR] Weather API returned error status: {}", status);
                                    if status == reqwest::StatusCode::UNAUTHORIZED {
                                        error!("[TOOL_AUTH_ERROR] 401 Unauthorized: Please check your OPENWEATHERMAP_API_KEY in .env. Note: OWM 3.0 might require a separate subscription.");
                                    }
                                    return Ok(tools::ToolResult {
                                        success: false,
                                        output: String::new(),
                                        error: Some(format!("Weather API returned error: {}", status)),
                                    });
                                }
                            }
                            Err(e) => {
                                error!("[TOOL_NETWORK_ERROR] Failed to call weather API: {}", e);
                                return Ok(tools::ToolResult {
                                    success: false,
                                    output: String::new(),
                                    error: Some(format!("Failed to call weather API: {}", e)),
                                });
                            }
                        }
                    } else {
                        warn!("[TOOL_ERROR] Could not find coordinates for city: {}", city);
                        return Ok(tools::ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Could not find coordinates for city: {}", city)),
                        });
                    }
                }
                Ok(geo_resp) => {
                    let status = geo_resp.status();
                    error!("[TOOL_ERROR] Geocoding API returned error status: {}", status);
                    return Ok(tools::ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Geocoding API returned error: {}", status)),
                    });
                }
                Err(e) => {
                    error!("[TOOL_NETWORK_ERROR] Failed to call geocoding API: {}", e);
                    return Ok(tools::ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to call geocoding API: {}", e)),
                    });
                }
            }
        }

        // Mock weather data fallback if no API key is provided
        let weather = match city.to_lowercase().as_str() {
            "berlin" => "Sunny, 22°C",
            "san francisco" => "Foggy, 15°C",
            "tokyo" => "Rainy, 18°C",
            _ => "Partly cloudy, 20°C",
        };

        Ok(tools::ToolResult {
            success: true,
            output: format!("The weather in {} is {}. (Mock data - set OPENWEATHERMAP_API_KEY for real data)", city, weather),
            error: None,
        })
    }
}