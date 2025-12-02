# GPUI Hacker News App - Development Roadmap

> **Current Version**: v0.11.0  
> **Target**: v1.0.0 (Feature-complete GPUI port)

This roadmap is inspired by the feature-complete TUI implementation (`tui-hn-app`) and outlines features to port to the GPUI version (`gpui_hn_app`).

## 📋 TUI Reference Implementation

The TUI version (v0.9.4) has successfully implemented:
- Story browsing (Top, New, Best, Ask, Show, Job)
- Story details and comments
- In-app article rendering
- Theming with JSON files
- Search/filter functionality
- Pagination (load more/all)
- **Bookmarks/Favorites system**
- **History tracking**
- **Comment threading** with collapse/expand
- **Enhanced search** (regex, search modes)
- **Sorting options** (Score, Comments, Time)
- **Key binding customization**
- **Interactive theme editor**
- **UI customization** (status bar, padding)
- **Log viewer** and performance metrics
- **Accessibility features** (high contrast, verbose status)
- **Request deduplication** and cancellation
- **Offline mode** with cache fallback
- **Property-based testing** and benchmarks

---

## 🚀 Version Roadmap (v0.11.0 → v1.0.0)

### v0.11.0 - Bookmarks & History 📚
**Focus**: User data management and personalization

- [x] **Bookmarks/Favorites System**
    - [x] Toggle bookmark with `b` key on any story
    - [x] View all bookmarks with `B` key
    - [x] Bookmark indicator (★) displayed next to bookmarked stories
    - [x] Persistent storage in `~/.config/gpui-hn-app/bookmarks.json`
    - [x] Export/import bookmarks
    - [x] Uses `jiff` for timestamp management

- [x] **History Tracking**
    - [x] Track last 50 viewed stories
    - [x] View history with `H` key
    - [x] Clear history with `X` key
    - [x] Persistent storage in `history.json`
    - [x] Display "viewed X ago" timestamp

**Dependencies**: `jiff`, `serde_json`

---

### v0.12.0 - Enhanced Search & Sorting 🔍
**Focus**: Discovery and organization

- [x] **Enhanced Search**
    - [x] Regex search support with `Ctrl+R` or `F3`
    - [x] Search modes: Title only, Comments only, or Both (cycle with `Ctrl+M` or `F2`)
    - [x] Search history navigation with `↑`/`↓` arrows
    - [x] Persistent search history (last 20 searches)
    - [x] Live regex error feedback

- [x] **Sorting Options**
    - [x] Sort by Score, Comments, or Time
    - [x] Toggle Ascending/Descending order with `O` key
    - [x] Visual indicator showing current sort mode

**Dependencies**: `regex`

---

### v0.13.0 - Comment Enhancements 💬
**Focus**: Improved comment readability and navigation

- [x] **Comment Threading**
    - [x] Indentation for nested comments
    - [x] Tree-like structure with visual guides (border-left)
    - [x] Recursive comment fetching (depth 3)

- [x] **Comment Pagination**
    - [x] Load comments incrementally in batches (20 per batch)
    - [x] "Load More Comments" button
    - [x] Show comment count vs loaded count
    - [x] Lazy loading for deep comment trees (3-level depth fetch)

---

### v0.14.0 - Article & Content Display 📰
**Focus**: Rich content rendering

- [x] **Better Article Rendering**
    - [x] Syntax highlighting for code blocks (using `syntect`)
    - [ ] Table rendering (ASCII style)
    - [ ] Image placeholders with alt text extraction
    - [x] Improved list and quote styling

- [x] **Story Metadata Display**
    - [x] Domain/source extraction (e.g., "github.com")
    - [x] Two-line layout for better readability
    - [x] Age indicator using relative time
    - [x] Long title wrapping with proper indentation

**Dependencies**: `syntect`, `scraper`, `textwrap`

---

### v0.15.0 - Caching & Performance Foundation ⚡
**Focus**: Performance infrastructure

