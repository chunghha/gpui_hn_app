# Project TODOs

## Features & Enhancements

### WebView
- [ ] **Theme Injection Improvements**
    - [ ] Add a domain whitelist so injection only runs on trusted/internal pages.
    - [ ] Improve selector scoping and background detection (detect non-transparent site roots, background images).
    - [ ] Provide a runtime UI toggle to enable/disable `webview_theme_injection` and persist to `config.ron`.
    - [ ] Offer a non-invasive theming mode that sets CSS variables instead of forcing `!important`.
- [ ] **State Management**
    - [ ] Improve `hide_webview` to remember and restore the previous view state (e.g., return to specific story detail).

### Story & Comments
- [x] **Pagination & Loading**
    - [x] Refactor `fetch_more_stories` logic to be more robust against failed fetches and accurate with `loaded_count`.
    - [x] Implement better pagination for comments (currently limited to top 20).
    - [x] Add visual feedback for "end of list" or "no more stories".

## Refactoring & Code Quality

### Architecture
- [x] **Componentization**
    - [x] Break down `HnLayout::render` into smaller, reusable components (e.g., `StoryList`, `StoryDetail`, `Header`).
    - [x] Extract platform-specific WebView creation logic from `HnLayout::new` into a dedicated builder or factory.
- [x] **Error Handling**
    - [x] Replace `map_err(|e| e.to_string())` in `api/mod.rs` with proper error types (e.g., `thiserror` or `anyhow`) to preserve context.
    - [x] Handle `unwrap` calls more gracefully throughout the application.

### Configuration
- [x] **Hardcoded Values**
    - [x] Remove hardcoded viewport height approximation (`800.0`) in `layout.rs` scroll handler; use actual window/container size.

## Testing
- [x] **Unit Tests**
    - [x] Add unit tests for `make_init_script` in `webview.rs`.
    - [x] Add tests for `ApiService` (mocking network requests).
- [ ] **Integration Tests**
    - [ ] Validate injection behavior on known test pages.

## Cross-Platform Support
- [ ] **Linux Verification**
    - [ ] Verify `webview_factory.rs` compilation on Linux.
    - [ ] Add `gtk` to `Cargo.toml` under `[target.'cfg(target_os = "linux")'.dependencies]` if missing (currently missing).
    - [ ] Test WebView rendering with `wry` and `gtk` integration.

## Image handling

- [x] **Initial approach** 
    - [x] Surface images found in fetched HTML as simple inline placeholders (e.g. "[Image: alt]") by extracting `<img>` tags when fetching content. This avoids downloading and rendering remote images in the app process and keeps the renderer lightweight.
- [ ] **Tradeoffs**
    - [ ] A regex-based extractor is fast and keeps dependencies small but can miss edge cases; using an HTML parser (e.g. `scraper`/`kuchiki`) is more robust but adds dependencies and complexity. Consider starting with regex and switching to a parser if needed.
