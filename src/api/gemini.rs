use crate::bindings::theater::simple::http_client::{send_http, HttpRequest};
use crate::bindings::theater::simple::runtime::log;
use crate::types::gemini::{
    GeminiError, GenerateContentRequest, GenerateContentResponse, ModelInfo,
};



/// Client for interacting with the Google Gemini API
pub struct GeminiClient {
    /// Google API key
    api_key: String,

    /// Base URL for the API
    base_url: String,


}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
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

        // Send the request
        let response = match send_http(&request) {
            Ok(resp) => resp,
            Err(e) => return Err(GeminiError::HttpError(e)),
        };

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

    /// Generate content using the Gemini API
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

        // Send the request
        let response = match send_http(&http_request) {
            Ok(resp) => resp,
            Err(e) => return Err(GeminiError::HttpError(e)),
        };

        // Check status code
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