- [x] **In-Memory Caching with TTL**
    - [x] Story cache (5 minute TTL)
    - [x] Comment cache (5 minute TTL)
    - [x] Article cache (15 minute TTL)
    - [x] Thread-safe implementation using `Arc<RwLock<>>`

- [x] **Rate Limiting**
    - [x] Semaphore-based API rate limiting
    - [x] Default: 3 requests/second (respects HN API guidelines)
    - [x] Automatic rate limiting via `tokio::sync::Semaphore`

**Dependencies**: `tokio`

---

### v0.16.0 - Concurrent Fetching & Request Management 🚄
**Focus**: Advanced performance and resilience

- [ ] **Concurrent Fetching**
    - [ ] Batch concurrent requests for faster loading
    - [ ] Configurable concurrent limit (default: 10 concurrent requests)
    - [ ] Use `futures::stream::buffer_unordered`
    - [ ] 3-5x faster story loading

- [ ] **Request Management**
    - [ ] Request deduplication (prevent duplicate API calls using `DashMap`)
    - [ ] Request cancellation tokens (cancel stale requests on view switch)
    - [ ] Offline mode / Cache fallback when network fails

**Dependencies**: `dashmap`, `tokio-util`, `futures`

---

### v0.17.0 - Rendering Optimizations 🎨
**Focus**: UI performance

- [ ] **Rendering Optimizations**
    - [ ] Manual windowing for large story lists
    - [ ] Only render visible items plus small buffer
    - [ ] ~50% reduction in render time for 500+ story lists
    - [ ] Article scroll position persistence
    - [ ] Optimize GPUI component rendering

---

### v0.18.0 - Configuration & Customization ⚙️
**Focus**: User customization

- [ ] **Key Binding Customization**
    - [ ] Define custom keybindings via `config.ron`
    - [ ] Global and per-view mode keybindings
    - [ ] Hierarchical resolution (context-specific overrides global)
    - [ ] Support for simple keys, special keys, and modifiers

- [ ] **Keybinding Conflict Detection**
    - [ ] Warn user if custom bindings conflict on startup
    - [ ] Display notifications for conflicts
    - [ ] Log conflict details

- [ ] **UI Customization**
    - [ ] Customizable padding (horizontal and vertical)
    - [ ] Status bar format with tokens (`{mode}`, `{category}`, `{count}`, etc.)
    - [ ] List view field visibility (show/hide score, comments, domain, age, author)

**Dependencies**: `ron`

---

### v0.19.0 - Theme System Enhancements 🎨
**Focus**: Advanced theming

- [ ] **Theme Enhancements**
    - [ ] Interactive theme editor with real-time preview
    - [ ] RGB sliders for color adjustment
    - [ ] Export custom themes to JSON
    - [ ] Auto-discovery from configured theme directory
    - [ ] Theme naming with auto-complementary generation

**Dependencies**: `serde_json`

---

### v0.20.0 - Error Handling & Notifications 🔔
**Focus**: User feedback and network resilience

- [ ] **Enhanced Notifications**
    - [ ] Color-coded notifications (Info, Warning, Error)
    - [ ] Auto-dismiss with configurable timeouts
    - [ ] Notification helper methods: `notify_info()`, `notify_warning()`, `notify_error()`

- [ ] **Retry Mechanism**
    - [ ] Exponential backoff for transient failures
    - [ ] Configurable retry count and delays via `NetworkConfig`
    - [ ] Smart retry only on network errors and timeouts (not 4xx)
    - [ ] Retry progress logging

---

### v0.21.0 - Logging & Debugging 🐛
**Focus**: Developer tools and observability

- [ ] **Configurable Logging**
    - [ ] Log level configuration per module
    - [ ] Custom log directory support
    - [ ] `RUST_LOG` environment variable override
    - [ ] Conditional performance metrics

- [ ] **Log Viewer**
    - [ ] In-app debug log viewer (toggle with `L`)
    - [ ] Syntax highlighting for log levels
    - [ ] Scrollable log history (last 1000 lines)
    - [ ] Auto-scroll to bottom when opened

**Dependencies**: `tracing`, `tracing-subscriber`, `tracing-appender`

