#!/bin/sh

# include a mp4 file with the script
path="/tmp/silly.mp4"

extract > "$path"
xdg-open "$path"
rm "$path"
