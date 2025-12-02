# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

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
- Added `syntect = "5.3.0"` for syntax highlighting
- Added `scraper = "0.24.0"` for HTML parsing (future use)
- Added `textwrap = "0.16.2"` for text utilities (future use)

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
