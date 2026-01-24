//! JavaScript bindings for Firebase via wasm-bindgen
//!
//! These functions call into the JavaScript bridge defined in firebase_bridge.js

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Initialization
    #[wasm_bindgen(js_name = initializeFirebase)]
    pub fn initialize_firebase() -> bool;

    #[wasm_bindgen(js_name = isFirebaseReady)]
    pub fn is_firebase_ready() -> bool;

    // Authentication
    #[wasm_bindgen(js_name = signInAnonymously, catch)]
    pub async fn sign_in_anonymously() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = getCurrentUserId)]
    pub fn get_current_user_id() -> Option<String>;

    #[wasm_bindgen(js_name = isAuthenticated)]
    pub fn is_authenticated() -> bool;

    // Room management
    #[wasm_bindgen(js_name = createRoom, catch)]
    pub async fn create_room(
        room_code: &str,
        host_name: &str,
        character_id: u8,
        color_index: usize,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = roomExists, catch)]
    pub async fn room_exists(room_code: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = joinRoom, catch)]
    pub async fn join_room(
        room_code: &str,
        player_name: &str,
        character_id: u8,
        color_index: usize,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = leaveRoom, catch)]
    pub async fn leave_room(room_code: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = setPlayerReady, catch)]
    pub async fn set_player_ready(room_code: &str, is_ready: bool) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = updatePlayerAppearance, catch)]
    pub async fn update_player_appearance(
        room_code: &str,
        character_id: u8,
        color_index: usize,
        name: Option<String>,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = startGame, catch)]
    pub async fn start_game(room_code: &str) -> Result<JsValue, JsValue>;

    // Game state sync
    #[wasm_bindgen(js_name = writeGameState, catch)]
    pub async fn write_game_state(room_code: &str, state_json: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = submitAction, catch)]
    pub async fn submit_action(room_code: &str, action_json: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = markActionProcessed, catch)]
    pub async fn mark_action_processed(
        room_code: &str,
        action_id: &str,
    ) -> Result<JsValue, JsValue>;

    // Subscriptions
    #[wasm_bindgen(js_name = subscribeToGameState)]
    pub fn subscribe_to_game_state(room_code: &str);

    #[wasm_bindgen(js_name = subscribeToActions)]
    pub fn subscribe_to_actions(room_code: &str);

    #[wasm_bindgen(js_name = subscribeToPlayers)]
    pub fn subscribe_to_players(room_code: &str);

    #[wasm_bindgen(js_name = subscribeToMetadata)]
    pub fn subscribe_to_metadata(room_code: &str);

    // Polling (for receiving Firebase updates)
    #[wasm_bindgen(js_name = pollGameState)]
    pub fn poll_game_state() -> Option<String>;

    #[wasm_bindgen(js_name = pollActions)]
    pub fn poll_actions() -> Option<String>;

    #[wasm_bindgen(js_name = pollPlayers)]
    pub fn poll_players() -> Option<String>;

    #[wasm_bindgen(js_name = hasGameStarted)]
    pub fn has_game_started() -> bool;

    #[wasm_bindgen(js_name = getRandomizeOrder)]
    pub fn get_randomize_order() -> bool;

    #[wasm_bindgen(js_name = setRandomizeOrder, catch)]
    pub async fn set_randomize_order(room_code: &str, randomize: bool) -> Result<JsValue, JsValue>;

    // Error handling
    #[wasm_bindgen(js_name = getFirebaseError)]
    pub fn get_firebase_error() -> Option<String>;

    // Cleanup
    #[wasm_bindgen(js_name = unsubscribeAll)]
    pub fn unsubscribe_all();

    #[wasm_bindgen(js_name = deleteRoom, catch)]
    pub async fn delete_room(room_code: &str) -> Result<JsValue, JsValue>;
}

/// Wrapper for async Firebase operations
pub mod async_ops {
    use super::*;
    use wasm_bindgen_futures::spawn_local;

