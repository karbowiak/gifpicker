#!/bin/bash
# Delete the app database and media files
rm -rf "$HOME/Library/Application Support/com.karbowiak.gifpicker/data"
rm -rf "$HOME/Library/Application Support/com.karbowiak.gifpicker/media"
echo "Database and media files deleted. Restart the app to create a fresh database."
