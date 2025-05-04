use genai_types::{
    messages::{Role as GenaiRole, StopReason},
    CompletionRequest, CompletionResponse, Message, MessageContent, Usage,
};
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

    /// Unsupported feature
    UnsupportedFeature(String),

    /// Serialization error
    SerializationError(String),
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

impl From<GenaiRole> for Role {
    fn from(role: GenaiRole) -> Self {
        match role {
            GenaiRole::User => Role::User,
            GenaiRole::Assistant => Role::Model,
            GenaiRole::System => Role::System,
        }
    }
}

impl From<Role> for GenaiRole {
    fn from(role: Role) -> Self {
        match role {
            Role::User => GenaiRole::User,
            Role::Model => GenaiRole::Assistant,
            Role::System => GenaiRole::System,
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::Model
    }
}

/// Content part type (text or inline_data)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentPart {
    Text { text: String },
    InlineData { mime_type: String, data: String },
}

impl TryFrom<MessageContent> for ContentPart {
    type Error = GeminiError;

    fn try_from(content: MessageContent) -> Result<Self, GeminiError> {
        match content {
            MessageContent::Text { text } => Ok(ContentPart::Text { text }),
            _ => Err(GeminiError::UnsupportedFeature(
                "only text is available right now".to_string(),
            )),
        }
    }
}

impl TryFrom<ContentPart> for MessageContent {
    type Error = GeminiError;

    fn try_from(content: ContentPart) -> Result<Self, GeminiError> {
        match content {
            ContentPart::Text { text } => Ok(MessageContent::Text { text }),
            _ => Err(GeminiError::UnsupportedFeature(
                "only text is available right now".to_string(),
            )),
        }
    }
}

/// Content from Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<ContentPart>,
    #[serde(default)]
    pub role: Role,
}

impl TryFrom<Message> for Content {
    type Error = GeminiError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        Ok(Content {
            role: message.role.into(),
            parts: message
                .content
                .into_iter()
                .map(|part| {
                    part.try_into().map_err(|e| {
                        GeminiError::SerializationError(format!(
                            "Failed to convert message part: {:?}",
                            e
                        ))
                    })
                })
                .collect::<Result<Vec<ContentPart>, GeminiError>>()
                .map_err(|e| GeminiError::SerializationError(format!("{:?}", e)))?,
        })
    }
}

impl TryFrom<Content> for Message {
    type Error = GeminiError;
    fn try_from(content: Content) -> Result<Self, Self::Error> {
        Ok(Message {
            role: content.role.into(),
            content: content
                .parts
                .into_iter()
                .map(|part| {
                    part.try_into().map_err(|e| {
                        GeminiError::SerializationError(format!(
                            "Failed to convert message part: {:?}",
                            e
                        ))
                    })
                })
                .collect::<Result<Vec<MessageContent>, GeminiError>>()
                .map_err(|e| GeminiError::SerializationError(format!("{:?}", e)))?,
        })
    }
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
    pub model: String,

    pub contents: Vec<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
}

impl TryFrom<CompletionRequest> for GenerateContentRequest {
    type Error = GeminiError;
    fn try_from(request: CompletionRequest) -> Result<Self, Self::Error> {
        let system_instruction = if request.system.is_some() {
            Some(request.system.as_ref().map(|s| Content {
                role: Role::System,
                parts: vec![ContentPart::Text { text: s.clone() }],
            }))
        } else {
            None
        };

        // For the generation config, I really don't want to deal with this right now, sorry king
        // hope you are drinking some coffee
        let generation_config = None;

        Ok(GenerateContentRequest {
            model: request.model,
            contents: request
                .messages
                .into_iter()
                .map(|msg| msg.try_into())
                .collect::<Result<Vec<Content>, GeminiError>>()
                .map_err(|e| {
                    GeminiError::SerializationError(format!("Failed to convert message: {:?}", e))
                })?,
            generation_config,
            system_instruction: system_instruction.flatten(),
        })
    }
}

/// Usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetadata {
    pub prompt_token_count: u32,
    pub candidates_token_count: u32,
    pub total_token_count: u32,
}

impl TryFrom<UsageMetadata> for Usage {
    type Error = GeminiError;

    fn try_from(usage: UsageMetadata) -> Result<Self, Self::Error> {
        Ok(genai_types::Usage {
            input_tokens: usage.prompt_token_count,
            output_tokens: usage.candidates_token_count,
        })
    }
}

