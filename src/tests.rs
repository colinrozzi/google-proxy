use std::convert::TryFrom;

use crate::types::gemini::GenerateContentResponse;
use genai_types::messages::StopReason;
use genai_types::MessageContent;

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
