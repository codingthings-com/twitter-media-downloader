//! _twitter-media-downloader_ main file
use std::path::PathBuf;

use clap::{ArgAction, Parser};
use env_logger::Env;
use log::{error, info};

use crate::common::Config;

pub mod common;
pub mod twitter;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArguments {
    /// Bearer Token. Can be passed as BEARER_TOKEN
    #[clap(short, long, value_parser, env)]
    bearer_token: String,

    /// Twitter handle - username
    #[clap(short, long, value_parser)]
    username: String,

    /// Number of media files to download in a batch
    #[clap(short, long, value_parser, default_value_t = 100)]
    count: u8,

    /// Reset the download marker to the latest tweet
    #[clap(short, long, action = ArgAction::SetTrue)]
    reset_marker: bool,

    /// Scan and download all photos of the user (-u ). Skips already downloaded files. Use with --reset-marker to reset to the latest tweet
    #[clap(short, long, action = ArgAction::SetTrue)]
    download_all: bool,

    /// Output directory
    #[clap(short, long, value_parser, default_value = ".")]
    output_dir: PathBuf,
}


#[tokio::main]
/// Parses the command line arguments into the `Config` object and upon validation starts the download
/// [twitter::start_download](twitter::start_download)
async fn main() {
    // set the env for logging
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    // parse the command line args
    let args = CliArguments::parse();

    // create the basic common to be passed around
    let config = Config {
        bearer_token: args.bearer_token,
        username: args.username,
        count: args.count,
        reset_marker: args.reset_marker,
        download_all: args.download_all,
        output_dir: args.output_dir,
    };

    info!("username: {}. Starting downloading media files", config.username );

    match twitter::start_download(config).await {
        Ok(s) => info!("{}", s),
        Err(e) => error!("{}", e)
    }
    info!("Exiting.")
}
