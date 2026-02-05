use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

use crate::models::CacheData;

/// Responsible for all external API communication and local caching.
pub struct ApiClient {
    client: reqwest::Client,
    cache_path: PathBuf,
}

/// Helper struct for deserializing Toptal's template JSON format.
#[derive(serde::Deserialize)]
struct ToptalTemplate {
    name: String,
    contents: String,
}

impl ApiClient {
    /// Initializes a new ApiClient, creating the necessary local cache directories.
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("autogitignore-tui"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let proj_dirs = ProjectDirs::from("com", "autogitignore", "autogitignore")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine cache directory"))?;
        let cache_dir = proj_dirs.cache_dir().to_path_buf();
        fs::create_dir_all(&cache_dir)?;
        let cache_path = cache_dir.join("cache.json");

        Ok(Self { client, cache_path })
    }

    /// Attempts to load the template data from the local cache file.
    pub fn load_cache(&self) -> Option<CacheData> {
        if !self.cache_path.exists() {
            return None;
        }
        let content = fs::read_to_string(&self.cache_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Persists the provided CacheData to the local file system.
    pub fn save_cache(&self, data: &CacheData) -> Result<()> {
        let content = serde_json::to_string(data)?;
        fs::write(&self.cache_path, content)?;
        Ok(())
    }

    /// Fetches the latest list of templates and their contents from gitignore.io (Toptal).
    pub async fn fetch_all_data(&self) -> Result<CacheData> {
        let url = "https://www.toptal.com/developers/gitignore/api/list?format=json";
        let response = self.client.get(url).send().await?;
        
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!("Toptal API error: {}", status));
        }

        let data: std::collections::HashMap<String, ToptalTemplate> = response.json().await?;
        
        let mut templates = Vec::new();
        let mut contents = std::collections::HashMap::new();

        for (_key, val) in data {
            templates.push(val.name.clone());
            contents.insert(val.name, val.contents);
        }

        templates.sort();

        Ok(CacheData {
            templates,
            contents,
        })
    }

}
