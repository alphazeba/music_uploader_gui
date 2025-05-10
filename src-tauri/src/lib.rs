mod gui_logger;
mod uploader_client;
mod settings;

use gui_logger::GuiLogger;
use serde::{Deserialize, Serialize};
use settings::{load_settings, Settings, UserEditableSettings};
use std::{
    env,
    fs, sync::RwLock,
};
use tauri::{AppHandle, Manager, State};
use uploader_client::{MusicUploaderClient, MusicUploaderClientConfig, MusicUploaderClientError};

#[derive(Deserialize)]
struct Song {
    song_name: String,
    path: String,
}

#[tauri::command]
async fn upload_album(
    app: AppHandle,
    state: State<'_, GuiState>,
    album_name: &str,
    album_id: &str,
    artist: &str,
    songs: Vec<Song>,
) -> Result<String, String> {
    result_to_string(upload_album_inner(app, state, album_name, album_id, artist, songs).await)
}

async fn upload_album_inner(
    app: AppHandle,
    state: State<'_, GuiState>,
    album_name: &str,
    album_id: &str,
    artist: &str,
    songs: Vec<Song>,
) -> Result<String, MusicUploaderClientError> {
    let logger = GuiLogger::new(app);
    let album_id = album_id.to_string();
    let album_name = album_name.to_string();
    let artist = artist.to_string();
    logger.log("gui backend received album upload request".to_string());
    logger.album_is_uploading(&album_id);
    let run_state = state
        .run_state
        .as_ref()
        .ok_or(MusicUploaderClientError::BadConfig(
            "Client did not succesfully boot".to_string(),
        ))?;
    let mut results: Vec<Result<String, MusicUploaderClientError>> = Vec::new();
    for song in songs.iter() {
        logger.file_is_uploading(&album_id, &song.path);
        let result = send_song(run_state, &album_name, &artist, song).await;
        logger.file_report(
            &album_id,
            &song.path,
            result.is_ok(),
            match &result {
                Ok(message) => message.to_string(),
                Err(e) => e.to_string(),
            },
        );
        results.push(result);
    }
    let total_result = get_album_upload_result(results);
    logger.album_report(
        &album_id,
        total_result.is_ok(),
        match &total_result {
            Ok(message) => message.to_string(),
            Err(e) => e.to_string(),
        },
    );
    total_result?;
    trigger_scan_inner(state).await
}

async fn send_song(
    run_state: &RunState,
    album: &String,
    artist: &String,
    song: &Song,
) -> Result<String, MusicUploaderClientError> {
    let data = fs::read(&song.path)
        .map_err(|e| MusicUploaderClientError::FileReadError(song.path.to_string(), Box::new(e)))?;
    let result = run_state
        .client
        .send_song(&run_state.get_config(), data, &artist, &album, &song.song_name)
        .await;
    // send report to the gui.
    result
}

fn get_album_upload_result(
    upload_results: Vec<Result<String, MusicUploaderClientError>>,
) -> Result<String, MusicUploaderClientError> {
    for result in upload_results {
        match result {
            Ok(_) => continue,
            // i do not log error because i am assuming that a file report was generated for this file already.
            Err(_) => {
                return Err(MusicUploaderClientError::AlbumUploadFailure(
                    "At least one song failed to upload".to_string(),
                ))
            }
        }
    }
    Ok("All files in album uploaded succesfully".to_string())
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

async fn trigger_scan_inner(
    state: State<'_, GuiState>,
) -> Result<String, MusicUploaderClientError> {
    let run_state = state
        .run_state
        .as_ref()
        .ok_or(MusicUploaderClientError::BadConfig(
            "Client did not succesfully boot".to_string(),
        ))?;
    println!("stargin trigger scan");
    let result = run_state.client.trigger_scan(&run_state.get_config()).await;
    println!("finished triggering scan: {:?}", result);
    result
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
                logger.log(format!(
                    "Cannot connect with {}: {}",
                    server_url, s
                ));
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
        GetSettingsResult { settings: None, success: false }
    }
}

#[tauri::command]
fn get_settings(state: State<'_, GuiState>) -> GetSettingsResult {
    match state.run_state.as_ref() {
        Some(run_state) =>  GetSettingsResult::success(run_state.settings
            .read()
            .unwrap()
            .get_user_editable_settings()),
        None => GetSettingsResult::fail(),
    }
}

#[tauri::command]
fn save_settings(state: State<'_, GuiState>, user: String, password: String, url: String) -> Result<String, String> {
    match state.run_state.as_ref() {
        Some(run_state) => {
            let incoming_settings = UserEditableSettings {
                user,
                password,
                server_url: url
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

fn result_to_string(result: Result<String, MusicUploaderClientError>) -> Result<String, String> {
    match result {
        Ok(x) => Ok(format!("Success: {}", x)),
        Err(e) => Ok(format!("Failure: {}", e)),
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
                    Ok(load_settings_result) => format!("{}\n{}", SUCCESS_MESSAGE, load_settings_result.startup_message),
                    Err(fail_message) => fail_message.clone(),
                },
                run_state: potential_settings.ok().map(|load_settings_result| RunState {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

