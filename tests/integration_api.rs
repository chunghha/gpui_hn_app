use gpui_hn_app::api::{ApiService, StoryListType};
use serde_json::json;
use tokio::runtime::Runtime;

#[test]
fn test_fetch_story_list_integration() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/topstories.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!([1, 2, 3, 4, 5]).to_string())
        .create();

    let service = ApiService::with_base_url(format!("{}/", server.url()));
    drop(_guard);

    let result = service.fetch_story_ids(StoryListType::Top);

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_fetch_story_details_integration() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let mut server = mockito::Server::new();
    let story_json = json!({
        "id": 100,
        "title": "Integration Test Story",
        "by": "tester",
        "type": "story",
        "time": 1234567890
    });

    let mock = server
        .mock("GET", "/item/100.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(story_json.to_string())
        .create();

    let service = ApiService::with_base_url(format!("{}/", server.url()));
    drop(_guard);

    let result = service.fetch_story_content(100);

    mock.assert();
    assert!(result.is_ok());
    let story = result.unwrap();
    assert_eq!(story.id, 100);
    assert_eq!(story.title, Some("Integration Test Story".to_string()));
}
