# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.20.0] - 2025-12-03

### Added - Theme System Completion
- **Theme Management**:
  - **Save Theme**: Save your custom theme to a JSON file in the configured themes directory
  - **Export Theme**: Export your theme configuration to a JSON file
  - **Theme Naming**: Name your custom themes directly in the editor
  - **Auto-Discovery**: Automatically load new themes from the themes directory

## [0.19.0] - 2025-12-03

### Added - Theme Editor (Core UI)
- **Interactive Theme Editor**:
  - Open with `t` key to access theme customization
  - RGB sliders for background, foreground, and accent colors (0-255 range)
  - Real-time color preview panel showing changes
  - Visual color swatches with hex color codes
  - Live preview area demonstrating theme appearance

### Future
- Remaining theme features (export, auto-discovery, theme naming) planned for v0.20.0

## [0.18.0] - 2025-12-02

### Added - Configuration & Customization
- **Configurable Keybindings**:
  - Define custom keybindings in `config.ron`
  - Support for modifiers (Ctrl, Alt, Shift, Cmd)
  - Context-aware keybinding resolution
  - Default keybindings preserved with override capability
- **UI Customization**:
  - Customizable status bar format via `status_bar_format` config
  - Configurable list view items (toggle visibility of score, comments, author, etc.)
  - `UiConfig` struct added to `AppConfig`
- **Internal Architecture**:
  - Introduced `Action` enum to represent all application actions
  - Refactored event handling to be data-driven instead of hardcoded strings

### Changed
- `handle_key_down` now resolves actions from `AppConfig` before executing
- Status bar rendering now uses the configured format string
- Story list item rendering now respects `list_view_items` configuration

## [0.17.0] - 2025-12-02

### Added - Rendering Optimizations
- **Manual Windowing**: Only renders visible story items + 5 item buffer
  - ~97% reduction in rendered elements for large lists (500 stories → ~20-25 rendered)
  - Dramatically improved rendering performance and scroll smoothness
  - Uses viewport-based calculation with `STORY_ITEM_HEIGHT` constant
- **Scroll Position Persistence**: Automatic save/restore of scroll positions
  - Story list scroll position preserved when switching to story detail or other views
  - Article scroll position preserved when switching back to list
  - Seamless navigation experience

### Changed
- Story list rendering now uses viewport-based windowing instead of rendering all items
- Both `StoryListView` and `StoryDetailView` automatically save/restore scroll positions

### Technical
- Added `AppState` fields: `viewport_start_index`, `viewport_end_index`, `visible_buffer`, `story_list_scroll_position`, `article_scroll_position`
- Created `src/internal/ui/constants.rs` with rendering constants (`STORY_ITEM_HEIGHT = 85px`)
- Implemented `calculate_visible_range()` method for viewport calculation
- Added `save_scroll_position()` and `get_scroll_position()` methods to `AppState`

## [0.16.0] - 2025-12-02

### Added
- **Concurrent Fetching**:
  - Parallel fetching of stories and comments (up to 10 concurrent requests)
  - Uses `futures::stream::buffer_unordered` for efficient batch processing
  - Significantly faster list loading (3-5x improvement)
- **Request Management**:
  - Automatic cancellation of stale fetch tasks when switching views
  - Cache-based request deduplication to prevent redundant API calls
  - Offline fallback: serves stale cache data if network request fails
- **Dependencies**:
  - Added `futures = "0.3.31"`
  - Added `tokio-util = "0.7.17"`
  - Added `dashmap = "6.1.0"` (infrastructure)

### Changed
- `fetch_stories` and `fetch_comments` now execute concurrently
- Improved responsiveness by cancelling unnecessary background work

## [0.15.0] - 2025-12-02

### Added
- **In-Memory Caching System**:
  - Thread-safe `Cache<T>` with TTL-based expiration
  - Story lists cached for 5 minutes
  - Individual stories cached for 5 minutes
  - Comments cached for 5 minutes
  - Significantly improved response times (cache hits <10ms)