---

### v0.22.0 - Accessibility Features ♿
**Focus**: Inclusive design

- [ ] **High Contrast Theme**
    - [ ] WCAG AAA compliant colors
    - [ ] Pure black background with pure white text
    - [ ] Bright yellow highlights for maximum visibility

- [ ] **Accessibility Configuration**
    - [ ] `high_contrast_mode: bool` config option
    - [ ] `verbose_status: bool` for descriptive status messages
    - [ ] Dedicated `AccessibilityConfig` struct

- [ ] **Verbose Status Mode**
    - [ ] Full-sentence status messages for screen readers
    - [ ] Context-aware verbose messaging
    - [ ] Keyboard navigation announcements

---

### v0.23.0 - Testing Infrastructure 🧪
**Focus**: Quality assurance and performance tracking

- [ ] **Integration Tests**
    - [ ] Mock API server setup with `mockito`
    - [ ] Tests for story list fetching (Top/New/etc.)
    - [ ] Tests for story details and comment fetching
    - [ ] End-to-end user flow tests

- [ ] **Property-Based Tests**
    - [ ] Use `proptest` for edge case testing
    - [ ] Test invariants and properties
    - [ ] Fuzz testing for parsing logic

- [ ] **Benchmarks**
    - [ ] Performance benchmarks using `criterion`
    - [ ] Render benchmarking for large lists
    - [ ] Cache performance benchmarks
    - [ ] API call performance tracking

**Dependencies**: `mockito`, `proptest`, `criterion`

---

## 🎯 GPUI-Specific Enhancements (Post-Port)

### WebView Integration
- [ ] **Theme Injection Improvements**
    - [ ] Add domain whitelist for trusted pages
    - [x] Improve selector scoping and background detection
    - [x] Runtime UI toggle to enable/disable injection
    - [x] Non-invasive theming mode with CSS variables

- [ ] **State Management**
    - [ ] Remember and restore previous view state on hide

### UI/UX Polish
- [ ] **Loading Indicators**
    - [ ] Animated loading spinner
    - [ ] Context-aware loading messages
    - [ ] Progress percentage display

- [ ] **Keyboard Shortcuts Help**
    - [ ] Press `?` to show help overlay
    - [ ] Multi-page help system with tabs
    - [ ] Context-sensitive shortcuts per view mode

---

## 🏁 v1.0.0 - Stable Release

**Target**: Feature-complete, production-ready GPUI Hacker News client

**Requirements**:
- ✅ All v0.11.0 - v0.23.0 features implemented
- ✅ Comprehensive test coverage (>80%)
- ✅ Performance benchmarks met
- ✅ All accessibility features working
- ✅ Documentation complete
- ✅ Stable API for themes and configuration
- ✅ Cross-platform testing (macOS, Linux, Windows)

---

## 🚀 Post-1.0 Ideas

### Advanced Features
- [ ] **Plugin/Extension System**
    - [ ] Plugin architecture for extending functionality
    - [ ] Custom view plugins
    - [ ] Theme plugin support

- [ ] **Cloud Sync**
    - [ ] Sync bookmarks across devices
    - [ ] Sync history and settings
    - [ ] Optional cloud backend

- [ ] **Multi-Account Support**
    - [ ] Multiple Hacker News accounts
    - [ ] Account-specific bookmarks and history

- [ ] **Advanced Filtering**
    - [ ] Save custom filters
    - [ ] Filter by domain, score range, date range
    - [ ] Filter presets

- [ ] **Custom Layouts**
    - [ ] Configurable pane layouts
    - [ ] Multi-column views
    - [ ] Customizable component placement

---

## 📝 Development Notes

### Dependencies Summary
Core dependencies to add during porting:
- `jiff` - Date/time handling (v0.11.0)
- `regex` - Regular expressions (v0.12.0)
- `syntect` - Code syntax highlighting (v0.14.0)
- `scraper` - HTML parsing (v0.14.0)
- `textwrap` - Text wrapping (v0.14.0)
- `dashmap` - Concurrent maps (v0.16.0)
- `tokio-util` - Cancellation tokens (v0.16.0)
- `futures` - Stream utilities (v0.16.0)
- `tracing`, `tracing-subscriber`, `tracing-appender` - Logging (v0.21.0)
- `mockito` - HTTP mocking (v0.23.0)
- `proptest` - Property testing (v0.23.0)
- `criterion` - Benchmarking (v0.23.0)

