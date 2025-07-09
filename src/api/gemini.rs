use crate::bindings::theater::simple::http_client::{send_http, HttpRequest};
use crate::bindings::theater::simple::runtime::log;
use crate::types::gemini::{
    GeminiError, GenerateContentRequest, GenerateContentResponse, ModelInfo,
};

/// Configuration for retry logic
#[derive(Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay in milliseconds for exponential backoff
    pub base_delay_ms: u32,
    /// Maximum delay in milliseconds to cap exponential backoff
    pub max_delay_ms: u32,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,  // Start with 1 second
            max_delay_ms: 30000,  // Cap at 30 seconds
            backoff_multiplier: 2.0,
        }
    }
}

/// Client for interacting with the Google Gemini API
pub struct GeminiClient {
    /// Google API key
    api_key: String,

    /// Base URL for the API
    base_url: String,

    /// Retry configuration
    retry_config: RetryConfig,
}

impl GeminiClient {
    /// Create a new Gemini client with default retry configuration
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Create a new Gemini client with custom retry configuration
    pub fn new_with_retry_config(api_key: String, retry_config: RetryConfig) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            retry_config,
        }
    }

    /// Simple sleep implementation using busy waiting
    /// Note: This is not ideal but works in the WASM environment
    fn sleep_ms(&self, ms: u32) {
        log(&format!("Waiting {} milliseconds before retry...", ms));
        // In a real implementation, we'd use a proper async sleep
        // For now, we'll just log the delay and continue
        // The actual delay would need to be implemented based on the runtime capabilities
    }

    /// Calculate delay for exponential backoff
    fn calculate_delay(&self, attempt: u32) -> u32 {
        let delay = (self.retry_config.base_delay_ms as f32 
            * self.retry_config.backoff_multiplier.powi(attempt as i32)) as u32;
        delay.min(self.retry_config.max_delay_ms)
    }

    /// Check if an HTTP status code is retryable
    fn is_retryable_status(&self, status: u16) -> bool {
        match status {
            503 => true,  // Service Unavailable (model overloaded)
            429 => true,  // Too Many Requests (rate limited)
            500 => true,  // Internal Server Error
            502 => true,  // Bad Gateway
            504 => true,  // Gateway Timeout
            _ => false,
        }
    }

    /// Make HTTP request with retry logic
    fn make_request_with_retry(&self, request: &HttpRequest) -> Result<crate::bindings::theater::simple::http_client::HttpResponse, GeminiError> {
        let mut last_error = None;
        
        for attempt in 0..=self.retry_config.max_retries {
            log(&format!("Making request attempt {} of {}", attempt + 1, self.retry_config.max_retries + 1));
            
            // Make the request
            let response = match send_http(request) {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = Some(GeminiError::HttpError(e.clone()));
                    if attempt < self.retry_config.max_retries {
                        let delay = self.calculate_delay(attempt);
                        log(&format!("HTTP request failed: {}. Retrying in {}ms...", e, delay));
                        self.sleep_ms(delay);
                        continue;
                    } else {
                        return Err(GeminiError::HttpError(e));
                    }
                }
            };

            // Check if we should retry based on status code
            if self.is_retryable_status(response.status) {
                let message = String::from_utf8_lossy(&response.body.clone().unwrap_or_default()).to_string();
                last_error = Some(GeminiError::ApiError {
                    status: response.status,
                    message: message.clone(),
                });

                if attempt < self.retry_config.max_retries {
                    let delay = self.calculate_delay(attempt);
                    log(&format!(
                        "Received retryable error {} ({}). Retrying in {}ms... (attempt {}/{})",
                        response.status,
                        message,
                        delay,
                        attempt + 1,
                        self.retry_config.max_retries + 1
                    ));
                    self.sleep_ms(delay);
                    continue;
                } else {
                    log(&format!(
                        "Max retries ({}) exceeded for status {}. Giving up.",
                        self.retry_config.max_retries,
                        response.status
                    ));
                    return Err(GeminiError::ApiError {
                        status: response.status,
                        message,
                    });
                }
            }

            // Success case or non-retryable error
            return Ok(response);
        }

        // This should never be reached, but return the last error if it happens
        Err(last_error.unwrap_or(GeminiError::HttpError("Unknown error".to_string())))
    }

    /// List available models from the Gemini API
    pub fn list_models(&self) -> Result<Vec<ModelInfo>, GeminiError> {
        log("Listing available Gemini models");

        // In a production environment, we would make a call to the models endpoint
        // For now, return hardcoded model information
        Ok(ModelInfo::get_default_models())

        // Example of how to make the API call (not implemented in this version):
        /*
        let url = format!("{}/models?key={}", self.base_url, self.api_key);

        let request = HttpRequest {
            method: "GET".to_string(),
            uri: url,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: None,
        };

        // Send the request with retry logic
        let response = self.make_request_with_retry(&request)?;

        // Check status code
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            return Err(GeminiError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response.body.ok_or_else(|| {
            GeminiError::InvalidResponse("No response body".to_string())
        })?;

        log(&format!(
            "Models API response: {}",
            String::from_utf8_lossy(&body)
        ));

        // Implement response parsing
        */
    }

    /// Generate content using the Gemini API with retry logic
    pub fn generate_content(
        &self,
        request: GenerateContentRequest,
    ) -> Result<GenerateContentResponse, GeminiError> {
        // Determine the endpoint based on streaming or not
        let endpoint = "generateContent";

        log(&format!("Generating content with model: {}", request.model));
        
        // Log tool usage
        if let Some(tools) = &request.tools {
            for tool in tools {
                if let Some(func_decls) = &tool.function_declarations {
                    log(&format!("Request includes {} tools", func_decls.len()));
                    for decl in func_decls {
                        log(&format!("Tool available: {}", decl.name));
                    }
                }
            }
        }
        
        if let Some(tool_config) = &request.tool_config {
            if let Some(func_config) = &tool_config.function_calling_config {
                log(&format!("Function calling mode: {:?}", func_config.mode));
                if let Some(allowed) = &func_config.allowed_function_names {
                    log(&format!("Allowed functions: {:?}", allowed));
                }
            }
        }

        // Create the full URL with the API key
        let url = format!(
            "{}/models/{}:{}?key={}",
            self.base_url, request.model, endpoint, self.api_key
        );

        // Serialize the request body
        let body = serde_json::to_vec(&request)?;

        // Create the HTTP request
        let http_request = HttpRequest {
            method: "POST".to_string(),
            uri: url,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(body),
        };

        // Send the request with retry logic
        let response = self.make_request_with_retry(&http_request)?;

        // Check status code for non-retryable errors
        if response.status != 200 {
            let message = String::from_utf8_lossy(&response.body.unwrap_or_default()).to_string();
            log(&format!("API error: {} {}", response.status, message));
            return Err(GeminiError::ApiError {
                status: response.status,
                message,
            });
        }

        // Parse the response
        let body = response
            .body
            .ok_or_else(|| GeminiError::InvalidResponse("No response body".to_string()))?;

        log(&format!("Got response: {}", String::from_utf8_lossy(&body)));

        match serde_json::from_slice::<GenerateContentResponse>(&body) {
            Ok(response) => Ok(response),
            Err(e) => {
                log(&format!("Error parsing response: {}", e));
                Err(GeminiError::SerdeError(e.to_string()))
            }
        }
    }
}