use std::{env, fs::File, io::Read};

use serde::Deserialize;
use music_uploader_client::{MusicUploaderClientConfig, MusicUploaderClient};
use tauri::{path::BaseDirectory, App, Manager, State};

#[derive(Deserialize)]
struct Song {
    song_name: String,
    data: Vec<u8>,
}

#[tauri::command]
fn upload_song(
    state: State<'_, GuiState>,
    album: &str,
    artist: &str,
    song: Song
) -> String {
    match upload_song_inner(state, album, artist, song) {
        Ok(x) => format!("Success: {}", x),
        Err(e) => format!("Failure: {}", e),
    }
}

fn upload_song_inner(
    state: State<'_, GuiState>,
    album: &str,
    artist: &str,
    song: Song
) -> Result<String, String> {
    let run_state = state.run_state.as_ref().ok_or("program did not succesfully boot")?;
    run_state.client.upload_song(
        &album.to_string(), 
        &artist.to_string(), 
        &song.song_name, 
        song.data
    ).map_err(|e| format!("error: {}", e))
}

#[tauri::command]
fn generate_guid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tauri::command]
fn get_valid_extensions(state: State<'_, GuiState>) -> Vec<String> {
    state.run_state
        .as_ref()
        .map(|s| s.settings.valid_extensions.clone())
        .unwrap_or(Vec::new())
}

#[tauri::command]
fn album_search(state: State<'_, GuiState>, album: String) -> Result<Vec<String>, String> {
    let run_state = state.run_state.as_ref().ok_or("program did not succesfully boot")?;
    run_state.client.album_search(&album).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_startup_message(state: State<'_, GuiState>) -> String {
    let mut message = state.startup_message.clone();
    if let Some(run_state) = state.run_state.as_ref() {
        match run_state.client.check_conn() {
            Ok(_) => {
                message += "\nConnection is good";
            }
            Err(s) => {
                message += &format!("\nCannot connect with {}: {}", run_state.settings.server_url, s);
            }
        }
        match run_state.client.check_auth() {
            Ok(_) => {
                message += &format!("\nAuthentication valid: hello {}", run_state.settings.user);
            }
            Err(s) => {
                message += &format!("\nAuthentication unsuccesful: {}", s);
            }
        }
    }
    message
}

struct GuiState {
    run_state: Option<RunState>,
    startup_message: String,
}

struct RunState {
    client: MusicUploaderClient,
    settings: Settings,
}

const SUCCESS_MESSAGE: &str = "Boot Success :)";
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let potential_settings = load_settings(app);
            let state = GuiState {
                startup_message: potential_settings
                    .as_ref().err().map(String::to_string).unwrap_or(SUCCESS_MESSAGE.to_string()),
                run_state: potential_settings.ok().map(|settings| RunState {
                    client: MusicUploaderClient::new(get_config(&settings)),
                    settings,
                }),
            };
            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            upload_song,
            generate_guid,
            get_valid_extensions,
            get_startup_message,
            album_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_settings(app: &App) -> Result<Settings, String> {
    let settings_path = app.path().resolve("Settings.toml", BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    let settings_path_str = settings_path.to_str().unwrap_or("no settings path :/");
    let mut f = File::open(&settings_path)
        .map_err(|_| format!("Failed to find {}. Make sure it is present.", settings_path_str))?;
    let mut file_text = String::new();
    let _ = f.read_to_string(&mut file_text)
        .map_err(|_| format!("Failed to read contents of {}. idk what ths menas", settings_path_str))?;
    toml::from_str::<Settings>(&file_text)
        .map_err(|_| format!("Failed to parse contents of {}, probably typo", settings_path_str))
}

#[derive(Deserialize)]
struct Settings {
    user: String,
    password: String,
    valid_extensions: Vec<String>,
    server_url: String,
}

fn get_config(settings: &Settings) -> MusicUploaderClientConfig {
    MusicUploaderClientConfig {
        user: settings.user.clone(),
        password: settings.password.clone(),
        valid_extension: settings.valid_extensions.clone(),
        server_url: settings.server_url.clone(),
    }
}