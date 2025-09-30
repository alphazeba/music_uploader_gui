use std::{collections::HashSet, fs};

use music_uploader_server::model::DeclareUploadResponse;

use crate::{gui_logger::GuiLogger, uploader_client::{MusicUploaderClient, MusicUploaderClientConfig, MusicUploaderClientError}, RunState, Song};

pub async fn upload_song(
    run_state: &RunState,
    logger: &GuiLogger,
    album: &String,
    artist: &String,
    song: &Song,
) -> Result<String, MusicUploaderClientError> {
    let uploader = UploadState::new(run_state, logger, album, artist, song)?;
    match uploader.should_upload_in_parts() {
        true => uploader.send_song_in_parts().await,
        false => uploader.send_song().await,
    }
}

const MAX_MULTIPART_UPLOAD_ATTEMPT: u8 = 2;

struct UploadState<'a> {
    client: &'a MusicUploaderClient,
    config: MusicUploaderClientConfig,
    logger: &'a GuiLogger,
    album: &'a String,
    artist: &'a String,
    song: &'a Song,
    data: Vec<u8>,
}

impl<'a> UploadState<'a> {
    fn new(
        run_state: &'a RunState,
        logger: &'a GuiLogger,
        album: &'a String,
        artist: &'a String,
        song: &'a Song,
    ) -> Result<Self, MusicUploaderClientError> {
        let data = fs::read(&song.path).map_err(|e| {
            MusicUploaderClientError::FileReadError(song.path.to_string(), Box::new(e))
        })?;
        let client = &run_state.client;
        let config = run_state.get_config();
        Ok(Self {
            client,
            config,
            logger,
            album,
            artist,
            song,
            data,
        })
    }

    fn should_upload_in_parts(&self) -> bool {
        let data_bytes = self.data.len() as u32;
        data_bytes > self.config.max_upload_part_size
    }

    async fn send_song(self) -> Result<String, MusicUploaderClientError> {
        self.client
            .send_song(
                &self.config,
                self.data,
                self.artist,
                self.album,
                &self.song.song_name,
            )
            .await
    }

    async fn send_song_in_parts(self) -> Result<String, MusicUploaderClientError> {
        self.logger.log("Starting multipart upload".to_string());
        let hash = sha256::digest(&self.data);
        let declared_size_bytes = self.data.len() as u32;
        for attempt in 0..MAX_MULTIPART_UPLOAD_ATTEMPT {
            match self.declare_upload(
                &hash, 
                self.config.max_upload_part_size,
                declared_size_bytes,
            ).await? {
                DeclareUploadResponse::Complete => return Ok(match attempt {
                    0 => "Song already present".to_string(),
                    n => format!("Succeeded multipart upload on {n} attempt"),
                }),
                DeclareUploadResponse::Incomplete {
                    key,
                    declared_size: _,
                    part_size,
                    received_parts
                } => self.upload_remaining_parts(key, part_size, received_parts).await?,
            }
        }
        Err(MusicUploaderClientError::AlbumUploadFailure(format!(
            "Could not upload after {MAX_MULTIPART_UPLOAD_ATTEMPT} attempts"
        )))
    }

    async fn upload_remaining_parts(
        &self, key: String, part_size: u32, received_parts: Vec<u8>
    ) -> Result<(), MusicUploaderClientError> {
        let received_parts = received_parts.into_iter().collect::<HashSet<_>>();
        let num_parts = self.calculate_num_parts(part_size)?;
        for index in 0..num_parts {
            if received_parts.contains(&index) {
                self.logger.log(format!(
                    "Skipping part {index} because it has already been uploaded"
                ));
                continue;
            }
            let result = self.upload_part(&key, index, part_size as usize).await?;
            self.logger.log(format!("Upload part result: {result}"));
        }
        Ok(())
    }

    fn calculate_num_parts(&self, part_size: u32) -> Result<u8, MusicUploaderClientError> {
        let data_len = self.data.len() as u32;
        let num_whole_parts = data_len / part_size;
        let num_parts = if num_whole_parts * part_size < data_len {
            num_whole_parts + 1
        } else {
            num_whole_parts
        };
        if num_parts > u8::MAX as u32 {
            return Err(MusicUploaderClientError::AlbumUploadFailure(
                "Cannot upload a file with this many parts!!".to_string(),
            ));
        }
        Ok(num_parts as u8)
    }

    async fn declare_upload(
        &self,
        hash: &String,
        part_size_bytes: u32,
        declared_size_bytes: u32,
    ) -> Result<DeclareUploadResponse, MusicUploaderClientError> {
        self.client
            .declare_upload(
                &self.config,
                hash,
                self.artist,
                self.album,
                &self.song.song_name,
                part_size_bytes,
                declared_size_bytes,
            )
            .await
    }

    async fn upload_part(
        &self,
        key: &String,
        index: u8,
        max_part_size: usize,
    ) -> Result<String, MusicUploaderClientError> {
        let start = index as usize * max_part_size;
        let end = usize::min(self.data.len(), (index as usize + 1) * max_part_size);
        if end - start <= 0 {
            return Err(MusicUploaderClientError::AlbumUploadFailure(format!(
                "Tried to upload a zero size part for index: {index}"
            )));
        }
        let data = &self.data[start..end];
        self.client
            .upload_part(&self.config, key, index, data.to_vec())
            .await
    }
}
