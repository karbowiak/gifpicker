# GitHub Copilot Instructions for GIF Picker

## Project Overview

GIF Picker is a native desktop application for searching, managing, and copying GIFs. Built with:
- **Frontend**: Svelte 4 + SvelteKit + TypeScript + Vite
- **Backend**: Tauri 2.x + Rust
- **Database**: SQLite with sqlx migrations
- **API**: Klipy API for GIF search

## Architecture

### Frontend Structure (`src/`)
- `routes/+page.svelte` - Main app page with keyboard navigation
- `lib/components/` - Reusable UI components
  - `SearchBar.svelte` - Search input with settings button
  - `MasonryLayout.svelte` - Grid layout for GIF display
  - `MediaItem.svelte` - Individual GIF card with lazy loading
  - `Settings.svelte` - Settings modal (forces API key on first run)
  - `Toast.svelte` - Notification system
  - `ContextMenu.svelte` - Right-click context menu
- `lib/stores/` - Svelte stores for state management
  - `search.ts` - Search query and results (local + Klipy)
  - `favorites.ts` - Favorited GIFs management
  - `settings.ts` - User settings (preferences)
  - `ui.ts` - UI state (selections, modals, toasts)
- `lib/types.ts` - TypeScript interfaces

### Backend Structure (`src-tauri/`)
- `src/main.rs` - Entry point
- `src/lib.rs` - Tauri app setup and command registration
- `src/commands/` - Tauri command handlers (Rust → Frontend bridge)
  - `search.rs` - Search local favorites and Klipy
  - `favorites.rs` - CRUD operations for favorites
  - `clipboard.rs` - Copy GIFs to clipboard
  - `files.rs` - Read local files as base64 data URLs
  - `settings.rs` - Get/save user settings
- `src/services/` - Business logic services
  - `klipy.rs` - Klipy API client (search, trending, categories)
  - `downloader.rs` - Download and cache GIF files locally
  - `clipboard.rs` - Platform-specific clipboard (macOS uses osascript)
- `src/db/` - Database layer
  - `connection.rs` - SQLite connection with migrations
  - `favorites.rs` - Favorites database queries
  - `settings.rs` - Settings database queries
- `src/models/` - Data models matching database schema
  - `favorite.rs` - Favorite GIF model with optional filepath + gif_url
  - `settings.rs` - Settings model with all user preferences
- `migrations/` - SQL migration files
  - `001_initial.sql` - Initial schema
  - `002_add_gif_url.sql` - Added URL fallback for favorites

## Key Features

### 1. GIF Search & Favorites
- Search Klipy API or local favorites
- Add Klipy GIFs to favorites (downloads and caches locally)
- Local cache: `~/Library/Application Support/com.karbowiak.gifpicker/media/gifs/`
- Each favorite has both `filepath` (local cache) and `gif_url` (backup/original)

### 2. Image Loading Strategy
- **Lazy loading**: IntersectionObserver watches containers, loads on scroll
- **Caching**: Map-based cache for base64 data URLs (no reload on hover)
- **Base64 serving**: `read_file_as_data_url` command converts local files
- **Why not convertFileSrc?**: Failed with spaces in Application Support path

### 3. Clipboard Integration
- **macOS**: Uses osascript AppleScript to copy actual file reference
  - `set the clipboard to (POSIX file "...") as «class furl»`
- **Cross-platform**: arboard for basic clipboard operations
- Copies GIF files (not URLs) so they paste as animated GIFs in Discord/Slack

### 4. Keyboard Navigation
- Arrow keys: Navigate grid (Up/Down/Left/Right)
- Enter: Select/copy current GIF
- Escape: Close window
- Selection state maintained in `selectedIndex` store

### 5. Settings System
- **Stored**: SQLite database with JSON serialization
- **UI**: Modal with preferences, link to Klipy

## Database Schema

### favorites table
```sql
id INTEGER PRIMARY KEY
filename TEXT NOT NULL
filepath TEXT (nullable - for backward compatibility)
gif_url TEXT (nullable - original Klipy URL)
source_id TEXT (Klipy ID)
source_url TEXT
file_size INTEGER
width INTEGER
height INTEGER
use_count INTEGER DEFAULT 0
created_at TEXT
last_used_at TEXT (nullable)
```

### settings table
```sql
key TEXT PRIMARY KEY
value TEXT (JSON serialized)
```

## Important Patterns

### 1. Tauri Commands
```rust
#[tauri::command]
async fn command_name(state: State<'_, AppState>) -> Result<T, String> {
    // Access services via state
    let result = state.service.method().await
        .map_err(|e| e.to_string())?;
    Ok(result)
}
```

### 2. Svelte Stores
```typescript
import { writable } from 'svelte/store';
export const store = writable<Type>(defaultValue);

// In components:
$: value = $store; // Auto-subscribe
store.set(newValue); // Update
```

