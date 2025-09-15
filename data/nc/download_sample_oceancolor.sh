#!/bin/bash

DEST_DIR=./files/

if [ ! -d "$DEST_DIR" ]; then
    mkdir -p $DEST_DIR
fi

# Remove existing all files in the destination directory
rm -rf $DEST_DIR/*

# Download sample data from NASA OceanColor in the destination directory

cd $DEST_DIR

echo "Starting download of NASA OceanColor data files..."
echo "Files will be saved to: $(pwd)"

if wget --load-cookies ~/.urs_cookies --save-cookies ~/.urs_cookies --auth-no-challenge=on --keep-session-cookies --content-disposition -i ../url.txt; then
    echo "Download completed successfully!"
    echo "Downloaded files:"
    ls -lh *.nc 2>/dev/null || echo "No .nc files found"
else
    echo "Download failed. Please check your NASA Earthdata credentials and network connection."
    exit 1
fi