- **Rate Limiting**:
  - Semaphore-based rate limiting (3 concurrent requests)
  - Respects Hacker News API guidelines
  - Automatic backpressure via `tokio::sync::Semaphore`
- **Performance Improvements**:
  - Reduced API request count through caching
  - Instant responses for cached content
  - Better app responsiveness during network delays

### Changed
- `ApiService` now includes integrated caching and rate limiting
- All API methods (fetch_story_ids, fetch_story_content, fetch_comment_content) now use cache-first strategy

## [0.14.0] - 2025-12-02

### Added
- **Article Rendering**:
  - Syntax highlighting for code blocks using `syntect`
  - Support for multiple programming languages (Rust, Python, JavaScript, etc.)
  - Base16 Ocean Dark theme for code highlighting
- **Story Metadata Display**:
  - Domain extraction from URLs (e.g., "github.com")
  - Two-line metadata layout for better visual hierarchy
  - Relative time display ("2h ago", "3d ago")
  - Enhanced title wrapping with indentation
  - "View Article" link button instead of full URL display
- **Utilities**:
  - `utils::url::extract_domain()` for extracting domains from URLs
  - `utils::datetime::format_relative_time()` for cleaner API

### Dependencies
- Added `syntect = "5.2.0"` for syntax highlighting
- Added `scraper = "0.20.0"` for HTML parsing (future use)
- Added `textwrap = "0.16.0"` for text utilities (future use)

## [0.13.0] - 2025-12-02

### Added
- **Comment Threading**:
  - Recursive comment fetching up to 3 levels deep
  - Visual indentation (20px per depth level)
  - Left border visual guides for nested comments
- **Comment Pagination**:
  - Incremental comment loading (20 per batch)
  - "Load More Comments" button with progress indicator
- **Code Quality**:
  - Refactored `story_detail.rs` to use pattern matching
  - `CommentViewModel` for better UI state management

## [0.12.0] - 2025-12-02

### Added
- **Enhanced Search**:
  - Regex search support for advanced queries
  - Search modes: Title, Comments, Both
  - Persistent search history (last 20 searches)
  - Arrow navigation through search history
  - Live regex error feedback
- **Sorting Options**:
  - Sort by Score, Comments, or Time
  - Ascending/Descending order toggle
  - Visual sort status indicator
- **Keyboard Shortcuts**:
  - `Ctrl+R`: Focus search bar
  - `Ctrl+M`: Cycle search modes
  - `Ctrl+S`: Cycle sort options
  - `O`: Toggle sort order
  - `Up`/`Down`: Navigate search history

### Dependencies
- Added `regex = "1.10.0"`

## [0.11.0] - 2025-12-01

### Added
- **Bookmarks System**:
  - Toggle bookmarks with `b` key on any story
  - View all bookmarks with `B` key
  - Bookmark indicator (★) displayed next to bookmarked stories
  - Persistent storage in `~/.config/gpui-hn-app/bookmarks.json`
  - Export/import bookmarks functionality
- **History Tracking**:
  - Track last 50 viewed stories
  - View history with `H` key
  - Clear history with `X` key
  - Display "viewed X ago" timestamps
  - Persistent storage in `history.json`

### Dependencies
- Added `jiff = "0.1.13"` for timestamp management

## [0.10.0] - 2025-11-30

### Added
- Theme system with hot-reload support
- WebView integration for in-app article rendering
- Configuration management via `config.json`
- Theme injection modes (disabled, auto, force)
- WebView zoom controls (80%-200%)

### Changed
- Improved error handling using `anyhow`
- Better component structure and separation of concerns

## [0.9.0] - 2025-11-29

### Added
- Initial GPUI implementation
- Story browsing (Top, New, Best, Ask, Show, Job)
- Story details view
- Basic pagination support
- Scroll navigation
