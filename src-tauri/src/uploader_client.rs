use music_uploader_server::model::{from_json, AlbumSearchResponse};
use reqwest::{Client, RequestBuilder, Response};
use serde::Deserialize;
use thiserror::Error;

use crate::gui_logger::GuiLogger;

pub struct MusicUploaderClientConfig {
    pub user: String,
    pub password: String,
    pub server_url: String,
}

pub struct MusicUploaderClient {
    config: MusicUploaderClientConfig,
    client: Client,
    logger: GuiLogger,
}

impl MusicUploaderClient {
    pub fn new(config: MusicUploaderClientConfig, logger: GuiLogger) -> Self {
        MusicUploaderClient {
            config,
            client: Client::new(),
            logger,
        }
    }

    pub async fn check_conn(&self) -> Result<String,MusicUploaderClientError> {
        let result = self.client.get(self.build_url("conn"))
            .send().await;
        handle_string_response(result).await
    }
    
    pub async fn check_auth(&self) -> Result<String,MusicUploaderClientError> {
        let result = self.apply_auth(self.client.get(self.build_url("auth")))
            .send().await;
        handle_string_response(result).await
    }
    
    pub async fn send_song(
        &self,
        file: Vec<u8>,
        artist: &String,
        album: &String,
        song_file_name: &String
    ) -> Result<String, MusicUploaderClientError> {
        self.log("hashing".to_string());
        let song_hash = sha256::digest(&file);
        self.log("building request".to_string());
        let request = self.client.post(self.build_url("upload"))
            .header("file", song_file_name)
            .header("album", album)
            .header("artist", artist)
            .header("hash", song_hash)
            .body(file);
        self.log("sending request".to_string());
        let result = self.apply_auth(request).send().await;
        self.log("received response".to_string());
        handle_string_response(result).await
    }
    
    pub async fn album_search(&self, album: &String) -> Result<AlbumSearchResponse, MusicUploaderClientError> {
        let request = self.client.get(self.build_url(&format!("albumsearch/{}", album)));
        let result = self.apply_auth(request).send().await;
        handle_response::<AlbumSearchResponse>(result).await
    }
    
    pub async fn trigger_scan(
        &self
    ) -> Result<String, MusicUploaderClientError> {
        let result=  self.apply_auth(self.client.post(self.build_url("triggerscan")))
            .send().await;
        handle_string_response(result).await
    }

    fn apply_auth(&self, request_builder: RequestBuilder) -> RequestBuilder {
        request_builder.basic_auth(
            self.config.user.clone(),
            Some(self.config.password.clone()))
    }

    fn build_url(&self, route: &str) -> String {
        format!("{}/{}", self.config.server_url, route)
    }

    fn log(&self, text: String) {
        self.logger.log(text);
    }
}

#[derive(Error, Debug)]
pub enum MusicUploaderClientError {
    #[error("unhappy response: ({0}) {1}")]
    UnhappyResponse(u16, String),
    #[error("Recieved an error from the server: {0}")]
    ErrorFromServer(String),
    #[error("Failed to parse server response: {0}")]
    ParseServerResponseFailure(String),
    #[error("Local settings is misconfigured: {0}")]
    BadConfig(String),
}

async fn handle_response<T: for<'a> Deserialize<'a>> (result: Result<Response, reqwest::Error>) -> Result<T, MusicUploaderClientError> {
    handle_string_response(result).await.and_then(|s| 
        from_json::<T>(&s)
            .map_err(|e| MusicUploaderClientError::ParseServerResponseFailure(e.to_string())))
}

async fn handle_string_response(result: Result<Response, reqwest::Error>) -> Result<String, MusicUploaderClientError> {
    match result {
        Ok(response) => {
            match response.status().is_success() {
                true => Ok(get_body(response).await),
                false => Err(MusicUploaderClientError::UnhappyResponse(response.status().as_u16(), get_body(response).await)),
            }
        }
        Err(e) => {
            println!("sending failed, if source error is disconnected this is likely an authorization issue");
            println!("sending error: {:?}", e);
            Err(MusicUploaderClientError::ErrorFromServer(e.to_string()))
        }
    }
}

async fn get_body(response: Response) -> String {
    response.text().await.unwrap_or_else(|_| "<no body>".to_string())
}