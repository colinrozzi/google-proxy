# Google Proxy Actor

A WebAssembly component actor that serves as a proxy for the Google Gemini API, making it easy to interact with Gemini models within the Theater system through message passing.

## Features

- **API Key Management**: Securely stores and manages Google API keys
- **Message Interface**: Simple request-response messaging system
- **Model Information**: Includes details about available Gemini models
- **Error Handling**: Robust error reporting and handling

## Usage

The actor implements a simple request-response message interface that supports:

- **Text Generation**: Generate responses from Gemini models
- **Image Understanding**: Process images with text prompts
- **Streaming Responses**: Support for streaming mode
- **Multi-turn Conversations**: Support for chat-like interactions

## Configuration

The actor accepts these configuration parameters during initialization:

```json
{
  "google_api_key": "YOUR_GEMINI_API_KEY",
  "store_id": "optional-store-id",
  "config": {
    "default_model": "gemini-2.0-flash",
    "max_cache_size": 100,
    "timeout_ms": 30000
  }
}
```

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
