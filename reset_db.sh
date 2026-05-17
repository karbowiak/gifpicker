#!/usr/bin/env bash
# Delete the app's database and downloaded media.
# Works on macOS and Linux; on Windows use the equivalent PowerShell command:
#   Remove-Item -Recurse -Force "$env:APPDATA\com.karbowiak.gifpicker\{data,media}"

set -euo pipefail

case "$(uname -s)" in
  Darwin)
    APP_DIR="$HOME/Library/Application Support/com.karbowiak.gifpicker"
    ;;
  Linux)
    APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/com.karbowiak.gifpicker"
    ;;
  *)
    echo "Unsupported platform: $(uname -s). Delete the app data directory manually." >&2
    exit 1
    ;;
esac

rm -rf "$APP_DIR/data" "$APP_DIR/media"
echo "Removed $APP_DIR/{data,media}. Restart the app to recreate the database."
