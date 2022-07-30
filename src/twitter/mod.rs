//! module to handle downloading media files for the Twitter user.
use std::{io, thread};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use log::{error, info, warn};
use twitter_v2::{Media, TwitterApi};
use twitter_v2::authorization::BearerToken;
use twitter_v2::data::{Expansions, MediaType};
use twitter_v2::query::{Exclude, MediaField, TweetExpansion, TweetField};

use crate::Config;


/// Name of the checkpoint file. Checkpoint file stores the tweet id of the oldest tweet processed the application
const CHECKPOINT_FILENAME: &str = "checkpoint";

/// Give it some time during iterations of get_user_tweets
const SLEEP_TIME: Duration = Duration::from_millis(250);

/// Gets this show on the road.
///
/// If `Config::download_all` is true keeps looping with the call [download_media](download_media) until there are no more Tweets. `marker` is read from [get_checkpoint](get_checkpoint).
/// The checkpoint file [update_checkpoint](update_checkpoint) is updated during iterations.
///
/// If `Config::download_all` is false, breaks after first call.
///
/// Returns Ok with count info or Error.
pub async fn start_download(config: Config) -> Result<String, Box<dyn Error>> {
    let api = TwitterApi::new(BearerToken::new(&config.bearer_token));

    let id = get_twitter_id(&api, &config).await?;

    let mut reset_once = config.reset_marker;

    let user_output_dir = get_user_output_dir(&config.output_dir, &config.username).unwrap();
    let user_checkpoint_file_path = get_user_checkpoint_file_path(&user_output_dir).unwrap();

    info!("username: {}, output_dir: {}", &config.username, &user_output_dir.into_os_string().into_string().unwrap());
    let mut total_count: u32 = 0;
    loop {
        let checkpoint = get_checkpoint(&user_checkpoint_file_path, reset_once)?;
        reset_once = false;

        if checkpoint == 0 {
            info!("username: {}, checkpoint: {}. All media files are downloaded. Consider --reset-marker if you want to start from latest.", config.username, checkpoint);
            return Ok("Ok".into());
        }

        info!("username: {}, checkpoint: {}. Will get media for tweets", &config.username, checkpoint);

        match download_media(&api, &config, id, checkpoint).await {
            Ok((mut oldest_id, count)) => {
                total_count += count;

                oldest_id = update_checkpoint(&user_checkpoint_file_path, &oldest_id).unwrap();

                info!("username: {}, oldest_id: {}. Downloaded {} files for tweets", &config.username, oldest_id, count);

                if !config.download_all {
                    break;
                }
                info!("username: {}, checkpoint: {}. Resetting checkpoint and resting a bit. Will continue...", config.username, oldest_id);
                thread::sleep(SLEEP_TIME);
            }
            Err(err) => {
                warn!("{}", err);
                break;
            }
        }
    }
    return Ok(format!("Download complete. {} files downloaded.", total_count).into());
}

/// Ensures that the user's output directory is present.
///
/// User's media will be stored under `output_dir`/`name`
fn get_user_output_dir(output_dir: &PathBuf, username: &str) -> Result<PathBuf, io::Error> {
    let mut path = PathBuf::new();
    path.push(output_dir);
    path.push(username);

    let mut builder = DirBuilder::new();
    builder.recursive(true);

    return match builder.create(&path) {
        Ok(..) => Ok(path),
        Err(err) => Err(err)
    };
}

/// Returns the path to the user's checkpoint file.
/// Checkpoint file stores the tweet id of the oldest tweet processed the application
fn get_user_checkpoint_file_path(user_output_dir: &PathBuf) -> Result<PathBuf, io::Error> {
    let mut path = PathBuf::new();
    path.push(user_output_dir);
    path.push(CHECKPOINT_FILENAME);

    Ok(path)
}

/// Reads the checkpoint file `user_checkpoint_file_path` and returns the value as u64. Value is a Tweet::id
///
/// If `reset_marker` is true update the `user_checkpoint_file_path` with u64::MAX value and return u64::MAX
///
/// If `user_checkpoint_file_path` does not exists, create the `user_checkpoint_file_path` with u64::MAX value and return u64::MAX
///
/// If `user_checkpoint_file_path` exists, read the contents and return the value
fn get_checkpoint(user_checkpoint_file_path: &PathBuf, reset_marker: bool) -> Result<u64, io::Error> {
    return if reset_marker || !Path::new(&user_checkpoint_file_path).exists() {
        let _ = update_checkpoint(user_checkpoint_file_path, u64::MAX.to_string().as_str())?;
        Ok(u64::MAX)
    } else {
        let contents: String = fs::read_to_string(&user_checkpoint_file_path)?;
        Ok(contents.parse::<u64>().unwrap_or(u64::MAX))
    };
}

/// Updates the `user_checkpoint_file_path` file with the given `checkpoint` value. Value is a Tweet::id
///
/// Returns the `checkpoint` untouched.
fn update_checkpoint(user_checkpoint_file_path: &PathBuf, checkpoint: &str) -> Result<String, io::Error> {
    let mut file = File::create(user_checkpoint_file_path)?;
    file.write(checkpoint.as_bytes())?;
    file.sync_all()?;
    Ok(checkpoint.into())
}

