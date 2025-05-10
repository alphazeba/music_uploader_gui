use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub struct GuiLogger {
    app: AppHandle,
}

impl GuiLogger {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn log(&self, text: String) {
        let _ = self.app.emit("music_uploader://log", text).map_err(|e| {
            println!("GuiLogger.log failed for: {}", e);
            ()
        });
    }

    pub fn file_report(&self, album_id: &String, file: &String, success: bool, message: String) {
        let file_report = FileReport {
            album_id: album_id.to_string(),
            file: file.to_string(),
            success: success,
            message: message.to_string(),
        };
        let _ = match serde_json::to_string(&file_report) {
            Ok(json) => self
                .app
                .emit("music_uploader://file_report", json)
                .map_err(|e| {
                    println!("GuiLogger.file_report failed for: {}", e);
                    ()
                }),
            Err(e) => Ok(self.log(e.to_string())),
        };
    }

    pub fn album_report(&self, album_id: &String, success: bool, message: String) {
        let file_report = AlbumReport {
            album_id: album_id.to_string(),
            success: success,
            message: message.to_string(),
        };
        let _ = match serde_json::to_string(&file_report) {
            Ok(json) => self
                .app
                .emit("music_uploader://album_report", json)
                .map_err(|e| {
                    println!("GuiLogger.album_report failed for: {}", e);
                    ()
                }),
            Err(e) => Ok(self.log(e.to_string())),
        };
    }

    pub fn album_is_uploading(&self, album_id: &String) {
        let _ = self
            .app
            .emit("music_uploader://album_is_uploading", album_id.to_string())
            .map_err(|e| {
                println!("GuiLogger.album_is_uploading failed for: {}", e);
                ()
            });
    }

    pub fn file_is_uploading(&self, album_id: &String, file: &String) {
        let file_is_uploading_payload = FileIsUploading {
            album_id: album_id.to_string(),
            file: file.to_string(),
        };
        let _ = match serde_json::to_string(&file_is_uploading_payload) {
            Ok(json) => self
                .app
                .emit("music_uploader://file_is_uploading", json)
                .map_err(|e| {
                    println!("GuiLogger.file_is_uploading failed for: {}", e);
                    ()
                }),
            Err(e) => Ok(self.log(e.to_string())),
        };
    }
}

#[derive(Serialize)]
struct FileReport {
    album_id: String,
    file: String,
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct AlbumReport {
    album_id: String,
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct FileIsUploading {
    album_id: String,
    file: String,
}
