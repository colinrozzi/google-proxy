use std::convert::TryFrom;

use crate::types::gemini::GenerateContentResponse;
use crate::types::state::{Config, RetryConfig, InitConfig, State};
use genai_types::messages::StopReason;
use genai_types::MessageContent;

#[test]
fn test_minimal_init_config() {
    let init_config = InitConfig {
        default_model: Some("gemini-2.0-flash".to_string()),
        max_cache_size: None,
        timeout_ms: None,
        retry_config: None,
    };

    let state = State::new(
        "test-id".to_string(),
        "test-api-key".to_string(),
        None,
        Some(init_config),
    );

    assert_eq!(state.config.default_model, "gemini-2.0-flash");
    assert_eq!(state.config.max_cache_size, Some(100)); // default
    assert_eq!(state.config.timeout_ms, 30000); // default
    assert_eq!(state.config.retry_config.max_retries, 3); // default
    assert_eq!(state.config.retry_config.base_delay_ms, 1000); // default
}

#[test]
fn test_empty_init_config() {
    let state = State::new(
        "test-id".to_string(),
        "test-api-key".to_string(),
        None,
        None,
    );

    assert_eq!(state.config.default_model, "gemini-2.0-flash");
    assert_eq!(state.config.max_cache_size, Some(100));
    assert_eq!(state.config.timeout_ms, 30000);
    assert_eq!(state.config.retry_config.max_retries, 3);
}

#[test]
fn gemini_function_call_pipeline() {
    // ─── JSON copied verbatim from the proxy log ────────────────────────────────
    let raw = r#"
    {
      "candidates": [
        {
          "content": {
            "parts": [
              {
                "functionCall": {
                  "name": "list_allowed_dirs",
                  "args": {}
                }
              }
            ],
            "role": "model"
          },
          "finishReason": "STOP"
        }
      ],
      "usageMetadata": {
        "promptTokenCount": 654,
        "candidatesTokenCount": 5,
        "totalTokenCount": 659
      },
      "modelVersion": "gemini-2.0-flash"
    }
    "#;
    // ─── 1.  Parse exactly what Gemini returned ────────────────────────────────
    let parsed: GenerateContentResponse =
        serde_json::from_str(raw).expect("should deserialize after the #[serde(untagged)] fix");

    // ─── 2.  Run it through *your* TryFrom impl; covers the whole pipeline ─────
    let completion = genai_types::CompletionResponse::try_from(parsed)
        .expect("conversion to CompletionResponse");

    // ─── 3.  Sanity assertions that prove the enum variant resolved correctly ──
    assert_eq!(completion.stop_reason, StopReason::ToolUse);

    match &completion.content[0] {
        MessageContent::ToolUse { name, .. } => {
            assert_eq!(name, "list_allowed_dirs");
        }
        other => panic!("unexpected first content part: {:?}", other),
    }
}

#[test]
fn gemini_text_pipeline() {
    // ─── JSON copied verbatim from the proxy log ────────────────────────────────
    let raw = r#"
    {
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "text": "Hi there! How can I help you today?\n"
          }
        ],
        "role": "model"
      },
      "finishReason": "STOP",
      "avgLogprobs": -0.053460695526816628
    }
  ],
  "usageMetadata": {
    "promptTokenCount": 644,
    "candidatesTokenCount": 11,
    "totalTokenCount": 655,
    "promptTokensDetails": [
      {
        "modality": "TEXT",
        "tokenCount": 644
      }
    ],
    "candidatesTokensDetails": [
      {
        "modality": "TEXT",
        "tokenCount": 11
      }
    ]
  },
  "modelVersion": "gemini-2.0-flash"
}
    "#;
    // ─── 1.  Parse exactly what Gemini returned ────────────────────────────────
    let parsed: GenerateContentResponse =
        serde_json::from_str(raw).expect("should deserialize after the #[serde(untagged)] fix");

    // ─── 2.  Run it through *your* TryFrom impl; covers the whole pipeline ─────
    let completion = genai_types::CompletionResponse::try_from(parsed)
        .expect("conversion to CompletionResponse");

    // ─── 3.  Sanity assertions that prove the enum variant resolved correctly ──
    assert_eq!(completion.stop_reason, StopReason::EndTurn);

    match &completion.content[0] {
        MessageContent::Text { text, .. } => {
            assert_eq!(text, "Hi there! How can I help you today?\n");
        }
        other => panic!("unexpected first content part: {:?}", other),
    }
}

#[test]
fn test_retry_config_defaults() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.base_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
    assert_eq!(config.backoff_multiplier, 2.0);
}

#[test]
fn test_config_with_retry_defaults() {
    let config = Config::default();
    assert_eq!(config.retry_config.max_retries, 3);
    assert_eq!(config.retry_config.base_delay_ms, 1000);
    assert_eq!(config.retry_config.max_delay_ms, 30000);
    assert_eq!(config.retry_config.backoff_multiplier, 2.0);
}

#[test]
fn test_retry_config_serialization() {
    let config = RetryConfig {
        max_retries: 5,
        base_delay_ms: 2000,
        max_delay_ms: 60000,
        backoff_multiplier: 1.5,
    };
    
    let json = serde_json::to_string(&config).expect("should serialize");
    let deserialized: RetryConfig = serde_json::from_str(&json).expect("should deserialize");
    
    assert_eq!(deserialized.max_retries, 5);
    assert_eq!(deserialized.base_delay_ms, 2000);
    assert_eq!(deserialized.max_delay_ms, 60000);
    assert_eq!(deserialized.backoff_multiplier, 1.5);
}

#[test]
fn test_config_with_custom_retry_serialization() {
    let config = Config {
        default_model: "gemini-2.0-pro".to_string(),
        max_cache_size: Some(50),
        timeout_ms: 15000,
        retry_config: RetryConfig {
            max_retries: 2,
            base_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 3.0,
        },
    };
    
    let json = serde_json::to_string(&config).expect("should serialize");
    let deserialized: Config = serde_json::from_str(&json).expect("should deserialize");
    
    assert_eq!(deserialized.default_model, "gemini-2.0-pro");
    assert_eq!(deserialized.max_cache_size, Some(50));
    assert_eq!(deserialized.timeout_ms, 15000);
    assert_eq!(deserialized.retry_config.max_retries, 2);
    assert_eq!(deserialized.retry_config.base_delay_ms, 500);
    assert_eq!(deserialized.retry_config.max_delay_ms, 10000);
    assert_eq!(deserialized.retry_config.backoff_multiplier, 3.0);
}