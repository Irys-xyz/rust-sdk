use std::{str::FromStr, thread::sleep, time::Duration};

use reqwest::{header::ACCEPT, Url};
use serde::{Deserialize, Serialize};

use crate::{
    consts::{CHUNKS_RETRIES, CHUNKS_RETRY_SLEEP, CHUNK_SIZE, DEFAULT_BUNDLER_URL},
    token::TokenType,
    error::BundlerError,
};

#[derive(Serialize, Deserialize)]
struct IdRes {
    id: String,
    max: u64,
    min: u64,
}

pub struct Uploader {
    url: Url,
    client: reqwest::Client,
    pub upload_id: Option<String>,
    token: TokenType,
    chunk_size: u64,
}

impl Default for Uploader {
    fn default() -> Self {
        let url = Url::from_str(DEFAULT_BUNDLER_URL).unwrap(); //Unwrap ok, never fails
        let client = reqwest::Client::new();
        Self {
            url,
            client,
            upload_id: None,
            token: TokenType::Arweave,
            chunk_size: CHUNK_SIZE,
        }
    }
}

impl Uploader {
    pub fn new(url: Url, client: reqwest::Client, token: TokenType) -> Self {
        Uploader {
            url,
            client,
            upload_id: None,
            token,
            chunk_size: CHUNK_SIZE,
        }
    }

    pub async fn upload(&mut self, _data: Vec<u8>) -> Result<(), BundlerError> {
        let (max, min) = if let Some(upload_id) = self.upload_id.clone() {
            let url = self
                .url
                .join(&format!("/chunks/{}/{}/-1", self.token, upload_id))
                .map_err(|err| BundlerError::ParseError(err.to_string()))?;
            let res = self
                .client
                .get(url)
                .header("x-chunking-version", "2")
                .send()
                .await
                .map_err(|err| BundlerError::UploadError(err.to_string()))?
                .json::<IdRes>()
                .await
                .map_err(|err| BundlerError::ParseError(err.to_string()))?;

            (res.max, res.min)
        } else {
            let url = self
                .url
                .join(&format!("/chunks/{}/-1/-1", self.token))
                .map_err(|err| BundlerError::ParseError(err.to_string()))?;
            let res = self
                .client
                .get(url)
                .header("x-chunking-version", "2")
                .send()
                .await
                .map_err(|err| BundlerError::UploadError(err.to_string()))?
                .json::<IdRes>()
                .await
                .map_err(|err| BundlerError::ParseError(err.to_string()))?;

            self.upload_id = Some(res.id);

            (res.max, res.min)
        };

        if self.chunk_size < min || self.chunk_size > max {
            return Err(BundlerError::ChunkSizeOutOfRange(min, max));
        }

        Ok(())
    }

    /*
    fn upload_transaction_chunks_stream<'a>(
        uploader: &'a Uploader,
        chunks: Vec<Vec<u8>>,
        buffer: usize,
    ) -> impl Stream<Item = Result<usize, BundlerError>> + 'a {
        stream::iter(0..chunks.len())
            .map(move |i| {
                let chunk = chunks[i].clone();
                uploader.post_chunk_with_retries(chunk, 0, vec![])
            })
            .buffer_unordered(buffer)
    }
    */

    pub async fn post_chunk_with_retries(
        &self,
        chunk: Vec<u8>,
        offset: usize,
        headers: Vec<(String, String)>,
    ) -> Result<usize, BundlerError> {
        let mut retries = 0;
        let mut resp = self.post_chunk(&chunk, offset, headers.clone()).await;

        while retries < CHUNKS_RETRIES {
            match resp {
                Ok(offset) => return Ok(offset),
                Err(e) => {
                    dbg!("post_chunk_with_retries: {:?}", e);
                    sleep(Duration::from_secs(CHUNKS_RETRY_SLEEP));
                    retries += 1;
                    resp = self.post_chunk(&chunk, offset, headers.clone()).await;
                }
            }
        }
        resp
    }

    pub async fn post_chunk(
        &self,
        chunk: &[u8],
        offset: usize,
        headers: Vec<(String, String)>,
    ) -> Result<usize, BundlerError> {
        let upload_id = match &self.upload_id {
            Some(id) => id,
            None => return Err(BundlerError::UploadError("No upload id".to_string())),
        };
        let url = self
            .url
            .join(&format!(
                "/chunks/{}/{}/{}",
                self.token, upload_id, offset
            ))
            .map_err(|err| BundlerError::ParseError(err.to_string()))?;

        let mut req = self
            .client
            .post(url)
            .json(&chunk)
            .header(&ACCEPT, "application/json");
        for (header, value) in headers {
            req = req.header(header, value);
        }

        let res = req
            .send()
            .await
            .map_err(|e| BundlerError::PostChunkError(e.to_string()))?;

        match res.status() {
            reqwest::StatusCode::OK => Ok(offset),
            err => Err(BundlerError::RequestError(err.to_string())),
        }
    }
}
