# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
