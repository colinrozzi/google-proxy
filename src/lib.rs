mod api;
mod bindings;
mod handlers;
pub mod types;

#[cfg(test)]
mod tests;

use crate::bindings::exports::theater::simple::actor::Guest;
use crate::bindings::exports::theater::simple::message_server_client::Guest as MessageServerClient;
use crate::bindings::theater::simple::runtime::log;
use crate::types::state::{InitConfig, State};

use bindings::theater::simple::environment;
use bindings::theater::simple::types::ChannelAccept;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct InitData {
    store_id: Option<String>,
    config: Option<InitConfig>,
}

struct Component;

impl Guest for Component {
    fn init(data: Option<Vec<u8>>, params: (String,)) -> Result<(Option<Vec<u8>>,), String> {
        log("Initializing google-proxy actor");
        let (id,) = params;
        log(&format!("Actor ID: {}", id));

        // Parse initialization data
        let init_data: InitData = match data {
            Some(bytes) => match serde_json::from_slice(&bytes) {
                Ok(data) => data,
                Err(e) => {
                    return Err(format!("Failed to parse init data: {}", e));
                }
            },
            None => {
                return Err("No initialization data provided".to_string());
            }
        };

        log("Init data parsed successfully");

        let google_api_key = match environment::get_var("GEMINI_API_KEY") {
            Some(key) => {
                log("Google API key found in environment");
                key
            }
            None => {
                return Err("Google API key not found in environment".to_string());
            }
        };

        // Initialize state
        let state = State::new(id, google_api_key, init_data.store_id, init_data.config);

        log("State initialized");

        // Serialize and return the state
        match serde_json::to_vec(&state) {
            Ok(state_bytes) => {
                log("Actor initialized successfully");
                Ok((Some(state_bytes),))
            }
            Err(e) => Err(format!("Failed to serialize state: {}", e)),
        }
    }
}

impl MessageServerClient for Component {
    fn handle_send(
        state: Option<Vec<u8>>,
        _params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Handling send message in google-proxy");

        // Nothing to return for a send
        Ok((state,))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Handling request message in google-proxy");
        let (request_id, data) = params;
        log(&format!("Request ID: {}", request_id));

        // Use our message handler
        handlers::message::handle_request(data, state.unwrap())
    }

    fn handle_channel_open(
        state: Option<Vec<u8>>,
        _params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (ChannelAccept,)), String> {
        log("Channel open request received");

        Ok((
            state,
            (ChannelAccept {
                accepted: true,
                message: None,
            },),
        ))
    }

    fn handle_channel_close(
        state: Option<Vec<u8>>,
        params: (String,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (channel_id,) = params;
        log(&format!("Channel {} closed", channel_id));

        Ok((state,))
    }

    fn handle_channel_message(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (channel_id, _message) = params;
        log(&format!("Received message on channel {}", channel_id));

        Ok((state,))
    }
}

bindings::export!(Component with_types_in bindings);
