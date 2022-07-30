# twitter-media-downloader

`twitter-media-downloader` downloads photos from "public" Twitter accounts to your local disk. 

```shell
twitter-media-downloader 0.1.1
hello@codingthings.com
Downloads photos from "public" Twitter accounts to your local disk.

USAGE:
    twitter-media-downloader [OPTIONS] --bearer-token <BEARER_TOKEN> --username <USERNAME>

OPTIONS:
    -b, --bearer-token <BEARER_TOKEN>
            Bearer Token. Can be passed as BEARER_TOKEN [env: BEARER_TOKEN=]

    -u, --username <USERNAME>
            Twitter handle - username

    -c, --count <COUNT>
            Number of media files to download in a batch [default: 100]

    -d, --download-all
            Scan and download all photos of the user (-u ). Skips already downloaded files.
            Use with --reset-marker to reset to the latest tweet

    -h, --help
            Print help information

    -o, --output-dir <OUTPUT_DIR>
            Output directory [default: .]

    -r, --reset-marker
            Reset the download marker to the latest tweet

    -V, --version
            Print version information

```

## Development
Built with rustc 1.59.0 (9d1b2106e 2022-02-23). Have your Rust development env ready [https://www.rust-lang.org/tools/install]. Checkout the code and.... 

```shell
cargo build
cargo run -- -u some_user
```


```shell
cargo build --release
cargo run --release
```

See code doc
```shell
cargo doc --open
```

```shell
BEARER_TOKEN=YOUR_TWITTER_BEARER_TOKEN_HERE ./target/release/twitter-media-downloader -u NASAHubble 
```

## Twitter Developer Platform

Signup for a developer account; https://developer.twitter.com/en/docs/twitter-api/getting-started/getting-access-to-the-twitter-api
Follow the instructions on the Twitter Developer Platform. In a nutshell: 
* Create a Project and an App. 
* Create a BEARER_TOKEN for your app. 
* AND do NOT share your token with anyone.


## Some Fun Use Cases

### Continuously Updating Photo Album on a Raspberry Pi 

Say you already downloaded photos from a number of Twitter accounts and your `--output-dir` has a bunch of photos. 

Why not create a cron job (`crontab -e`) to scan your `--output-dir` directory and keep downloading the latest media for those accounts! 

```shell
# Make sure that your `BEARER_TOKEN` is set in the environment or pass it in as an argument.
find . -maxdepth 1 -type d ! -name "." -execdir sh -c '../target/release/twitter-media-downloader -c 5 -r -o ./ -u  `basename {}`;' \;
```
The example above is for Linux / Raspberry Pi.  _MacOSX directory scan and basename extraction looks slightly different_

You can then use `feh` to display those images in a forever loop. `feh` is an `apt install` away from you.
E.g. 
```shell
feh -F -Y -x -q -D 0.5 -R 300 -B black -Z -z -d --draw-tinted -r ./out
```

## Disclaimer
This project is an attempt to learn Rust. And it is put together over a weekend. Please be nice. :) 
