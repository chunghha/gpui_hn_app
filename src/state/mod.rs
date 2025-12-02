mod imp {
    use crate::api::{ApiService, StoryListType};
    use crate::bookmarks::Bookmarks;
    use crate::history::History;
    use crate::internal::models::{CommentViewModel, Story};
    use crate::search::SearchHistory;
    use crate::utils::html::extract_text_from_html;
    use futures::StreamExt;
    use futures::channel::mpsc;
    use gpui::{App, Entity, Task, prelude::*};
    use std::sync::Arc;

    #[derive(Clone, PartialEq, Debug)]
    pub enum ViewMode {
        List,
        Story(Story),
        Webview(String),
        Bookmarks,
        History,
        ThemeEditor,
    }

    #[derive(Clone, PartialEq, Debug, Copy)]
    pub enum SearchMode {
        Title,
        Comments,
        Both,
    }

    #[derive(Clone, PartialEq, Debug, Copy)]
    pub enum SortOption {
        Score,
        Comments,
        Time,
    }

    #[derive(Clone, PartialEq, Debug, Copy)]
    pub enum SortOrder {
        Ascending,
        Descending,
    }

    pub struct AppState {
        pub stories: Vec<Story>,
        pub loading: bool,
        pub loading_more: bool,
        pub all_stories_loaded: bool,
        pub current_list: StoryListType,
        api_service: Arc<ApiService>,
        pub view_mode: ViewMode,
        pub selected_story_content: Option<String>,
        pub selected_story_content_loading: bool,
        pub comments: Vec<CommentViewModel>,
        pub comments_loading: bool,
        pub loaded_comment_count: usize,
        pub comment_ids: Vec<u32>,
        pub config: crate::config::AppConfig,
        pub story_ids: Vec<u32>,
        pub loaded_count: usize,
        pub bookmarks: Bookmarks,
        pub history: History,
        pub search_history: SearchHistory,
        pub search_query: String,
        pub search_mode: SearchMode,
        pub sort_option: SortOption,
        pub sort_order: SortOrder,
        pub regex_error: Option<String>,
        pub should_focus_search: bool,
        pub fetch_task: Option<Task<()>>,
        pub comment_fetch_task: Option<Task<()>>,
        // Windowing for performance optimization
        pub viewport_start_index: usize,
        pub viewport_end_index: usize,
        pub visible_buffer: usize,
        // Scroll position persistence
        pub story_list_scroll_position: f32,
        pub article_scroll_position: f32,
    }

    impl AppState {
        pub fn new(config: crate::config::AppConfig, cx: &mut App) -> Entity<Self> {
            let api_service = Arc::new(ApiService::new());
            let bookmarks = Bookmarks::load();
            let history = History::load();
            // Assuming config dir is available or we can construct it.
            // For now, let's use the same logic as Bookmarks/History which seem to handle paths internally
            // or we might need to pass the config dir.
            // Looking at Bookmarks::load(), it seems to determine path internally.
            // But SearchHistory::new takes a PathBuf.
            // Let's check where Bookmarks stores files.
            // It uses dirs::config_dir().
            let config_dir = dirs::config_dir()
                .map(|p| p.join("gpui-hn-app"))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let search_history = SearchHistory::new(config_dir);

            cx.new(|_cx| Self {
                stories: Vec::new(),
                loading: true,
                loading_more: false,
                all_stories_loaded: false,
                current_list: StoryListType::Top,
                api_service,
                view_mode: ViewMode::List,
                selected_story_content: None,
                selected_story_content_loading: false,
                comments: Vec::new(),
                comments_loading: false,
                loaded_comment_count: 0,
                comment_ids: Vec::new(),
                config,
                story_ids: Vec::new(),
                loaded_count: 0,
                bookmarks,
                history,
                search_history,
                search_query: String::new(),
                search_mode: SearchMode::Both,
                sort_option: SortOption::Score,
                sort_order: SortOrder::Descending,
                regex_error: None,
                should_focus_search: false,
                fetch_task: None,
                comment_fetch_task: None,
                // Windowing defaults
                viewport_start_index: 0,
                viewport_end_index: 0,
                visible_buffer: 5, // Render 5 items above/below viewport
                // Scroll positions
                story_list_scroll_position: 0.0,
                article_scroll_position: 0.0,
            })
        }

        pub fn fetch_stories(entity: Entity<Self>, list_type: StoryListType, cx: &mut App) {
            tracing::info!("Fetching story IDs for: {}", list_type);
            let api_service = entity.read(cx).api_service.clone();

            // Cancel any existing fetch task
            entity.update(cx, |state, cx| {
                state.fetch_task = None;

                tracing::info!("List type changed, resetting state.");
                state.loading = true;
                state.loading_more = false;
                state.all_stories_loaded = false;
                state.current_list = list_type;
                state.stories.clear();
                state.story_ids.clear();
                state.loaded_count = 0;
                cx.notify();
            });

            let (tx, mut rx) = mpsc::unbounded::<Vec<u32>>();

            let entity_clone = entity.clone();
            let mut async_cx = cx.to_async();
            let background = cx.background_executor().clone();

            // Create a new task for fetching IDs and then the first batch of stories
            let task = cx.foreground_executor().spawn(async move {
                // Fetch IDs in background
                let ids_result = background
                    .spawn(async move { api_service.fetch_story_ids(list_type) })
                    .await;

                match ids_result {
                    Ok(ids) => {
                        tracing::info!(
                            "Successfully fetched {} story IDs for: {}",
                            ids.len(),
                            list_type
                        );
                        let _ = tx.unbounded_send(ids);
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch story ids: {}", e);
                        let _ = tx.unbounded_send(Vec::new());
                    }
                }

                // Wait for IDs and trigger first batch fetch
                if let Some(ids) = rx.next().await {
                    let _ = entity_clone.update(&mut async_cx, |state, cx| {
                        state.story_ids = ids;
                        cx.notify();
                    });
                    // Trigger fetching the first batch
                    Self::fetch_more_stories(entity_clone, &mut async_cx).await;
                }
            });

            entity.update(cx, |state, _| {
                state.fetch_task = Some(task);
            });
        }

        pub async fn fetch_more_stories(entity: Entity<Self>, cx: &mut gpui::AsyncApp) {
            // Get the batch of IDs to fetch and track the batch size
            let (api_service, ids_to_fetch, batch_size) = entity
                .update(cx, |state, cx| {
                    // Check if we've already loaded all stories
                    if state.loaded_count >= state.story_ids.len() {
                        state.loading = false;
                        state.loading_more = false;
                        state.all_stories_loaded = true;
                        cx.notify();
                        return (None, Vec::new(), 0);
                    }

                    // Use loading_more for pagination, loading for initial fetch
                    state.loading_more = true;
                    cx.notify();

                    let start = state.loaded_count;
                    let end = (start + 20).min(state.story_ids.len());
                    let ids = state.story_ids[start..end].to_vec();
                    let batch_size = ids.len();

                    (Some(state.api_service.clone()), ids, batch_size)
                })
                .ok()
                .unwrap_or((None, Vec::new(), 0));

            if let Some(api_service) = api_service {
                let ids_count = ids_to_fetch.len();
                tracing::info!("Fetching {} story details concurrently...", ids_count);

                // Use concurrent fetch
                let stories = cx
                    .background_executor()
                    .spawn(async move { api_service.fetch_stories_concurrent(ids_to_fetch).await })
                    .await;

                let _ = entity.update(cx, |state, cx| {
                    state.stories.extend(stories);
                    // Increment loaded_count by the batch size (number of IDs attempted)
                    state.loaded_count += batch_size;
                    state.loading_more = false;
                    state.loading = false;

                    // Check if we've now loaded everything
                    if state.loaded_count >= state.story_ids.len() {
                        state.all_stories_loaded = true;
                    }

                    cx.notify();
                });
            }
        }

        pub fn select_story(entity: Entity<Self>, story_id: u32, cx: &mut App) {
            // Set initial selection & mark content loading
            entity.update(cx, |state, cx| {
                if let Some(story) = state.stories.iter().find(|s| s.id == story_id).cloned() {
                    // Add to history when selecting a story
                    state.history.add(&story);
                    state.history.save();
                    state.view_mode = ViewMode::Story(story);
                }
                state.selected_story_content = None;
                state.selected_story_content_loading = true;
                state.comments.clear();
                state.comment_ids.clear();
                state.loaded_comment_count = 0;
                state.comments_loading = false;
                cx.notify();
            });

            let story = match entity.read(cx).view_mode.clone() {
                ViewMode::Story(story) => story,
                _ => {
                    entity.update(cx, |state, cx| {
                        state.selected_story_content_loading = false;
                        cx.notify();
                    });
                    return;
                }
            };

            // Trigger comment fetching for the selected story
            Self::fetch_comments(entity.clone(), story.clone(), cx);

            match story.url.clone() {
                Some(url) => {
                    let (tx, mut rx) = mpsc::unbounded::<String>();
                    let background = cx.background_executor().clone();
                    let foreground = cx.foreground_executor().clone();
                    let mut async_cx = cx.to_async();

                    // Foreground receiver updates state when content arrives
                    foreground
                        .spawn({
                            let entity_fg = entity.clone();
                            async move {
                                if let Some(content) = rx.next().await {
                                    let _ = entity_fg.update(&mut async_cx, |state, cx| {
                                        if let ViewMode::Story(sel) = &state.view_mode
                                            && sel.id == story.id
                                        {
                                            state.selected_story_content = Some(content);
                                            state.selected_story_content_loading = false;
                                            cx.notify();
                                        }
                                    });
                                }
                            }
                        })
                        .detach();

                    // Background fetch
                    background
                        .spawn(async move {
                            let fetched = reqwest::blocking::get(&url)
                                .and_then(|resp| resp.text())
                                .unwrap_or_default();
                            let text = extract_text_from_html(&fetched);
                            let _ = tx.unbounded_send(text);
                        })
                        .detach();
                }
                None => {
                    // No URL; mark as not loading with placeholder
                    entity.update(cx, |state, cx| {
                        state.selected_story_content_loading = false;
                        state.selected_story_content = Some("(No URL for this story)".to_string());
                        cx.notify();
                    });
                }
            }
        }

        pub fn clear_selection(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.view_mode = ViewMode::List;
                state.selected_story_content = None;
                state.selected_story_content_loading = false;
                state.comments.clear();
                state.comment_ids.clear();
                state.loaded_comment_count = 0;
                state.comments_loading = false;
                cx.notify();
            });
        }

        pub fn show_webview(entity: Entity<Self>, url: String, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.view_mode = ViewMode::Webview(url);
                cx.notify();
            });
        }

        pub fn hide_webview(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                // This is a simplification. A better approach would be to remember the previous state.
                // For now, we assume hiding webview always goes back to the story detail.
                // This requires that we have the story context when we are in webview.
                // Let's adjust ViewMode to hold the story context.
                // No, let's keep it simple and just go to List for now.
                // A better way: go back to the story if it's there.
                // But AppState does not retain the story when in webview mode.
                // So, let's change `clear_selection` to go to List, and `hide_webview` needs to be smarter.
                // To do this, we need to adjust the state again.
                // What if we keep the story in the state when we go to webview?
                // `pub selected_story: Option<Story>`
                // `pub view_mode: ViewMode` where `ViewMode` is `List` or `Webview(String)`
                // Then `select_story` would set `selected_story` and not touch `view_mode`.
                // The `render` function would `if let Some(story) = state.selected_story` show detail.
                // And `if let ViewMode::Webview(url) = state.view_mode` show webview.
                // This is getting complicated. Let's stick to the investigator's suggestion for now.
                // `hide_webview` will go back to the List for now. It's a start.
                state.view_mode = ViewMode::List; // Simplified: just go back to the list
                cx.notify();
            });
        }

        pub fn fetch_comments(entity: Entity<Self>, story: Story, cx: &mut App) {
            let comment_ids = match story.kids.clone() {
                Some(ids) if !ids.is_empty() => ids,
                _ => {
                    // No comments to fetch
                    return;
                }
            };

            let api_service = entity.read(cx).api_service.clone();

            // Cancel existing comment fetch
            entity.update(cx, |state, cx| {
                state.comment_fetch_task = None;
                state.comments_loading = true;
                state.comment_ids = comment_ids.clone();
                state.loaded_comment_count = 0;
                cx.notify();
            });

            let (tx, mut rx) = mpsc::unbounded::<Vec<CommentViewModel>>();

            let entity_clone = entity.clone();
            let mut async_cx = cx.to_async();
            let background = cx.background_executor().clone();

            let task = cx.foreground_executor().spawn(async move {
                // Fetch first batch in background
                let comments = background
                    .spawn(async move {
                        Self::fetch_comments_recursive(
                            &api_service,
                            comment_ids.into_iter().take(20).collect(),
                            0,
                            3,
                        )
                        .await
                    })
                    .await;

                let _ = tx.unbounded_send(comments);

                if let Some(comments) = rx.next().await {
                    let _ = entity_clone.update(&mut async_cx, |state, cx| {
                        state.comments = comments;
                        state.loaded_comment_count = 20.min(state.comment_ids.len());
                        state.comments_loading = false;
                        cx.notify();
                    });
                }
            });

            entity.update(cx, |state, _| {
                state.comment_fetch_task = Some(task);
            });
        }

        async fn fetch_comments_recursive(
            api: &ApiService,
            ids: Vec<u32>,
            depth: u32,
            max_depth: u32,
        ) -> Vec<CommentViewModel> {
            let mut results = Vec::new();

            // Fetch comments concurrently
            let comments = api.fetch_comments_concurrent(ids).await;

            for comment in comments {
                let kids = comment.kids.clone();
                let vm = CommentViewModel {
                    id: comment.id,
                    comment: comment.clone(),
                    depth,
                    collapsed: false,
                    loading: false,
                };
                results.push(vm);

                if depth < max_depth
                    && let Some(kids_ids) = kids
                {
                    // Recursively fetch children
                    // Note: We could parallelize this too with join_all, but let's keep it simple for now
                    // as the depth is limited and branching factor can be high.
                    // Actually, since we are async, we can just await.
                    let children = Box::pin(Self::fetch_comments_recursive(
                        api,
                        kids_ids,
                        depth + 1,
                        max_depth,
                    ))
                    .await;
                    results.extend(children);
                }
            }
            results
        }

        pub fn fetch_more_comments(entity: Entity<Self>, cx: &mut App) {
            let (api_service, comment_ids_to_fetch, batch_size) = entity.update(cx, |state, cx| {
                // Check if we've already loaded all comments
                if state.loaded_comment_count >= state.comment_ids.len() {
                    state.comments_loading = false;
                    cx.notify();
                    return (None, Vec::new(), 0);
                }

                state.comments_loading = true;
                cx.notify();

                let start = state.loaded_comment_count;
                let end = (start + 20).min(state.comment_ids.len());
                let ids = state.comment_ids[start..end].to_vec();
                let batch_size = end - start;

                (Some(state.api_service.clone()), ids, batch_size)
            });

            if let Some(api_service) = api_service {
                let (tx, mut rx) = mpsc::unbounded::<Vec<CommentViewModel>>();

                let entity_clone = entity.clone();
                let mut async_cx = cx.to_async();
                let background = cx.background_executor().clone();

                let task = cx.foreground_executor().spawn(async move {
                    let comments = background
                        .spawn(async move {
                            Self::fetch_comments_recursive(&api_service, comment_ids_to_fetch, 0, 3)
                                .await
                        })
                        .await;

                    let _ = tx.unbounded_send(comments);

                    if let Some(new_comments) = rx.next().await {
                        let _ = entity_clone.update(&mut async_cx, |state, cx| {
                            state.comments.extend(new_comments);
                            state.loaded_comment_count += batch_size;
                            state.comments_loading = false;
                            cx.notify();
                        });
                    }
                });

                entity.update(cx, |state, _| {
                    state.comment_fetch_task = Some(task);
                });
            }
        }
        pub fn set_zoom_level(entity: Entity<Self>, zoom: u32, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.config.webview_zoom = zoom;
                cx.notify();
            });
        }

        pub fn set_theme_injection(entity: Entity<Self>, mode: String, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.config.webview_theme_injection = mode;
                state.config.save();
                cx.notify();
            });
        }

        /// Toggle bookmark for the current story
        pub fn toggle_bookmark(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                if let ViewMode::Story(story) = &state.view_mode {
                    state.bookmarks.toggle(story);
                    state.bookmarks.save();
                    cx.notify();
                }
            });
        }

        /// Toggle bookmark by story data (for context menus)
        pub fn toggle_bookmark_by_data(
            entity: Entity<Self>,
            id: u32,
            title: Option<String>,
            url: Option<String>,
            cx: &mut App,
        ) {
            entity.update(cx, |state, cx| {
                // Create a minimal Story object for bookmarking
                let story = Story {
                    id,
                    title,
                    url,
                    by: None,
                    time: None,
                    score: None,
                    descendants: None,
                    kids: None,
                };
                state.bookmarks.toggle(&story);
                state.bookmarks.save();
                cx.notify();
            });
        }

        /// Switch to Bookmarks view
        pub fn show_bookmarks(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.view_mode = ViewMode::Bookmarks;
                cx.notify();
            });
        }

        /// Switch to History view
        pub fn show_history(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.view_mode = ViewMode::History;
                cx.notify();
            });
        }

        /// Switch to Stories view
        pub fn show_stories(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.view_mode = ViewMode::List;
                cx.notify();
            });
        }

        /// Clear history
        pub fn clear_history(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.history.clear();
                state.history.save();
                cx.notify();
            });
        }

        pub fn set_search_query(entity: Entity<Self>, query: String, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.search_query = query;
                // Validate regex if we want to show errors immediately
                if !state.search_query.is_empty() {
                    match regex::Regex::new(&state.search_query) {
                        Ok(_) => state.regex_error = None,
                        Err(e) => state.regex_error = Some(e.to_string()),
                    }
                } else {
                    state.regex_error = None;
                }
                cx.notify();
            });
        }

        pub fn trigger_search_focus(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.should_focus_search = true;
                cx.notify();
            });
        }

        pub fn consume_search_focus(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, _cx| {
                state.should_focus_search = false;
                // No notify needed usually, or maybe yes to clear the flag?
                // But we are in render loop usually when consuming.
                // Actually if we update in render, we might trigger re-render?
                // GPUI warns about updating state during render.
                // So we should probably consume it in a `defer` or similar?
                // Or just set it to false.
            });
        }

        pub fn set_search_mode(entity: Entity<Self>, mode: SearchMode, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.search_mode = mode;
                cx.notify();
            });
        }

        pub fn set_sort_option(entity: Entity<Self>, option: SortOption, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.sort_option = option;
                cx.notify();
            });
        }

        pub fn toggle_sort_order(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.sort_order = match state.sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
                cx.notify();
            });
        }

        pub fn get_filtered_sorted_stories(&self) -> Vec<Story> {
            filter_and_sort_stories(
                &self.stories,
                &self.search_query,
                self.search_mode,
                self.sort_option,
                self.sort_order,
            )
        }

        /// Calculate which story indices should be visible based on viewport
        pub fn calculate_visible_range(
            &mut self,
            scroll_y: f32,
            viewport_height: f32,
            item_height: f32,
        ) -> (usize, usize) {
            let total_stories = self.get_filtered_sorted_stories().len();

            if total_stories == 0 {
                self.viewport_start_index = 0;
                self.viewport_end_index = 0;
                return (0, 0);
            }

            // Calculate visible range
            let start_index = (scroll_y / item_height).floor() as usize;
            let visible_items = (viewport_height / item_height).ceil() as usize + 1;
            let end_index = (start_index + visible_items).min(total_stories);

            // Apply buffer
            let buffered_start = start_index.saturating_sub(self.visible_buffer);
            let buffered_end = (end_index + self.visible_buffer).min(total_stories);

            // Update state
            self.viewport_start_index = buffered_start;
            self.viewport_end_index = buffered_end;

            (buffered_start, buffered_end)
        }

        /// Save scroll position for the current view
        pub fn save_scroll_position(&mut self, position: f32) {
            match self.view_mode {
                ViewMode::List => self.story_list_scroll_position = position,
                ViewMode::Story(_) => self.article_scroll_position = position,
                _ => {} // Other views don't persist scroll
            }
        }

        /// Get saved scroll position for the current view
        pub fn get_scroll_position(&self) -> f32 {
            match self.view_mode {
                ViewMode::List => self.story_list_scroll_position,
                ViewMode::Story(_) => self.article_scroll_position,
                _ => 0.0,
            }
        }
    }

    pub fn filter_and_sort_stories(
        stories: &[Story],
        search_query: &str,
        search_mode: SearchMode,
        sort_option: SortOption,
        sort_order: SortOrder,
    ) -> Vec<Story> {
        let mut stories = stories.to_vec();

        // Filter
        if !search_query.is_empty() {
            if let Ok(re) = regex::Regex::new(search_query) {
                stories.retain(|story| match search_mode {
                    SearchMode::Title => {
                        if let Some(title) = &story.title {
                            re.is_match(title)
                        } else {
                            false
                        }
                    }
                    SearchMode::Comments => {
                        if let Some(title) = &story.title {
                            re.is_match(title)
                        } else {
                            false
                        }
                    }
                    SearchMode::Both => {
                        let title_match = story
                            .title
                            .as_ref()
                            .map(|t| re.is_match(t))
                            .unwrap_or(false);
                        let url_match = story.url.as_ref().map(|u| re.is_match(u)).unwrap_or(false);
                        title_match || url_match
                    }
                });
            } else {
                let query = search_query.to_lowercase();
                stories.retain(|story| match search_mode {
                    SearchMode::Title => story
                        .title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&query))
                        .unwrap_or(false),
                    SearchMode::Comments => story
                        .title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&query))
                        .unwrap_or(false),
                    SearchMode::Both => {
                        let title_match = story
                            .title
                            .as_ref()
                            .map(|t| t.to_lowercase().contains(&query))
                            .unwrap_or(false);
                        let url_match = story
                            .url
                            .as_ref()
                            .map(|u| u.to_lowercase().contains(&query))
                            .unwrap_or(false);
                        title_match || url_match
                    }
                });
            }
        }

        // Sort
        stories.sort_by(|a, b| {
            let ord = match sort_option {
                SortOption::Score => a.score.unwrap_or(0).cmp(&b.score.unwrap_or(0)),
                SortOption::Comments => a.descendants.unwrap_or(0).cmp(&b.descendants.unwrap_or(0)),
                SortOption::Time => a.time.unwrap_or(0).cmp(&b.time.unwrap_or(0)),
            };
            match sort_order {
                SortOrder::Ascending => ord,
                SortOrder::Descending => ord.reverse(),
            }
        });

        stories
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::internal::models::Story;

        fn create_story(id: u32, title: &str, score: u32, comments: u32, time: u64) -> Story {
            Story {
                id,
                title: Some(title.to_string()),
                url: None,
                by: None,
                score: Some(score),
                time: Some(time as i64),
                descendants: Some(comments),
                kids: None,
            }
        }

        #[test]
        fn test_filter_by_title() {
            let stories = vec![
                create_story(1, "Rust is great", 100, 10, 1000),
                create_story(2, "Python is good", 50, 5, 2000),
                create_story(3, "Rust vs C++", 80, 20, 3000),
            ];

            let filtered = filter_and_sort_stories(
                &stories,
                "Rust",
                SearchMode::Title,
                SortOption::Score,
                SortOrder::Descending,
            );

            assert_eq!(filtered.len(), 2);
            assert_eq!(filtered[0].id, 1); // 100 score
            assert_eq!(filtered[1].id, 3); // 80 score
        }

        #[test]
        fn test_filter_regex() {
            let stories = vec![
                create_story(1, "Rust 1.0", 100, 10, 1000),
                create_story(2, "Rust 2.0", 50, 5, 2000),
                create_story(3, "Go 1.0", 80, 20, 3000),
            ];

            let filtered = filter_and_sort_stories(
                &stories,
                r"Rust \d\.\d",
                SearchMode::Title,
                SortOption::Score,
                SortOrder::Descending,
            );

            assert_eq!(filtered.len(), 2);
        }

        #[test]
        fn test_sort_by_comments() {
            let stories = vec![
                create_story(1, "A", 10, 5, 1000),
                create_story(2, "B", 10, 20, 1000),
                create_story(3, "C", 10, 10, 1000),
            ];

            let sorted = filter_and_sort_stories(
                &stories,
                "",
                SearchMode::Title,
                SortOption::Comments,
                SortOrder::Descending,
            );

            assert_eq!(sorted[0].id, 2); // 20 comments
            assert_eq!(sorted[1].id, 3); // 10 comments
            assert_eq!(sorted[2].id, 1); // 5 comments
        }

        #[test]
        fn test_sort_ascending() {
            let stories = vec![
                create_story(1, "A", 100, 10, 1000),
                create_story(2, "B", 50, 10, 1000),
                create_story(3, "C", 200, 10, 1000),
            ];

            let sorted = filter_and_sort_stories(
                &stories,
                "",
                SearchMode::Title,
                SortOption::Score,
                SortOrder::Ascending,
            );

            assert_eq!(sorted[0].id, 2); // 50
            assert_eq!(sorted[1].id, 1); // 100
            assert_eq!(sorted[2].id, 3); // 200
        }
    }
}

pub use imp::{AppState, SearchMode, SortOption, ViewMode};
