use gpui_hn_app::internal::models::{Comment, Story};
use proptest::prelude::*;
use serde_json::json;

// Strategy for generating valid story JSON objects
fn story_json_strategy() -> impl Strategy<Value = serde_json::Value> {
    (
        any::<u32>(),                               // id
        prop::option::of("[a-zA-Z0-9 ]{1,100}"),    // title
        prop::option::of("[a-zA-Z0-9_]{1,20}"),     // by (username)
        prop::option::of(0u32..10000),              // score
        prop::option::of(0u32..1000),               // descendants
        prop::option::of(0i64..2000000000i64),      // time
        prop::collection::vec(any::<u32>(), 0..10), // kids
    )
        .prop_map(|(id, title, by, score, descendants, time, kids)| {
            let mut obj = json!({
                "id": id,
                "type": "story"
            });
            if let Some(t) = title {
                obj["title"] = json!(t);
            }
            if let Some(b) = by {
                obj["by"] = json!(b);
            }
            if let Some(s) = score {
                obj["score"] = json!(s);
            }
            if let Some(d) = descendants {
                obj["descendants"] = json!(d);
            }
            if let Some(t) = time {
                obj["time"] = json!(t);
            }
            if !kids.is_empty() {
                obj["kids"] = json!(kids);
            }
            obj
        })
}

// Strategy for generating valid comment JSON objects
fn comment_json_strategy() -> impl Strategy<Value = serde_json::Value> {
    (
        any::<u32>(),                               // id
        prop::option::of("[a-zA-Z0-9_]{1,20}"),     // by (username)
        prop::option::of("[a-zA-Z0-9 <>&]{0,500}"), // text (HTML content)
        prop::option::of(any::<u32>()),             // parent
        prop::option::of(0i64..2000000000i64),      // time
        prop::collection::vec(any::<u32>(), 0..5),  // kids
    )
        .prop_map(|(id, by, text, parent, time, kids)| {
            let mut obj = json!({
                "id": id,
                "type": "comment"
            });
            if let Some(b) = by {
                obj["by"] = json!(b);
            }
            if let Some(t) = text {
                obj["text"] = json!(t);
            }
            if let Some(p) = parent {
                obj["parent"] = json!(p);
            }
            if let Some(t) = time {
                obj["time"] = json!(t);
            }
            if !kids.is_empty() {
                obj["kids"] = json!(kids);
            }
            obj
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn story_deserialization_never_panics(story_json in story_json_strategy()) {
        let json_str = story_json.to_string();
        // Should not panic - may return Ok or Err, but no panic
        let _result: Result<Story, _> = serde_json::from_str(&json_str);
    }

    #[test]
    fn comment_deserialization_never_panics(comment_json in comment_json_strategy()) {
        let json_str = comment_json.to_string();
        // Should not panic - may return Ok or Err, but no panic
        let _result: Result<Comment, _> = serde_json::from_str(&json_str);
    }

    #[test]
    fn story_id_is_preserved(id in any::<u32>()) {
        let story_json = json!({
            "id": id,
            "type": "story"
        });
        let story: Story = serde_json::from_str(&story_json.to_string()).unwrap();
        prop_assert_eq!(story.id, id);
    }

    #[test]
    fn comment_id_is_preserved(id in any::<u32>()) {
        let comment_json = json!({
            "id": id,
            "type": "comment"
        });
        let comment: Comment = serde_json::from_str(&comment_json.to_string()).unwrap();
        prop_assert_eq!(comment.id, id);
    }

    #[test]
    fn malformed_json_does_not_panic(garbage in "[a-zA-Z0-9{}:,\"\\[\\]]{0,100}") {
        // Any random string should not cause a panic, just an error
        let _result: Result<Story, _> = serde_json::from_str(&garbage);
        let _result2: Result<Comment, _> = serde_json::from_str(&garbage);
    }
}