### Performance Targets
Based on TUI implementation:
- Story loading: 3-5x faster with concurrent fetching
- List rendering: ~50% faster with manual windowing
- Cache hit rate: High for frequently accessed content
- API calls: Rate limited to 3 req/sec (configurable)

### Configuration Structure
Each version should update `config.ron` structure:
- v0.11.0: Bookmarks and history paths
- v0.12.0: Search configuration
- v0.15.0: Cache TTL settings
- v0.16.0: Network configuration (retry, rate limit, concurrent)
- v0.18.0: Keybindings and UI customization
- v0.19.0: Theme directory
- v0.21.0: Logging configuration
- v0.22.0: Accessibility options


## ✅ Already Implemented in TUI (Reference)

The TUI version has implemented:
- Story browsing (Top, New, Best, Ask, Show, Job)
- Story details and comments
- In-app article rendering
- Theming with JSON files
- Search/filter functionality
- Pagination (load more/all)
- Bookmarks/Favorites system
- History tracking
- Comment threading with collapse/expand
- Enhanced search (regex, search modes)
- Sorting options (Score, Comments, Time)
- Key binding customization
- Interactive theme editor
- UI customization (status bar, padding)
- Log viewer and performance metrics
- Accessibility features (high contrast, verbose status)
- Request deduplication and cancellation
- Offline mode with cache fallback
- Property-based testing and benchmarks

## 🚀 Core Features to Port to GPUI

### Essential TUI Features
- [ ] **Bookmarks/Favorites System**
    - [ ] Toggle bookmark with `b` key on any story
    - [ ] View all bookmarks with `B` key
    - [ ] Bookmark indicator (★) displayed next to bookmarked stories
    - [ ] Persistent storage in `~/.config/gpui-hn-app/bookmarks.json`
    - [ ] Export/import bookmarks
    - [ ] Uses `jiff` for timestamp management

- [ ] **History Tracking**
    - [ ] Track last 50 viewed stories
    - [ ] View history with `H` key
    - [ ] Clear history with `X` key
    - [ ] Persistent storage in `history.json`
    - [ ] Display "viewed X ago" timestamp

- [ ] **Enhanced Search**
    - [ ] Regex search support with `Ctrl+R` or `F3`
    - [ ] Search modes: Title only, Comments only, or Both (cycle with `Ctrl+M` or `F2`)
    - [ ] Search history navigation with `↑`/`↓` arrows
    - [ ] Persistent search history (last 20 searches)
    - [ ] Live regex error feedback

- [ ] **Sorting Options**
    - [ ] Sort by Score, Comments, or Time
    - [ ] Toggle Ascending/Descending order with `O` key
    - [ ] Visual indicator showing current sort mode

### Comments Enhancements
- [ ] **Comment Threading**
    - [ ] Indentation for nested comments
    - [ ] Tree-like structure with visual guides (└─, │)
    - [ ] Collapse/expand comment threads
    - [ ] Recursive comment fetching

- [ ] **Comment Pagination**
    - [ ] Load comments incrementally in batches
    - [ ] "Load More Comments" action (press `n`)
    - [ ] Show comment count vs loaded count
    - [ ] Lazy loading for deep comment trees (initial 3-level depth fetch)
    - [ ] On-demand loading when expanding collapsed comments

### Article & Content
- [ ] **Better Article Rendering**
    - [ ] Syntax highlighting for code blocks (using `syntect`)
    - [ ] Table rendering (ASCII style)
    - [ ] Image placeholders with alt text extraction
    - [ ] Improved list and quote styling
    - [ ] Use `scraper` for robust HTML parsing

