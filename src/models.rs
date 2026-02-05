use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Holds the complete set of template names and their contents for local caching.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheData {
    /// Ordered list of all available template names.
    pub templates: Vec<String>,
    /// Map of template names to their respective .gitignore content.
    pub contents: HashMap<String, String>,
}
