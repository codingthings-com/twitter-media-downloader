//! module to hold common structs for `twitter-media-downloader`
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub bearer_token: String,
    pub username: String,
    pub count: u8,
    pub reset_marker: bool,
    pub download_all: bool,
    pub output_dir: PathBuf,
}

