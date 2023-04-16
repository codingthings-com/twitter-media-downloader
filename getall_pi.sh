echo "Scanning all directories / twitter users under out and downloading the new stuff"
cd /home/pi/twitter-media-downloader/out
export BEARER_TOKEN="AAAAAAAAAAAAAAAAAAAAAIyoZwEAAAAAD4AD7nqL7Y4aNBXo5gMPCYuUK7Q%3Dw4FCblpdukzKbfUU3DpcrFUqlR2uRUD8lOV1QiyOtnQk9qOnTp"
find . -maxdepth 1 -type d ! -name "." -execdir sh -c '../target/release/twitter-media-downloader -c 5 -r -o ./ -u  `basename {}`;' \;
echo "completed"
