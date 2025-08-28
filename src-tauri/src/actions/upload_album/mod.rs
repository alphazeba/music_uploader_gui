mod upload_logic;

use crate::gui_logger::GuiLogger;
use crate::uploader_client::MusicUploaderClientError;
use crate::{result_to_string, GuiState, Song};
use tauri::{AppHandle, State};
use upload_logic::upload_song;

#[tauri::command]
pub async fn upload_album(
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
        let result = upload_song(run_state, &logger, &album_name, &artist, song).await;
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
