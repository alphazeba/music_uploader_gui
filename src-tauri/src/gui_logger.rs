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
}