- [ ] **Story Metadata Display**
    - [ ] Domain/source extraction (e.g., "github.com")
    - [ ] Two-line layout for better readability
    - [ ] Age indicator using relative time
    - [ ] Long title wrapping with proper indentation

### Performance & Caching
- [ ] **In-Memory Caching with TTL**
    - [ ] Story cache (5 minute TTL)
    - [ ] Comment cache (5 minute TTL)
    - [ ] Article cache (15 minute TTL)
    - [ ] Thread-safe implementation using `Arc<RwLock<>>`

- [ ] **Concurrent Fetching**
    - [ ] Batch concurrent requests for faster loading
    - [ ] Configurable concurrent limit (default: 10 concurrent requests)
    - [ ] Use `futures::stream::buffer_unordered`
    - [ ] 3-5x faster story loading

- [ ] **Rate Limiting**
    - [ ] Semaphore-based API rate limiting
    - [ ] Default: 3 requests/second (respects HN API guidelines)
    - [ ] Automatic rate limiting via `tokio::sync::Semaphore`

- [ ] **Request Management**
    - [ ] Request deduplication (prevent duplicate API calls using `DashMap`)
    - [ ] Request cancellation tokens (cancel stale requests on view switch)
    - [ ] Offline mode / Cache fallback when network fails

- [ ] **Rendering Optimizations**
    - [ ] Manual windowing for large story lists
    - [ ] Only render visible items plus small buffer
    - [ ] ~50% reduction in render time for 500+ story lists
    - [ ] Article scroll position persistence

### Configuration & Customization
- [ ] **Key Binding Customization**
    - [ ] Define custom keybindings via `config.ron`
    - [ ] Global and per-view mode keybindings
    - [ ] Hierarchical resolution (context-specific overrides global)
    - [ ] Support for simple keys, special keys, and modifiers

- [ ] **Keybinding Conflict Detection**
    - [ ] Warn user if custom bindings conflict on startup
    - [ ] Display notifications for conflicts
    - [ ] Log conflict details

- [ ] **UI Customization**
    - [ ] Customizable padding (horizontal and vertical)
    - [ ] Status bar format with tokens (`{mode}`, `{category}`, `{count}`, etc.)
    - [ ] List view field visibility (show/hide score, comments, domain, age, author)

- [ ] **Theme Enhancements**
    - [ ] Interactive theme editor with real-time preview
    - [ ] RGB sliders for color adjustment
    - [ ] Export custom themes to JSON
    - [ ] Auto-discovery from configured theme directory
    - [ ] Theme naming with auto-complementary generation

### Error Handling & Logging
- [ ] **Enhanced Notifications**
    - [ ] Color-coded notifications (Info, Warning, Error)
    - [ ] Auto-dismiss with configurable timeouts
    - [ ] Notification helper methods: `notify_info()`, `notify_warning()`, `notify_error()`

- [ ] **Retry Mechanism**
    - [ ] Exponential backoff for transient failures
    - [ ] Configurable retry count and delays via `NetworkConfig`
    - [ ] Smart retry only on network errors and timeouts (not 4xx)
    - [ ] Retry progress logging

- [ ] **Configurable Logging**
    - [ ] Log level configuration per module
    - [ ] Custom log directory support
    - [ ] `RUST_LOG` environment variable override
    - [ ] Conditional performance metrics

- [ ] **Log Viewer**
    - [ ] In-app debug log viewer (toggle with `L`)
    - [ ] Syntax highlighting for log levels
    - [ ] Scrollable log history (last 1000 lines)
    - [ ] Auto-scroll to bottom when opened

### Accessibility
- [ ] **High Contrast Theme**
    - [ ] WCAG AAA compliant colors
    - [ ] Pure black background with pure white text
    - [ ] Bright yellow highlights for maximum visibility

- [ ] **Accessibility Configuration**
    - [ ] `high_contrast_mode: bool` config option
    - [ ] `verbose_status: bool` for descriptive status messages
    - [ ] Dedicated `AccessibilityConfig` struct

- [ ] **Verbose Status Mode**
    - [ ] Full-sentence status messages for screen readers
    - [ ] Context-aware verbose messaging

