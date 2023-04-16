#!/bin/zsh

echo "Scanning all directories / twitter users under out and downloading the new stuff"
cd out
find . -type d -mindepth 1 -maxdepth 1 -execdir ../target/release/twitter-media-downloader -c 5 -r -o ./ -u "$(basename {})"  \;
cd ../