    /// Initialize Firebase and sign in anonymously
    pub fn init_and_authenticate(on_complete: impl FnOnce(Result<String, String>) + 'static) {
        spawn_local(async move {
            if !initialize_firebase() {
                on_complete(Err("Failed to initialize Firebase".to_string()));
                return;
            }

            match sign_in_anonymously().await {
                Ok(uid) => {
                    if let Some(uid_str) = uid.as_string() {
                        on_complete(Ok(uid_str));
                    } else {
                        on_complete(Err("Failed to get user ID".to_string()));
                    }
                }
                Err(e) => {
                    let msg = e
                        .as_string()
                        .unwrap_or_else(|| "Unknown authentication error".to_string());
                    on_complete(Err(msg));
                }
            }
        });
    }

    /// Create a new room
    pub fn create_room_async(
        room_code: String,
        host_name: String,
        character_id: u8,
        color_index: usize,
        on_complete: impl FnOnce(Result<(), String>) + 'static,
    ) {
        spawn_local(async move {
            match create_room(&room_code, &host_name, character_id, color_index).await {
                Ok(result) => {
                    if result.as_bool().unwrap_or(false) {
                        on_complete(Ok(()));
                    } else {
                        on_complete(Err(
                            get_firebase_error().unwrap_or_else(|| "Failed to create room".into())
                        ));
                    }
                }
                Err(e) => {
                    on_complete(Err(
                        e.as_string()
                            .unwrap_or_else(|| "Room creation error".to_string())
                    ));
                }
            }
        });
    }

    /// Join an existing room
    pub fn join_room_async(
        room_code: String,
        player_name: String,
        character_id: u8,
        color_index: usize,
        on_complete: impl FnOnce(Result<(), String>) + 'static,
    ) {
        spawn_local(async move {
            match join_room(&room_code, &player_name, character_id, color_index).await {
                Ok(result) => {
                    if result.as_bool().unwrap_or(false) {
                        on_complete(Ok(()));
                    } else {
                        on_complete(Err(
                            get_firebase_error().unwrap_or_else(|| "Failed to join room".into())
                        ));
                    }
                }
                Err(e) => {
                    on_complete(Err(
                        e.as_string()
                            .unwrap_or_else(|| "Join room error".to_string())
                    ));
                }
            }
        });
    }

    /// Write game state to Firebase
    pub fn write_state_async(room_code: String, state_json: String) {
        spawn_local(async move {
            if let Err(e) = write_game_state(&room_code, &state_json).await {
                bevy::log::warn!(
                    "Failed to write game state: {}",
                    e.as_string().unwrap_or_default()
                );
            }
        });
    }

    /// Submit an action to Firebase
    pub fn submit_action_async(room_code: String, action_json: String) {
        spawn_local(async move {
            if let Err(e) = submit_action(&room_code, &action_json).await {
                bevy::log::warn!(
                    "Failed to submit action: {}",
                    e.as_string().unwrap_or_default()
                );
            }
        });
    }

    /// Start the game
    pub fn start_game_async(room_code: String, on_complete: impl FnOnce(Result<(), String>) + 'static) {
        spawn_local(async move {
            match start_game(&room_code).await {
                Ok(result) => {
                    if result.as_bool().unwrap_or(false) {
                        on_complete(Ok(()));
                    } else {
                        on_complete(Err("Failed to start game".to_string()));
                    }
                }
                Err(e) => {
                    on_complete(Err(e.as_string().unwrap_or_else(|| "Start game error".to_string())));
                }
            }
        });
    }

    /// Set player ready status
    pub fn set_ready_async(room_code: String, is_ready: bool) {
        spawn_local(async move {
            let _ = set_player_ready(&room_code, is_ready).await;
        });
    }

    /// Update player appearance (character, color, and optionally name)
    pub fn update_appearance_async(room_code: String, character_id: u8, color_index: usize, name: Option<String>) {
        spawn_local(async move {
            let _ = update_player_appearance(&room_code, character_id, color_index, name).await;
        });
    }

    /// Leave the room
    pub fn leave_room_async(room_code: String) {
        spawn_local(async move {
            let _ = leave_room(&room_code).await;
        });
    }

    /// Set randomize order (host only)
    pub fn set_randomize_order_async(room_code: String, randomize: bool) {
        spawn_local(async move {
            let _ = set_randomize_order(&room_code, randomize).await;
        });
    }
}
