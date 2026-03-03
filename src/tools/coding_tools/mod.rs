mod file_read;
mod file_write;
mod file_list;
mod file_edit;
mod git;
mod terminal;
mod workspace;
mod code_run;
mod glob_search;
mod content_search;
mod web_fetch;
mod web_search;
mod http_request;
mod pdf_read;
mod image_info;

use async_trait::async_trait;
use std::collections::HashMap;
use zeroclaw::tools;

/// Base trait for coding tools that extend ZeroClaw's tool interface
#[async_trait]
pub trait CodingTool: tools::Tool {
    /// Get the tool's specific configuration requirements
    fn config(&self) -> &HashMap<String, String>;
    
    /// Set the tool's configuration requirements
    fn set_config(&mut self, config: HashMap<String, String>);
}

pub use file_read::FileReadTool;
pub use file_write::FileWriteTool;
pub use file_list::FileListTool;
pub use file_edit::FileEditTool;
pub use git::GitTool;
pub use terminal::TerminalTool;
pub use workspace::WorkspaceTool;
pub use code_run::CodeRunTool;
pub use glob_search::GlobSearchTool;
pub use content_search::ContentSearchTool;
pub use web_fetch::WebFetchTool;
pub use web_search::WebSearchTool;
pub use http_request::HttpRequestTool;
pub use pdf_read::PdfReadTool;
pub use image_info::ImageInfoTool;
