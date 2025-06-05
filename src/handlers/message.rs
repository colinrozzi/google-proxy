use crate::api::GeminiClient;
use crate::bindings::theater::simple::runtime::log;
use crate::types::state::State;
use genai_types::{ProxyRequest, ProxyResponse};

pub fn handle_request(
    data: Vec<u8>,
    state_bytes: Vec<u8>,
) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
    log("Handling request in google-proxy actor");

    // Parse the state
    let state: State = match serde_json::from_slice(&state_bytes) {
        Ok(s) => s,
        Err(e) => {
            log(&format!("Error parsing state: {}", e));
            return Err(format!("Failed to parse state: {}", e));
        }
    };

    // Debug log the incoming request
    log(&format!(
        "Received request data: {}",
        String::from_utf8_lossy(&data)
    ));

    // Parse the request
    let request: ProxyRequest = match serde_json::from_slice(&data) {
        Ok(req) => req,
        Err(e) => {
            log(&format!("Error parsing request: {}", e));

            // Try to respond with a properly formatted error
            let error_response = ProxyResponse::Error {
                error: format!("Invalid request format: {}", e),
            };

            match serde_json::to_vec(&error_response) {
                Ok(bytes) => return Ok((Some(state_bytes), (Some(bytes),))),
                Err(_) => return Err(format!("Invalid request format: {}", e)),
            }
        }
    };

    // Create Gemini client
    let client = GeminiClient::new(state.api_key.clone());

    // Process based on operation type
    let response = match request {
        ProxyRequest::GenerateCompletion { request } => match request.try_into() {
            Ok(req) => match client.generate_content(req) {
                Ok(content) => {
                    log("Content generated successfully");
                    // Convert the content to the expected format
                    match content.try_into() {
                        Ok(content) => ProxyResponse::Completion {
                            completion: content,
                        },
                        Err(e) => {
                            log(&format!("Error converting content: {:?}", e));
                            return Err(format!("Failed to convert content: {:?}", e));
                        }
                    }
                }

                Err(e) => {
                    log(&format!("Error generating content: {:?}", e));
                    ProxyResponse::Error {
                        error: format!("Failed to generate content: {:?}", e),
                    }
                }
            },
            Err(e) => {
                log(&format!("Error converting request: {:?}", e));
                ProxyResponse::Error {
                    error: format!("Failed to convert request: {:?}", e),
                }
            }
        },

        ProxyRequest::ListModels => {
            log("Listing available models");

            match client.list_models() {
                Ok(models) => ProxyResponse::ListModels {
                    models: models.into_iter().map(|m| m.into()).collect(),
                },
                Err(e) => {
                    log(&format!("Error listing models: {:?}", e));
                    ProxyResponse::Error {
                        error: format!("Failed to list models: {:?}", e),
                    }
                }
            }
        }
    };

    // Serialize the response
    let response_bytes = match serde_json::to_vec(&response) {
        Ok(bytes) => bytes,
        Err(e) => {
            log(&format!("Error serializing response: {}", e));
            return Err(format!("Failed to serialize response: {}", e));
        }
    };

    // Return the updated state and response
    Ok((Some(state_bytes), (Some(response_bytes),)))
}
