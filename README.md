# GIF Picker

A fast, native GIF picker app built with Tauri, SvelteKit, and TypeScript. Search and manage your favorite GIFs with a beautiful, keyboard-driven interface.

## Features

- 🔍 Search GIFs via Giphy API
- ⭐ Save favorites locally with offline access
- ⌨️ Keyboard navigation (Arrow keys, Enter, Escape)
- 📋 Copy GIFs to clipboard (works with Discord, Slack, etc.)
- 🎨 Beautiful masonry layout
- 🚀 Fast native performance
- 💾 Local caching for instant access

## Downloads

Pre-built installers are available on the [Releases](https://github.com/YOUR_USERNAME/gifpicker/releases) page:

- **macOS**: DMG installer
- **Windows**: NSIS .exe installer

## Building

See [BUILDING.md](BUILDING.md) for instructions on building from source and creating releases.

## Development

### Prerequisites

- [Bun](https://bun.sh/) - JavaScript runtime and package manager
- [Rust](https://www.rust-lang.org/) - For Tauri backend
- [Node.js](https://nodejs.org/) (optional, if not using Bun)

### Setup

```bash
# Install dependencies
bun install

# Run development server
bun run tauri dev

# Build for production
bun run tauri build
```

## Configuration

On first run, you'll be prompted to add your Giphy API key:

1. Visit [developers.giphy.com](https://developers.giphy.com)
2. Login and create an app
3. Copy your API key
4. Paste it in the Settings panel

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
