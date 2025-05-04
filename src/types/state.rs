use serde::{Deserialize, Serialize};

/// Configuration options for the Google Gemini API proxy
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// The default Gemini model to use
    pub default_model: String,
    
    /// Maximum number of items to keep in the optional cache
    pub max_cache_size: Option<usize>,
    
    /// Request timeout in milliseconds
    pub timeout_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_model: "gemini-2.0-flash".to_string(),
            max_cache_size: Some(100),
            timeout_ms: 30000,  // 30 seconds
        }
    }
}

/// Main state for the google-proxy actor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    /// Actor ID
    pub id: String,
    
    /// Google API key
    pub api_key: String,
    
    /// Actor configuration
    pub config: Config,
    
    /// Store ID (if using runtime store)
    pub store_id: Option<String>,
}

impl State {
    pub fn new(
        id: String,
        api_key: String,
        store_id: Option<String>,
        config: Option<Config>,
    ) -> Self {
        Self {
            id,
            api_key,
            config: config.unwrap_or_default(),
            store_id,
        }
    }
}
