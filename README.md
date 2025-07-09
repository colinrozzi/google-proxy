# Google Proxy Actor

A WebAssembly component actor that serves as a proxy for the Google Gemini API, making it easy to interact with Gemini models within the Theater system through message passing.

## Features

- **API Key Management**: Securely stores and manages Google API keys
- **Message Interface**: Simple request-response messaging system
- **Model Information**: Includes details about available Gemini models
- **Error Handling**: Robust error reporting and handling
- **Retry Logic**: Automatic retry with exponential backoff for transient API errors

## Usage

The actor implements a simple request-response message interface that supports:

- **Text Generation**: Generate responses from Gemini models
- **Image Understanding**: Process images with text prompts
- **Streaming Responses**: Support for streaming mode
- **Multi-turn Conversations**: Support for chat-like interactions

## Configuration

The actor accepts these configuration parameters during initialization. All configuration fields are optional and will use sensible defaults if not provided:

```json
{
  "store_id": "optional-store-id",
  "config": {
    "default_model": "gemini-2.0-flash",
    "max_cache_size": 100,
    "timeout_ms": 30000,
    "retry_config": {
      "max_retries": 3,
      "base_delay_ms": 1000,
      "max_delay_ms": 30000,
      "backoff_multiplier": 2.0
    }
  }
}
```

**Minimal Configuration Example:**
```json
{
  "store_id": null,
  "config": {
    "default_model": "gemini-2.0-flash"
  }
}
```

**Default Values:**
- `default_model`: "gemini-2.0-flash"
- `max_cache_size`: 100
- `timeout_ms`: 30000 (30 seconds)
- `retry_config`: Uses default retry configuration (see below)

Note: The `GEMINI_API_KEY` environment variable is required and must be set in the actor's environment.

### Retry Configuration

The retry system automatically handles transient API errors, particularly useful for:

- **503 Service Unavailable**: When the model is overloaded
- **429 Too Many Requests**: When rate limits are exceeded
- **500/502/504 Server Errors**: Temporary server issues

**Configuration Options:**

- `max_retries`: Maximum number of retry attempts (default: 3)
- `base_delay_ms`: Initial delay in milliseconds (default: 1000)
- `max_delay_ms`: Maximum delay cap in milliseconds (default: 30000)
- `backoff_multiplier`: Exponential backoff multiplier (default: 2.0)

**Example Retry Behavior:**
- Attempt 1: Fails with 503 → Wait 1 second
- Attempt 2: Fails with 503 → Wait 2 seconds  
- Attempt 3: Fails with 503 → Wait 4 seconds
- Attempt 4: Fails with 503 → Give up and return error

## Building

Build the actor using cargo-component:

```bash
cargo component build --release --target wasm32-unknown-unknown
```

Then update the `component_path` in `manifest.toml` to point to the built WASM file.

## Starting

Start the actor using the Theater system:

```rust
let actor_id = start_actor(
    "/path/to/google-proxy/manifest.toml",
    Some(init_data),
    ("google-proxy-instance",)
);
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.