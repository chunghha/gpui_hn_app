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
- [ ] **Pagination & Loading**
    - [ ] Refactor `fetch_more_stories` logic to be more robust against failed fetches and accurate with `loaded_count`.
    - [ ] Implement better pagination for comments (currently limited to top 20).
    - [ ] Add visual feedback for "end of list" or "no more stories".

## Refactoring & Code Quality

### Architecture
- [ ] **Componentization**
    - [ ] Break down `HnLayout::render` into smaller, reusable components (e.g., `StoryList`, `StoryDetail`, `Header`).
    - [ ] Extract platform-specific WebView creation logic from `HnLayout::new` into a dedicated builder or factory.
- [ ] **Error Handling**
    - [ ] Replace `map_err(|e| e.to_string())` in `api/mod.rs` with proper error types (e.g., `thiserror` or `anyhow`) to preserve context.
    - [ ] Handle `unwrap` calls more gracefully throughout the application.

### Configuration
- [ ] **Hardcoded Values**
    - [x] Remove hardcoded viewport height approximation (`800.0`) in `layout.rs` scroll handler; use actual window/container size.

## Testing
- [ ] **Unit Tests**
    - [ ] Add unit tests for `make_init_script` in `webview.rs`.
    - [ ] Add tests for `ApiService` (mocking network requests).
- [ ] **Integration Tests**
    - [ ] Validate injection behavior on known test pages.
