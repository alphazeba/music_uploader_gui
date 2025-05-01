mod gui_logger;
mod uploader_client;

use gui_logger::GuiLogger;
use serde::Deserialize;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::Path,
};
use tauri::{path::BaseDirectory, App, AppHandle, Manager, State};
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
                Err(e) => e.to_string()
            });
        results.push(result);
    }
    let total_result = get_album_upload_result(results);
    logger.album_report(&album_id, total_result.is_ok(), match &total_result {
        Ok(message) => message.to_string(),
        Err(e) => e.to_string(),
    });
    total_result?;
    trigger_scan_inner(state).await
}

async fn send_song(run_state: &RunState, album: &String, artist: &String, song: &Song) -> Result<String, MusicUploaderClientError> {
    let data = fs::read(&song.path)
        .map_err(|e| MusicUploaderClientError::FileReadError(song.path.to_string(), Box::new(e)))?;
    let result = run_state
        .client
        .send_song(
            data,
            &artist,
            &album,
            &song.song_name,
        ).await;
    // send report to the gui.
    result
}

fn get_album_upload_result(upload_results: Vec<Result<String, MusicUploaderClientError>>) -> Result<String, MusicUploaderClientError> {
    for result in upload_results {
        match result {
            Ok(_) => continue,
            // i do not log error because i am assuming that a file report was generated for this file already.
            Err(_) => return Err(MusicUploaderClientError::AlbumUploadFailure("At least one song failed to upload".to_string())),
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
        .map(|s| s.settings.valid_extensions.clone())
        .unwrap_or(Vec::new())
}

#[tauri::command]
async fn album_search(state: State<'_, GuiState>, album: String) -> Result<Vec<String>, String> {
    let run_state = state
        .run_state
        .as_ref()
        .ok_or("program did not succesfully boot")?;
    run_state
        .client
        .album_search(&album)
        .await
        .map(|response| response.albums)
        .map_err(|e| e.to_string())
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
    let result = run_state.client.trigger_scan().await;
    println!("finished triggering scan: {:?}", result);
    result
}

#[tauri::command]
async fn get_startup_message(state: State<'_, GuiState>) -> Result<String, String> {
    let mut message = state.startup_message.clone();
    if let Some(run_state) = state.run_state.as_ref() {
        match run_state.client.check_conn().await {
            Ok(_) => {
                message += "\nConnection is good";
            }
            Err(s) => {
                message += &format!(
                    "\nCannot connect with {}: {}",
                    run_state.settings.server_url, s
                );
            }
        };
        match run_state.client.check_auth().await {
            Ok(_) => {
                message += &format!("\nAuthentication valid: hello {}", run_state.settings.user);
            }
            Err(s) => {
                message += &format!("\nAuthentication unsuccesful: {}", s);
            }
        };
    }
    Ok(message)
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
}

struct RunState {
    client: MusicUploaderClient,
    settings: Settings,
}

const SUCCESS_MESSAGE: &str = "Boot Success :)";
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let logger = GuiLogger::new(app.handle().clone());
            let potential_settings = load_settings(app);

            let state = GuiState {
                startup_message: match &potential_settings {
                    Ok((_, success_message)) => format!("{SUCCESS_MESSAGE}\n{success_message}"),
                    Err(fail_message) => fail_message.clone(),
                },
                run_state: potential_settings.ok().map(|(settings, _)| RunState {
                    client: MusicUploaderClient::new(get_config(&settings), logger),
                    settings,
                }),
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
            album_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

const SETTINGS_FILE_NAME: &str = "Settings.toml";

// i dislike the current mechanism i have for passing messages to the user
// however, the gui listener for rust log events has not been added by the 
// point that the tuari app is being configured.
fn load_settings(app: &App) -> Result<(Settings, String), String> {
    let settings_path = app.path()
        .resolve(SETTINGS_FILE_NAME, BaseDirectory::AppConfig)
        .map_err(|e| e.to_string())?;
    let mut success_message = format!("looking for settings at ({})", path_string(&settings_path));
    // likely first time running, create the settings directory and copy the default settings over.
    if !fs::exists(&settings_path).unwrap_or(false) {
        let config_dir = app.path().app_config_dir().map_err(|e| format!("failed to get app config dir: {}", e))?;
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("failed to find ({}) so tried to create the directory ({}) to create it, but failed: {}",
                path_string(&settings_path), path_string(&config_dir), e))?;
        let example_settings_path = app.path()
            .resolve(SETTINGS_FILE_NAME, BaseDirectory::Resource)
            .map_err(|e| e.to_string())?;
        let _ = fs::copy(&example_settings_path, &settings_path)
            .map_err(|e| format!("failed to open settings ({}) so tried copying example settings over ({}), but failed: {}",
                path_string(&settings_path),
                path_string(&example_settings_path),
                e))?;
        success_message = format!("{success_message}\nHello, this looks like your first time using music uploader! You will need to configure your settings to talk to your server. Find settings here ({:?}) ", &settings_path);
    }
    let mut f = File::open(&settings_path).map_err(|_| {
        format!(
            "Failed to find {}. Make sure it is present.",
            path_string(&settings_path)
        )
    })?;
    let mut file_text = String::new();
    let _ = f.read_to_string(&mut file_text).map_err(|_| {
        format!(
            "Failed to read contents of {}. idk what ths menas",
            path_string(&settings_path)
        )
    })?;
    toml::from_str::<Settings>(&file_text)
        .map(|x| (x, success_message))
        .map_err(|_| {
            format!(
                "Failed to parse contents of {}, probably typo",
                path_string(&settings_path)
            )
        })
}

fn path_string(path: &Path) -> String {
    path.to_str().unwrap_or("<no path>").to_string()
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
        server_url: settings.server_url.clone(),
    }
}