/// Candidate response from Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: FinishReason,
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub safety_ratings: Vec<SafetyRating>,
}

/// Finish reason for the a candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    #[serde(rename = "FINISH_REASON_UNSPECIFIED")]
    FinishReasonUnspecified,

    #[serde(rename = "STOP")]
    Stop,

    #[serde(rename = "MAX_TOKENS")]
    MaxTokens,

    #[serde(rename = "SAFETY")]
    Safety,

    #[serde(rename = "RECITATION")]
    Recitation,

    #[serde(rename = "LANGUAGE")]
    Language,

    #[serde(rename = "OTHER")]
    Other,

    #[serde(rename = "BLOCKLIST")]
    Blocklist,

    #[serde(rename = "PROHIBITED_CONTENT")]
    ProhibitedContent,

    #[serde(rename = "SPII")]
    Spii,

    #[serde(rename = "MALFORMED_FUNCTION_CALL")]
    MalformedFunctionCall,

    #[serde(rename = "IMAGE_SAFETY")]
    ImageSafety,
}

impl From<FinishReason> for StopReason {
    fn from(reason: FinishReason) -> Self {
        match reason {
            FinishReason::FinishReasonUnspecified => StopReason::EndTurn,
            FinishReason::Stop => StopReason::EndTurn,
            FinishReason::MaxTokens => StopReason::MaxTokens,
            _ => StopReason::Other(
                serde_json::to_string(&reason).unwrap_or_else(|_| "Unknown".to_string()),
            ),
        }
    }
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
    pub model_version: String,
}

impl TryFrom<GenerateContentResponse> for CompletionResponse {
    type Error = GeminiError;

    fn try_from(response: GenerateContentResponse) -> Result<Self, Self::Error> {
        // We are only interested in the first candidate for now
        let candidate = response.candidates[0].clone();
        let content = candidate
            .content
            .parts
            .iter()
            .map(|part| (*part).clone().try_into().unwrap())
            .collect();

        let usage = match response.usage_metadata {
            Some(usage) => Usage {
                input_tokens: usage.prompt_token_count,
                output_tokens: usage.candidates_token_count,
            },
            None => Usage {
                input_tokens: 0,
                output_tokens: 0,
            },
        };

        Ok(CompletionResponse {
            content,
            id: candidate.index.to_string(),
            model: response.model_version,
            role: candidate.content.role.into(),
            stop_reason: candidate.finish_reason.into(),
            stop_sequence: None,
            message_type: "gemini".to_string(),
            usage,
        })
    }
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
    Content { content: GenerateContentResponse },
    ListModels { models: Vec<ModelInfo> },
    Error { error: String },
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

impl From<ModelInfo> for genai_types::ModelInfo {
    fn from(model: ModelInfo) -> Self {
        genai_types::ModelInfo {
            id: model.id,
            display_name: model.display_name,
            provider: "google".to_string(),
            max_tokens: model.output_token_limit,
            pricing: None,
        }
    }
}

impl ModelInfo {
    pub fn get_default_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-2.0-flash".to_string(),
                display_name: "Gemini 2.0 Flash".to_string(),
                description: Some(
                    "Optimized for speed, versatile on a broad range of tasks".to_string(),
                ),
                input_token_limit: 32_000,
                output_token_limit: 8_000,
                supported_generation_methods: vec![
                    "generateContent".to_string(),
                    "streamGenerateContent".to_string(),
                ],
                temperature_range: Some((0.0, 2.0)),
                top_p_range: Some((0.0, 1.0)),
                top_k_range: Some((1, 40)),
            },
            ModelInfo {
                id: "gemini-2.0-pro".to_string(),
                display_name: "Gemini 2.0 Pro".to_string(),
                description: Some(
                    "High-quality model with strong reasoning across a variety of tasks"
                        .to_string(),
                ),
                input_token_limit: 32_000,
                output_token_limit: 16_000,
                supported_generation_methods: vec![
                    "generateContent".to_string(),
                    "streamGenerateContent".to_string(),
                ],
                temperature_range: Some((0.0, 2.0)),
                top_p_range: Some((0.0, 1.0)),
                top_k_range: Some((1, 40)),
            },
        ]
    }
}
