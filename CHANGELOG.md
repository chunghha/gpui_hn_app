# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0] - 2025-12-02

### Added
- **Enhanced Search**:
  - Regex search support (with error feedback).
  - Search modes: Title, Comments, Both.
  - Persistent search history (max 20 items).
  - Search bar in `StoryListView` with history navigation.
- **Sorting**:
  - Sort stories by Score, Comments, or Time.
  - Toggle Ascending/Descending order.
- **Keyboard Shortcuts**:
  - `Ctrl+R`: Focus search bar.
  - `Ctrl+M`: Cycle search modes.
  - `Ctrl+S`: Cycle sort options.
  - `O`: Toggle sort order.
  - `Up`/`Down`: Navigate search history.

## [0.11.0] - 2025-12-01

### Added
- **Bookmarks System**:
  - Toggle bookmarks with `b` key on any story.
  - View all bookmarks with `B` key.
  - Visual indicator (★) for bookmarked stories.
  - Persistent storage in `~/.config/gpui-hn-app/bookmarks.json`.
- **History System**:
  - Automatically tracks last 50 viewed stories.
  - View history with `H` key.
  - Clear history with `X` key.
  - Relative timestamps (e.g., "viewed 5m ago").
  - Persistent storage in `~/.config/gpui-hn-app/history.json`.
- **Keyboard Shortcuts**:
  - `b`: Toggle bookmark.
  - `B`: Show bookmarks list.
  - `H`: Show history list.
  - `X`: Clear history (in history view).

### Changed
- Updated `Taskfile.yml` with `bench` and `install` tasks.
- Improved code organization with new `bookmarks` and `history` modules.
- Updated `AppState` to manage bookmarks and history.