### Testing Infrastructure
- [ ] **Integration Tests**
    - [ ] Mock API server setup with `mockito`
    - [ ] Tests for story list fetching (Top/New/etc.)
    - [ ] Tests for story details and comment fetching

- [ ] **Property-Based Tests**
    - [ ] Use `proptest` for edge case testing
    - [ ] Test invariants and properties

- [ ] **Benchmarks**
    - [ ] Performance benchmarks using `criterion`
    - [ ] Render benchmarking for large lists
    - [ ] Cache performance benchmarks

## 🎯 GPUI-Specific Enhancements

### WebView Integration
- [ ] **Theme Injection Improvements**
    - [ ] Add domain whitelist for trusted pages
    - [x] Improve selector scoping and background detection
    - [x] Runtime UI toggle to enable/disable injection
    - [x] Non-invasive theming mode with CSS variables

- [ ] **State Management**
    - [ ] Remember and restore previous view state on hide

### UI/UX Polish
- [ ] **Loading Indicators**
    - [ ] Animated loading spinner with Unicode characters
    - [ ] Context-aware loading messages
    - [ ] Compact loading overlay (3 lines)
    - [ ] Progress percentage display

- [ ] **Keyboard Shortcuts Help**
    - [ ] Press `?` to show help overlay
    - [ ] Multi-page help system with tabs
    - [ ] Context-sensitive shortcuts per view mode

## 🧹 Code Quality & Refactoring

### Architecture
- [x] **Componentization**
    - [x] Break down rendering into smaller, reusable components
    - [x] Extract platform-specific logic

- [x] **Error Handling**
    - [x] Use `anyhow::Result` for better error context
    - [x] Replace `unwrap` calls with proper error handling

### Testing
- [x] **Unit Tests**
    - [x] Add unit tests for core functionality
    - [x] Mock network requests for API tests

- [ ] **Integration Tests**
    - [ ] Validate behavior on known test pages
    - [ ] End-to-end user flow tests

## 🏁 Post-1.0 Ideas

### Advanced Features
- [ ] **Plugin/Extension System**
    - [ ] Plugin architecture for extending functionality
    - [ ] Custom view plugins
    - [ ] Theme plugin support

- [ ] **Cloud Sync**
    - [ ] Sync bookmarks across devices
    - [ ] Sync history and settings
    - [ ] Optional cloud backend

- [ ] **Multi-Account Support**
    - [ ] Multiple Hacker News accounts
    - [ ] Account-specific bookmarks and history

- [ ] **Advanced Filtering**
    - [ ] Save custom filters
    - [ ] Filter by domain, score range, date range
    - [ ] Filter presets

- [ ] **Custom Layouts**
    - [ ] Configurable pane layouts
    - [ ] Multi-column views
    - [ ] Customizable component placement

## 📝 Notes

### Dependencies to Consider
- `jiff` - Date/time handling for bookmarks and history
- `dashmap` - Concurrent in-flight request tracking
- `tokio-util` - Cancellation tokens
- `scraper` - HTML parsing for articles
- `syntect` - Code syntax highlighting
- `textwrap` - Text wrapping utilities
- `mockito` - HTTP mocking for tests
- `proptest` - Property-based testing
- `criterion` - Benchmarking

### Configuration Structure
The TUI app uses a comprehensive `config.ron` structure with:
- `theme_name` and `theme_directory`
- `auto_switch_dark_to_light` for terminal-based theme switching
- `accessibility` section with `high_contrast_mode` and `verbose_status`
- `ui` section with `padding`, `status_bar_format`, and `list_view` settings
- `logging` section with `level`, `module_levels`, and `enable_performance_metrics`
- `network` section with retry configuration and rate limiting
- `keybindings` section with global and per-view mappings

### Performance Targets
Based on TUI implementation:
- Story loading: 3-5x faster with concurrent fetching
- List rendering: ~50% faster with manual windowing
- Cache hit rate: High for frequently accessed content
- API calls: Rate limited to 3 req/sec (configurable)