### 3. Lazy Loading Pattern
```svelte
let containerElement: HTMLDivElement;
let observer: IntersectionObserver;
let hasLoadedOnce = false;

onMount(() => {
    observer = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting && !hasLoadedOnce) {
            loadImageUrl(); // Load once, cache forever
            hasLoadedOnce = true;
        }
    }, { rootMargin: '50px', threshold: 0.01 });
    observer.observe(containerElement);
});
```

### 4. Platform-Specific Clipboard (macOS)
```rust
#[cfg(target_os = "macos")]
fn copy_file_macos(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script = format!(
        r#"set the clipboard to (POSIX file "{}") as «class furl»"#,
        filepath
    );
    std::process::Command::new("osascript")
        .args(&["-e", &script])
        .output()?;
    Ok(())
}
```

## Build & Deploy

### Local Development
```bash
bun install
bun run tauri dev
```

### Production Build
```bash
bun run tauri build
# macOS: src-tauri/target/release/bundle/dmg/*.dmg
# Windows: src-tauri/target/release/bundle/nsis/*.exe
```

### GitHub Actions
- **CI**: `.github/workflows/ci.yml` - Runs on push/PR, builds both platforms
- **Release**: `.github/workflows/release.yml` - Triggers on tags (v*), creates releases
- **Version bump**: `./bump-version.sh 0.1.0` to update all version files

## Common Tasks

### Adding a new Tauri command
1. Create function in `src-tauri/src/commands/*.rs`
2. Register in `src-tauri/src/lib.rs` `.invoke_handler()`
3. Call from frontend: `await invoke('command_name', { params })`

### Adding a new setting
1. Update `Settings` interface in `src/lib/types.ts`
2. Update `Settings` struct in `src-tauri/src/models/settings.rs`
3. Update default in `src/lib/stores/settings.ts`
4. Add UI in `src/lib/components/Settings.svelte`

### Adding a database migration
1. Create `src-tauri/migrations/00X_description.sql`
2. Restart app - migrations run automatically on startup
3. Update models if schema changes

### Debugging
- Frontend: Chrome DevTools in Tauri window
- Backend: Add `println!()` or `dbg!()`, check terminal output
- Database: Check `~/Library/Application Support/com.karbowiak.gifpicker/gifpicker.db`

## Environment Setup

### No .env files!
### No .env files!
- Project uses internal configuration for Keys
- This ensures security and proper key management

### Dependencies
- Bun (JavaScript runtime)
- Rust (via rustup)
- Xcode Command Line Tools (macOS)
- Visual Studio Build Tools (Windows)

## Known Issues & Solutions

### Issue: convertFileSrc fails with spaces
**Solution**: Use `read_file_as_data_url` command to serve base64 data URLs

### Issue: Base64 loading causes UI stalls
**Solution**: Lazy loading with IntersectionObserver + caching

### Issue: Clipboard pastes static PNG
**Solution**: Use Klipy's high-quality GIF URL

### Issue: macOS clipboard doesn't paste in Discord
**Solution**: Use osascript to copy file reference, not path string

## File Locations

### User Data
- Database: `~/Library/Application Support/com.karbowiak.gifpicker/gifpicker.db`
- GIF Cache: `~/Library/Application Support/com.karbowiak.gifpicker/media/gifs/`

### Project Structure
```
gifpicker/
├── .github/
│   ├── workflows/          # CI/CD
│   └── copilot-instructions.md
├── src/                    # Frontend (Svelte)
├── src-tauri/              # Backend (Rust/Tauri)
├── static/                 # Static assets
├── package.json            # Node dependencies
├── bun.lock                # Bun lockfile
├── svelte.config.js        # Svelte config
├── vite.config.js          # Vite bundler config
├── tsconfig.json           # TypeScript config
└── bump-version.sh         # Version bump script
```

## Code Style

### TypeScript/Svelte
- Use TypeScript strict mode
- Prefer async/await over promises
- Use Svelte stores for global state
- Component props with `export let`

### Rust
- Use `async/await` with tokio
- Return `Result<T, String>` for Tauri commands
- Use `State` for dependency injection
- Follow Rust naming conventions (snake_case)

## Testing Strategy

Currently no automated tests. When adding:
- Frontend: Create `*.test.ts` files, use Bun's test runner
- Backend: Use `#[tokio::test]` with `#[ignore]` for integration tests

## Tips for Copilot

- **When modifying commands**: Always update both Rust handler and TypeScript types
- **When adding features**: Consider keyboard navigation implications
- **When handling images**: Remember lazy loading + caching pattern
- **When working with clipboard**: Consider platform-specific implementations
- **When updating database**: Always create migration, never modify existing ones
- **When handling errors**: Convert to strings for Tauri commands: `.map_err(|e| e.to_string())`
- **Settings changes**: Remember to force settings modal if API key is missing
