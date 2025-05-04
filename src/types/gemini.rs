use serde::{Deserialize, Serialize};

/// Represents an error from the Gemini API
#[derive(Debug, Serialize, Deserialize)]
pub enum GeminiError {
    /// Invalid request format
    InvalidRequest(String),
    
    /// API error response
    ApiError { status: u16, message: String },
    
    /// HTTP error
    HttpError(String),
    
    /// Invalid response format
    InvalidResponse(String),
    
    /// JSON serialization/deserialization error
    SerdeError(String),
}

impl From<serde_json::Error> for GeminiError {
    fn from(err: serde_json::Error) -> Self {
        GeminiError::SerdeError(err.to_string())
    }
}

/// Role in a conversation (user or model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    
    #[serde(rename = "model")]
    Model,
    
    #[serde(rename = "system")]
    System,
}

/// Content part type (text or inline_data)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text(String),
    
    #[serde(rename = "inline_data")]
    InlineData {
        mime_type: String,
        data: String,
    },
}

/// A message in a conversation, consisting of a role and content parts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub parts: Vec<ContentPart>,
}

/// Generation config for Gemini API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Request to generate content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateContentRequest {
    pub contents: Vec<Message>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Message>,
}

/// Content from Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<ContentPart>,
    pub role: Role,
}

/// Usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetadata {
    pub prompt_token_count: u32,
    pub candidates_token_count: u32,
    pub total_token_count: u32,
}

/// Candidate response from Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: String,
    pub index: u32,
    pub safety_ratings: Vec<SafetyRating>,
}

/// Safety rating from Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

/// Response from Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateContentResponse {
    pub candidates: Vec<Candidate>,
    pub prompt_feedback: Option<PromptFeedback>,
    pub usage_metadata: Option<UsageMetadata>,
}

/// Feedback on prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptFeedback {
    pub safety_ratings: Vec<SafetyRating>,
}

/// Request type for the Google Proxy
#[derive(Debug, Serialize, Deserialize)]
pub enum GeminiRequest {
    GenerateContent {
        request: GenerateContentRequest,
        model: String,
        stream: bool,
    },
    ListModels,
}

/// Response from Google Proxy
#[derive(Debug, Serialize, Deserialize)]
pub enum GeminiResponse {
    Content {
        content: GenerateContentResponse,
    },
    ListModels {
        models: Vec<ModelInfo>,
    },
    Error {
        error: String,
    },
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub input_token_limit: u32,
    pub output_token_limit: u32,
    pub supported_generation_methods: Vec<String>,
    pub temperature_range: Option<(f32, f32)>,
    pub top_p_range: Option<(f32, f32)>,
    pub top_k_range: Option<(u32, u32)>,
}

impl ModelInfo {
    pub fn get_default_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-2.0-flash".to_string(),
                display_name: "Gemini 2.0 Flash".to_string(),
                description: Some("Optimized for speed, versatile on a broad range of tasks".to_string()),
                input_token_limit: 32_000,
                output_token_limit: 8_000,
                supported_generation_methods: vec!["generateContent".to_string(), "streamGenerateContent".to_string()],
                temperature_range: Some((0.0, 2.0)),
                top_p_range: Some((0.0, 1.0)),
                top_k_range: Some((1, 40)),
            },
            ModelInfo {
                id: "gemini-2.0-pro".to_string(),
                display_name: "Gemini 2.0 Pro".to_string(),
                description: Some("High-quality model with strong reasoning across a variety of tasks".to_string()),
                input_token_limit: 32_000,
                output_token_limit: 16_000,
                supported_generation_methods: vec!["generateContent".to_string(), "streamGenerateContent".to_string()],
                temperature_range: Some((0.0, 2.0)),
                top_p_range: Some((0.0, 1.0)),
                top_k_range: Some((1, 40)),
            },
            // Add more models as needed
        ]
    }
}
