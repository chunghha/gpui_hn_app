mod imp {
    use crate::api::{ApiService, StoryListType};
    use crate::internal::models::{Comment, Story};
    use crate::utils::html::extract_text_from_html;
    use futures::StreamExt;
    use futures::channel::mpsc;
    use gpui::{App, Entity, prelude::*};
    use std::sync::Arc;

    #[derive(Clone, PartialEq)]
    pub enum ViewMode {
        List,
        Story(Story),
        Webview(String),
    }

    pub struct AppState {
        pub stories: Vec<Story>,
        pub loading: bool,
        pub current_list: StoryListType,
        api_service: Arc<ApiService>,
        pub view_mode: ViewMode,
        pub selected_story_content: Option<String>,
        pub selected_story_content_loading: bool,
        pub comments: Vec<Comment>,
        pub comments_loading: bool,
        pub config: crate::config::AppConfig,
        pub story_ids: Vec<u32>,
        pub loaded_count: usize,
    }

    impl AppState {
        pub fn new(config: crate::config::AppConfig, cx: &mut App) -> Entity<Self> {
            let api_service = Arc::new(ApiService::new());
            cx.new(|_cx| Self {
                stories: Vec::new(),
                loading: false,
                current_list: StoryListType::Best,
                api_service,
                view_mode: ViewMode::List,
                selected_story_content: None,
                selected_story_content_loading: false,
                comments: Vec::new(),
                comments_loading: false,
                config,
                story_ids: Vec::new(),
                loaded_count: 0,
            })
        }

        pub fn fetch_stories(entity: Entity<Self>, list_type: StoryListType, cx: &mut App) {
            tracing::info!("Fetching story IDs for: {}", list_type);
            let api_service = entity.read(cx).api_service.clone();
            entity.update(cx, |state, cx| {
                tracing::info!("List type changed, resetting state.");
                state.loading = true;
                state.current_list = list_type;
                state.stories.clear();
                state.story_ids.clear();
                state.loaded_count = 0;
                cx.notify();
            });

            let (tx, mut rx) = mpsc::unbounded::<Vec<u32>>();
            let background = cx.background_executor().clone();
            let foreground = cx.foreground_executor().clone();
            let mut async_cx = cx.to_async();

            // Spawn foreground task to receive IDs
            foreground
                .spawn({
                    let entity = entity.clone();
                    async move {
                        if let Some(ids) = rx.next().await {
                            let _ = entity.update(&mut async_cx, |state, cx| {
                                state.story_ids = ids;
                                // Initial fetch of first batch
                                cx.notify();
                            });
                            // Trigger fetching the first batch
                            Self::fetch_more_stories(entity, &mut async_cx).await;
                        }
                    }
                })
                .detach();

            // Spawn background task to fetch IDs
            background
                .spawn(async move {
                    match api_service.fetch_story_ids(list_type) {
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
                })
                .detach();
        }

        pub async fn fetch_more_stories(entity: Entity<Self>, cx: &mut gpui::AsyncApp) {
            let (api_service, ids_to_fetch) = entity
                .update(cx, |state, cx| {
                    if state.loaded_count >= state.story_ids.len() {
                        state.loading = false;
                        cx.notify();
                        return (None, Vec::new());
                    }

                    state.loading = true;
                    cx.notify();

                    let start = state.loaded_count;
                    let end = (start + 20).min(state.story_ids.len());
                    let ids = state.story_ids[start..end].to_vec();

                    (Some(state.api_service.clone()), ids)
                })
                .ok()
                .unwrap_or((None, Vec::new()));

            if let Some(api_service) = api_service {
                let (tx, mut rx) = mpsc::unbounded::<Vec<Story>>();

                cx.background_executor()
                    .spawn(async move {
                        tracing::info!("Fetching {} story details...", ids_to_fetch.len());
                        let mut stories = Vec::new();
                        for id in ids_to_fetch {
                            if let Ok(story) = api_service.fetch_story_content(id) {
                                stories.push(story);
                            }
                        }
                        let _ = tx.unbounded_send(stories);
                    })
                    .detach();

                if let Some(new_stories) = rx.next().await {
                    let _ = entity.update(cx, |state, _cx| {
                        state.stories.extend(new_stories);
                        state.loaded_count += 20; // Approximate, or we could use new_stories.len()
                        // Correctly update loaded_count based on actual fetched batch size attempted
                        // But to be safe against failures, maybe just increment by batch size?
                        // Let's stick to the logic: we tried to load 20.
                        // Actually, better to just set loaded_count to stories.len() if we trust it,
                        // but we might have failed fetches.
                        // Let's just increment by the number of IDs we TRIED to fetch to avoid stuck loops?
                        // Or better: loaded_count should reflect the index in story_ids.
                        // So:
                        let _current_len = state.stories.len(); // This might be less if some failed
                        // We need to track where we are in story_ids.
                        // Let's rely on the previous calculation: start + 20.
                        // So we need to store the intended new count or just re-calculate.
                        // Let's just assume 20 for now as per requirement.
                        // A cleaner way:
                    });
                    // Re-read state to update loaded_count correctly
                    let _ = entity.update(cx, |state, cx| {
                        let new_count = (state.loaded_count + 20).min(state.story_ids.len());
                        state.loaded_count = new_count;
                        state.loading = false;

                        if let ViewMode::Story(sel) = &state.view_mode
                            && !state.stories.iter().any(|s| s.id == sel.id)
                        {
                            // selection is gone, should not happen
                        }
                        cx.notify();
                    });
                }
            }
        }

        pub fn select_story(entity: Entity<Self>, story_id: u32, cx: &mut App) {
            // Set initial selection & mark content loading
            entity.update(cx, |state, cx| {
                if let Some(story) = state.stories.iter().find(|s| s.id == story_id).cloned() {
                    state.view_mode = ViewMode::Story(story);
                }
                state.selected_story_content = None;
                state.selected_story_content_loading = true;
                state.comments.clear();
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

            if let Some(url) = story.url.clone() {
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
            } else {
                // No URL; mark as not loading with placeholder
                entity.update(cx, |state, cx| {
                    state.selected_story_content_loading = false;
                    state.selected_story_content = Some("(No URL for this story)".to_string());
                    cx.notify();
                });
            }
        }

        pub fn clear_selection(entity: Entity<Self>, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.view_mode = ViewMode::List;
                state.selected_story_content = None;
                state.selected_story_content_loading = false;
                state.comments.clear();
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
            entity.update(cx, |state, cx| {
                state.comments_loading = true;
                cx.notify();
            });

            let (tx, mut rx) = mpsc::unbounded::<Vec<Comment>>();
            let background = cx.background_executor().clone();
            let foreground = cx.foreground_executor().clone();
            let mut async_cx = cx.to_async();

            // Foreground task to receive comments
            foreground
                .spawn({
                    let entity = entity.clone();
                    async move {
                        if let Some(comments) = rx.next().await {
                            let _ = entity.update(&mut async_cx, |state, cx| {
                                state.comments = comments;
                                state.comments_loading = false;
                                cx.notify();
                            });
                        }
                    }
                })
                .detach();

            // Background task to fetch comments
            background
                .spawn(async move {
                    let mut comments = Vec::new();
                    // Fetch top-level comments (limit to first 20 to avoid overwhelming)
                    for id in comment_ids.into_iter().take(20) {
                        match api_service.fetch_comment_content(id) {
                            Ok(comment) => {
                                comments.push(comment);
                            }
                            Err(e) => {
                                tracing::error!("Failed to fetch comment {}: {}", id, e);
                            }
                        }
                    }
                    let _ = tx.unbounded_send(comments);
                })
                .detach();
        }
        pub fn set_zoom_level(entity: Entity<Self>, zoom: u32, cx: &mut App) {
            entity.update(cx, |state, cx| {
                state.config.webview_zoom = zoom;
                cx.notify();
            });
        }
    }
}

pub use imp::{AppState, ViewMode};