/// Calls [TwitterApi::get_user_by_username](TwitterApi::get_user_by_username) to retrieve `u64` userid associated with Twitter username
///
/// Returns Error is any error occurs or Twitter user does not exist.
async fn get_twitter_id(api: &TwitterApi<BearerToken>, config: &Config) -> Result<u64, Box<dyn Error>> {
    let username: &str = &(config.username);

    if username.len() == 0 {
        return Err("username is required to lookup user id".into());
    }

    let user = api.get_user_by_username(username)
        .send()
        .await?;

    if let Some(data) = user.into_data() {
        let id = data.id.as_u64();
        if id > 0 {
            info!("username {}, id: {}", username, id);
            return Ok(id);
        }
    }

    return Err(format!("Cannot find id for username {}", username).into());
}

/// Retrieves Tweets for the user extracts the `Media` info and triggers the download the files locally.
///
/// Get `Config::count` Tweets for `Config::username` until the `marker` Tweet id.
///
/// Check if there is Media associated with the Tweet. If there is a `Media::Photo` then [download_url](download_url)
///
/// If the file is not downloaded because it exists, check the `Config::download_all` parameter to decide to bail iteration or not.
/// If the file exists and `Config::download_all` is false, there is no need to iterate the rest because we most like got them during previous runs of the program.
/// If [download_url](download_url) fails, log the error keep iterating the tweets, do not bail.
///
/// Returns a tuple for `oldest_id` for the id of the last(actually earliest) Tweet id and the a counter for the successfully downloaded files.
///
/// Or returns an Error.
async fn download_media(api: &TwitterApi<BearerToken>, config: &Config, id: u64, marker: u64) -> Result<(String, u32), Box<dyn Error>> {
    let mut count: u32 = 0;
    let user_output_dir = get_user_output_dir(&config.output_dir, &config.username)?;

    let mut req_tweets = api.get_user_tweets(id);

    req_tweets
        .max_results(config.count.into())
        .exclude([Exclude::Replies, Exclude::Retweets])
        .media_fields([MediaField::Url, MediaField::Type])
        .tweet_fields(
            [TweetField::AuthorId,
                TweetField::CreatedAt,
                TweetField::Attachments,
                TweetField::Entities,
                TweetField::Text
            ])
        .expansions([TweetExpansion::AttachmentsMediaKeys, ]);

    if marker != u64::MAX {
        req_tweets.until_id(marker);
    }

    let tweets_response = req_tweets.send().await?;
    let tweets_data = tweets_response.clone().into_data();


    match tweets_data {
        Some(td) => {
            let tweets_includes = tweets_response.clone().into_includes();
            let media_map = generate_media_map(tweets_includes);
            for tweet in td.iter() {
                if let Some(attachments) = &tweet.attachments {
                    if let Some(media_keys) = &attachments.media_keys {
                        for media_key in media_keys.iter() {
                            if let Some(media) = media_map.get(&media_key.to_string()) {
                                if media.kind == MediaType::Photo {
                                    let downloaded = download_url(&config.username, &user_output_dir, media).await;
                                    match downloaded {
                                        Ok(d) => {
                                            if d {
                                                count = count + 1;
                                            } else if !config.download_all {
                                                warn!("username: {}. File exists. Bailing because we most likely downloaded the rests of the media already. Use --download_all option to go through all tweets", &config.username);
                                                return Ok((tweet.id.to_string(), count));
                                            }
                                        }
                                        Err(e) => {
                                            error!("{}", e.to_string());
                                            continue;
                                        }
                                    } // end downloaded or not
                                } // end this is a photo
                            } // end matched the tweet's mediakey in the media_map
                        } // end loop attachments.media_keys
                    } // end has attachments.media_keys
                } // end has attachments
            } // end loop tweets
        }
        None => () // let this be handled by the return section below
    } // end no tweets returned

    let tweets_meta = tweets_response.clone().into_meta();

    return match tweets_meta {
        Some(meta) => {
            if let Some(oldest_id) = meta.oldest_id {
                Ok((oldest_id, count))
            } else {
                Err(format!("username: {}. No more tweets", &config.username).into())
            }
        }
        _ => Err(format!("username: {}. Cannot access Tweets Meta. Something is up!", &config.username).into())
    };
}

/// Create a hashmap of media_keys to Media objects in order to help locate the Media objects which are
/// referred by media_key in the Tweet responses.
fn generate_media_map(expansions: Option<Expansions>) -> HashMap<String, Media> {
    let mut media_map: HashMap<String, Media> = HashMap::new();

    if let Some(e) = expansions {
        if let Some(media) = e.media {
            for m in media.iter() {
                media_map.insert(m.media_key.to_string(), m.clone());
            }
        }
    }

    media_map
}

/// Download the Media::url into user's output directory.
///
/// If the file exists, return false
///
/// If any error occurs, return the Error.
async fn download_url(username: &String, user_output_dir: &PathBuf, media: &Media) -> Result<bool, Box<dyn Error>> {
    return match &media.url {
        Some(u) => {
            let url = u.clone();

            let filename = url.path().split("/").last().unwrap_or("");
            let local_filename = format!("{}_{}_{}", media.media_key.to_string(), username, filename);

            let mut output_file = PathBuf::new();
            output_file.push(user_output_dir);
            output_file.push(&local_filename);

            if !Path::new(&output_file).exists() {
                let resp = reqwest::get(url.clone()).await?.bytes().await?;
                let mut out = File::create(output_file)?;
                out.write_all(&*resp)?;

                info!("username: {}, media_key: {}, remote: {}, local: {}. Downloaded", username, media.media_key.as_str(), url, &local_filename);
                Ok(true)
            } else {
                warn!("username: {}, media_key: {}, remote: {}, local: {}. File exists, skipping.", username, media.media_key.as_str(), url, &local_filename);
                Ok(false)
            }
        }
        None => Err("Media url not available.".into())
    };
}

