use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::{path::BaseDirectory, AppHandle, Manager};

use crate::uploader_client::MusicUploaderClientConfig;

const SETTINGS_FILE_NAME: &str = "Settings.toml";

const DEFAULT_PART_SIZE_MB: u32 = 5;
const MEGABYTE_BYTES: u32 = 1_000_000;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub user: String,
    password: String,
    pub valid_extensions: Vec<String>,
    pub server_url: String,
    pub max_part_size_mb: Option<u32>,
}

impl Settings {
    pub fn get_config(&self) -> MusicUploaderClientConfig {
        MusicUploaderClientConfig {
            user: self.user.clone(),
            password: self.password.clone(),
            server_url: self.server_url.clone(),
            max_upload_part_size: self.max_part_size_mb.unwrap_or(DEFAULT_PART_SIZE_MB) * MEGABYTE_BYTES,
        }
    }

    pub fn get_user_editable_settings(&self) -> UserEditableSettings {
        UserEditableSettings {
            user: self.user.clone(),
            password: self.password.clone(),
            server_url: self.server_url.clone(),
            max_part_size_mb: self.max_part_size_mb.unwrap_or(DEFAULT_PART_SIZE_MB)
        }
    }

    pub fn update(&mut self, user_editable_settings: UserEditableSettings) {
        self.user = user_editable_settings.user;
        self.password = user_editable_settings.password;
        self.server_url = user_editable_settings.server_url;
        self.max_part_size_mb = Some(user_editable_settings.max_part_size_mb);
    }

    pub fn save_settings(&self, app: &AppHandle) -> Result<String, String> {
        let settings_path = get_settings_path(app)?;
        let stringified_settings = toml::to_string(self).map_err(|e| e.to_string())?;
        let mut f = File::create(&settings_path).map_err(|e| e.to_string())?;
        let () = f
            .write_all(stringified_settings.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(format!(
            "Succesfully wrote settings to {}",
            path_string(&settings_path)
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserEditableSettings {
    pub user: String,
    pub password: String,
    pub server_url: String,
    pub max_part_size_mb: u32,
}

pub struct LoadSettingsResult {
    pub settings: Settings,
    pub startup_message: String,
}

// i dislike the current mechanism i have for passing messages to the user
// however, the gui listener for rust log events has not been added by the
// point that the tuari app is being configured.
pub fn load_settings(app: &AppHandle) -> Result<LoadSettingsResult, String> {
    let settings_path = get_settings_path(app)?;
    let mut success_message = format!("looking for settings at ({})", path_string(&settings_path));
    // likely first time running, create the settings directory and copy the default settings over.
    if !fs::exists(&settings_path).unwrap_or(false) {
        let config_dir = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("failed to get app config dir: {}", e))?;
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("failed to find ({}) so tried to create the directory ({}) to create it, but failed: {}",
                path_string(&settings_path), path_string(&config_dir), e))?;
        let example_settings_path = app
            .path()
            .resolve(SETTINGS_FILE_NAME, BaseDirectory::Resource)
            .map_err(|e| e.to_string())?;
        let _ = fs::copy(&example_settings_path, &settings_path)
            .map_err(|e| format!("failed to open settings ({}) so tried copying example settings over ({}), but failed: {}",
                path_string(&settings_path),
                path_string(&example_settings_path),
                e))?;
        success_message = format!("{success_message}\nHello, this looks like your first time using music uploader! You will need to configure your settings to talk to your server. Click the gear icon in top right to configure client settings. ");
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
        .map(|x| LoadSettingsResult {
            settings: x,
            startup_message: success_message,
        })
        .map_err(|_| {
            format!(
                "Failed to parse contents of {}, probably typo",
                path_string(&settings_path)
            )
        })
}

fn get_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resolve(SETTINGS_FILE_NAME, BaseDirectory::AppConfig)
        .map_err(|e| e.to_string())
}

fn path_string(path: &Path) -> String {
    path.to_str().unwrap_or("<no path>").to_string()
}
