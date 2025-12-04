use criterion::{Criterion, black_box, criterion_group, criterion_main};
use gpui_hn_app::internal::models::Story;
use serde_json::json;

fn bench_story_deserialization(c: &mut Criterion) {
    let story_json = json!({
        "id": 12345,
        "title": "A sample Hacker News story title that is reasonably long",
        "by": "username",
        "type": "story",
        "time": 1234567890,
        "score": 150,
        "descendants": 42,
        "url": "https://example.com/some/long/path/to/article",
        "kids": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    })
    .to_string();

    c.bench_function("deserialize_story", |b| {
        b.iter(|| {
            let _story: Story = serde_json::from_str(black_box(&story_json)).unwrap();
        })
    });
}

fn bench_large_story_list(c: &mut Criterion) {
    // Simulate a list of 500 story IDs
    let story_ids: Vec<u32> = (1..=500).collect();
    let ids_json = serde_json::to_string(&story_ids).unwrap();

    c.bench_function("deserialize_500_story_ids", |b| {
        b.iter(|| {
            let _ids: Vec<u32> = serde_json::from_str(black_box(&ids_json)).unwrap();
        })
    });
}

fn bench_html_text_extraction(c: &mut Criterion) {
    let html_content = r#"
        <p>This is a paragraph with <a href="https://example.com">a link</a> and some <b>bold text</b>.</p>
        <p>Another paragraph with <i>italic</i> and <code>inline code</code>.</p>
        <pre><code>fn main() {
    println!("Hello, world!");
}</code></pre>
        <p>Final paragraph with an unordered list:</p>
        <ul>
            <li>Item 1</li>
            <li>Item 2</li>
            <li>Item 3</li>
        </ul>
    "#;

    c.bench_function("html_to_text_extraction", |b| {
        b.iter(|| {
            let _text = html2text::from_read(black_box(html_content.as_bytes()), 80);
        })
    });
}

criterion_group!(
    benches,
    bench_story_deserialization,
    bench_large_story_list,
    bench_html_text_extraction
);
criterion_main!(benches);
