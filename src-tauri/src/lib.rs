mod actions;
pub(crate) mod gui_logger;
pub(crate) mod settings;
pub(crate) mod uploader_client;

use crate::actions::upload_album::upload_album;

use gui_logger::GuiLogger;
use music_uploader_server::model::AlbumSearchResponse;
use serde::{Deserialize, Serialize};
use settings::{load_settings, Settings, UserEditableSettings};
use std::{env, sync::RwLock};
use tauri::{AppHandle, Manager, State};
use uploader_client::{MusicUploaderClient, MusicUploaderClientConfig, MusicUploaderClientError};

#[derive(Deserialize)]
struct Song {
    song_name: String,
    path: String,
}

#[tauri::command]
fn generate_guid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tauri::command]
fn get_valid_extensions(state: State<'_, GuiState>) -> Vec<String> {
    state
        .run_state
        .as_ref()
        .map(|s| s.settings.read().unwrap().valid_extensions.clone())
        .unwrap_or(Vec::new())
}

#[tauri::command]
async fn get_startup_message(state: State<'_, GuiState>) -> Result<String, String> {
    Ok(state.startup_message.clone())
}

#[tauri::command]
async fn run_settings_checks(state: State<'_, GuiState>) -> Result<String, String> {
    let logger = GuiLogger::new(state.app_handle.clone());
    if let Some(ref run_state) = state.run_state {
        let config = run_state.get_config();
        let (user, server_url) = {
            let settings = &run_state.settings.read().unwrap();
            (settings.user.clone(), settings.server_url.clone())
        };
        match run_state.client.check_conn(&config).await {
            Ok(_) => {
                logger.log("Connection is good".to_string());
            }
            Err(s) => {
                logger.log(format!("Cannot connect with {}: {}", server_url, s));
            }
        };
        match run_state.client.check_auth(&config).await {
            Ok(_) => {
                logger.log(format!("Authentication valid: hello {}", user));
            }
            Err(s) => {
                logger.log(format!("Authentication unsuccesful: {}", s));
            }
        };
    }
    Ok("settings checks complete".to_string())
}

#[tauri::command]
fn reload_settings(state: State<'_, GuiState>) -> Result<String, String> {
    let app_handle = &state.app_handle;
    let result = load_settings(app_handle)?;
    let logger = GuiLogger::new(app_handle.clone());
    match state.run_state.as_ref() {
        Some(run_state) => {
            let mut settings = run_state.settings.write().unwrap();
            *settings = result.settings;
            logger.log(result.startup_message);
            Ok("Success".to_string())
        }
        None => {
            logger.log(result.startup_message);
            Err("run state was empty".to_string())
        }
    }
}

#[derive(Serialize)]
struct GetSettingsResult {
    settings: Option<UserEditableSettings>,
    success: bool,
}

impl GetSettingsResult {
    pub fn success(settings: UserEditableSettings) -> Self {
        GetSettingsResult {
            settings: Some(settings),
            success: true,
        }
    }

    pub fn fail() -> Self {
        GetSettingsResult {
            settings: None,
            success: false,
        }
    }
}

#[tauri::command]
fn get_settings(state: State<'_, GuiState>) -> GetSettingsResult {
    match state.run_state.as_ref() {
        Some(run_state) => GetSettingsResult::success(
            run_state
                .settings
                .read()
                .unwrap()
                .get_user_editable_settings(),
        ),
        None => GetSettingsResult::fail(),
    }
}

#[tauri::command]
fn save_settings(
    state: State<'_, GuiState>,
    user: String,
    password: String,
    url: String,
    max_part_size_mb: u32,
) -> Result<String, String> {
    match state.run_state.as_ref() {
        Some(run_state) => {
            let incoming_settings = UserEditableSettings {
                user,
                password,
                server_url: url,
                max_part_size_mb,
            };
            let to_save = {
                let mut settings = run_state.settings.write().unwrap();
                settings.update(incoming_settings);
                settings.clone()
            };
            to_save.save_settings(&state.app_handle)
        }
        None => Err("no run state".to_string()),
    }
}

#[tauri::command]
async fn album_search(
    state: State<'_, GuiState>,
    album: String,
) -> Result<AlbumSearchResponse, String> {
    album_search_inner(state, album)
        .await
        .map_err(|e| format!("Failure: {}", e))
}

async fn album_search_inner(
    state: State<'_, GuiState>,
    album: String,
) -> Result<AlbumSearchResponse, MusicUploaderClientError> {
    // run query to server
    let run_state = state
        .run_state
        .as_ref()
        .ok_or(MusicUploaderClientError::BadConfig(
            "Client did not succesfully boot".to_string(),
        ))?;
    run_state
        .client
        .album_search(&run_state.get_config(), album)
        .await
}

fn result_to_string(result: Result<String, MusicUploaderClientError>) -> Result<String, String> {
    match result {
        Ok(x) => Ok(format!("Success: {}", x)),
        Err(e) => Err(format!("Failure: {}", e)), // this was an ok, not sure if on purpose
    }
}
struct GuiState {
    run_state: Option<RunState>,
    startup_message: String,
    app_handle: AppHandle,
}

struct RunState {
    client: MusicUploaderClient,
    settings: RwLock<Settings>,
}

impl RunState {
    pub fn get_config(&self) -> MusicUploaderClientConfig {
        self.settings.read().unwrap().get_config()
    }
}

const SUCCESS_MESSAGE: &str = "Boot Success :)";
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let logger = GuiLogger::new(app.handle().clone());
            let potential_settings = load_settings(app.handle());
            let state = GuiState {
                startup_message: match &potential_settings {
                    Ok(load_settings_result) => format!(
                        "{}\n{}",
                        SUCCESS_MESSAGE, load_settings_result.startup_message
                    ),
                    Err(fail_message) => fail_message.clone(),
                },
                run_state: potential_settings
                    .ok()
                    .map(|load_settings_result| RunState {
                        client: MusicUploaderClient::new(logger),
                        settings: RwLock::new(load_settings_result.settings),
                    }),
                app_handle: app.handle().clone(),
            };
            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            upload_album,
            generate_guid,
            get_valid_extensions,
            get_startup_message,
            run_settings_checks,
            reload_settings,
            get_settings,
            save_settings,
            album_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
