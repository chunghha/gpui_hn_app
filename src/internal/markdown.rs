use crate::config::AppConfig;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use gpui::{FontWeight, Hsla, SharedString, div, prelude::*, px};

#[derive(Clone)]
pub struct MarkdownStyle {
    pub text_color: Hsla,
    pub link_color: Hsla,
    pub code_bg_color: Hsla,
    pub font_sans: SharedString,
    pub font_mono: SharedString,
}

/// Parse markdown text and render it as GPUI elements
pub fn render_markdown(text: &str, style: MarkdownStyle, config: &AppConfig) -> impl IntoElement {
    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, text, &options);

    div()
        .flex()
        .flex_col()
        .gap_3()
        .children(iter_nodes(root, &style, config))
}

fn iter_nodes<'a>(
    node: &'a AstNode<'a>,
    style: &MarkdownStyle,
    config: &AppConfig,
) -> Vec<gpui::AnyElement> {
    let mut elements = Vec::new();

    for child in node.children() {
        if let Some(element) = render_node(child, style, config) {
            elements.push(element);
        }
    }

    elements
}

fn render_node<'a>(
    node: &'a AstNode<'a>,
    style: &MarkdownStyle,
    config: &AppConfig,
) -> Option<gpui::AnyElement> {
    match &node.data.borrow().value {
        NodeValue::Paragraph => {
            if collect_text(node).trim().is_empty() {
                return None;
            }

            // Make paragraph a full-width flex row with wrapping so inline parts can wrap
            Some(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .text_color(style.text_color)
                    .text_base()
                    .font_family(style.font_sans.clone())
                    .child(render_inline_content(node, style, config))
                    .into_any_element(),
            )
        }
        NodeValue::Heading(heading) => {
            let (size, weight) = match heading.level {
                1 => (px(24.0), FontWeight(700.0)),
                2 => (px(20.0), FontWeight(600.0)),
                3 => (px(18.0), FontWeight(600.0)),
                _ => (px(16.0), FontWeight(600.0)),
            };

            Some(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .text_color(style.text_color)
                    .text_size(size)
                    .font_weight(weight)
                    .font_family(style.font_sans.clone())
                    .mt_2()
                    .mb_1()
                    .child(render_inline_content(node, style, config))
                    .into_any_element(),
            )
        }
        NodeValue::CodeBlock(code_block) => {
            let code = code_block.literal.clone();

            Some(
                div()
                    .bg(style.code_bg_color)
                    .rounded_md()
                    .p_3()
                    .my_2()
                    .font_family(style.font_mono.clone())
                    .text_sm()
                    .text_color(style.text_color)
                    .child(soft_wrap(&code, config.soft_wrap_max_run))
                    .into_any_element(),
            )
        }
        NodeValue::List(_) => {
            let items: Vec<_> = node
                .children()
                .filter_map(|child| render_node(child, style, config))
                .collect();

            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pl_4()
                    .children(items)
                    .into_any_element(),
            )
        }
        NodeValue::Item(_) => {
            // Render list item using the inline renderer so that long
            // runs are split into smaller elements and can wrap.
            Some(
                div()
                    .flex()
                    .gap_2()
                    .text_color(style.text_color)
                    .font_family(style.font_sans.clone())
                    .child("•")
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .w_full()
                            .child(render_inline_content(node, style, config)),
                    )
                    .into_any_element(),
            )
        }
        _ => None,
    }
}

fn render_inline_content<'a>(
    node: &'a AstNode<'a>,
    style: &MarkdownStyle,
    config: &AppConfig,
) -> gpui::AnyElement {
    let mut parts = Vec::new();
    collect_inline_parts(node, &mut parts, style, config);

    div().flex().flex_wrap().children(parts).into_any_element()
}

fn collect_inline_parts<'a>(
    node: &'a AstNode<'a>,
    parts: &mut Vec<gpui::AnyElement>,
    style: &MarkdownStyle,
    config: &AppConfig,
) {
    match &node.data.borrow().value {
        NodeValue::Text(text) => {
            let content = text.clone().to_string();
            let wrapped = soft_wrap(&content, config.soft_wrap_max_run);
            for word in split_into_words(&wrapped) {
                parts.push(div().child(word).into_any_element());
            }
        }
        NodeValue::Strong => {
            let text = collect_text(node);
            for word in split_into_words(&text) {
                parts.push(
                    div()
                        .font_weight(FontWeight(700.0))
                        .child(word)
                        .into_any_element(),
                );
            }
        }
        NodeValue::Emph => {
            let text = collect_text(node);
            for word in split_into_words(&text) {
                parts.push(div().italic().child(word).into_any_element());
            }
        }
        NodeValue::Code(code) => {
            let code_text = code.literal.clone();
            let wrapped = soft_wrap(&code_text, config.soft_wrap_max_run);
            parts.push(
                div()
                    .bg(Hsla {
                        h: 0.0,
                        s: 0.0,
                        l: 0.15,
                        a: 1.0,
                    })
                    .rounded_sm()
                    .px_1()
                    .font_family(style.font_mono.clone())
                    .text_sm()
                    .child(wrapped)
                    .into_any_element(),
            );
        }
        NodeValue::Link(link) => {
            let url = link.url.clone();
            let text = collect_text(node);
            let display = if text.trim().is_empty() {
                soft_wrap(&url, config.soft_wrap_max_run)
            } else {
                soft_wrap(&text, config.soft_wrap_max_run)
            };

            for word in split_into_words(&display) {
                parts.push(
                    div()
                        .text_color(style.link_color)
                        .underline()
                        .child(word)
                        .into_any_element(),
                );
            }
        }
        _ => {
            for child in node.children() {
                collect_inline_parts(child, parts, style, config);
            }
        }
    }
}

fn split_into_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current_word = String::new();

    for c in s.chars() {
        current_word.push(c);
        if c.is_whitespace() {
            words.push(current_word);
            current_word = String::new();
        }
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
}

pub fn soft_wrap(s: &str, max_run: usize) -> String {
    // insert a regular space after long runs of non-whitespace characters to allow wrapping
    // made public so other modules (e.g. StoryDetailView) can reuse it to ensure
    // long inline content wraps instead of causing horizontal overflow.
    // Inserting an actual space gives the layout engine a stronger break point
    // when soft break opportunities (ZWSP/soft-hyphen) are not honored.
    // Use a smaller run length and insert both zero-width space and
    // soft-hyphen as break opportunities. Soft-hyphen (U+00AD) will
    // show a hyphen when a break occurs which improves readability
    // for very long compound words or URLs.

    // If configuration explicitly disabled soft-wrapping (0), return original string.
    if max_run == 0 {
        return s.to_string();
    }

    // Use the configured run length to size capacity and insertion frequency.
    let mut out = String::with_capacity(s.len() + s.len() / max_run + 4);
    let mut run = 0usize;

    for ch in s.chars() {
        out.push(ch);
        if ch.is_whitespace() {
            run = 0;
        } else {
            run += 1;
            if run >= max_run {
                // insert a zero-width space and a soft-hyphen to offer
                // multiple breakpoints. Order matters: ZWSP allows
                // a clean break without visible characters; soft-hyphen
                // is a fallback that shows a hyphen where a break occurs.
                out.push('\u{200B}');
                out.push('\u{00AD}');
                run = 0;
            }
        }
    }

    out
}

fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    collect_text_recursive(node, &mut text);
    text
}

fn collect_text_recursive<'a>(node: &'a AstNode<'a>, text: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(bytes) => {
            text.push_str(bytes);
        }
        NodeValue::Code(code) => {
            text.push_str(&code.literal);
        }
        _ => {
            for child in node.children() {
                collect_text_recursive(child, text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::soft_wrap;

    #[test]
    fn soft_wrap_disabled_when_zero() {
        // A long run of characters without whitespace
        let input = "abcdefghijklmnopqrstuvwxyz0123456789";
        // When max_run is 0, soft_wrap should be a no-op and return the original string
        let out = soft_wrap(input, 0);
        assert_eq!(out, input, "soft_wrap should be disabled when max_run == 0");
    }

    #[test]
    fn soft_wrap_inserts_breaks_for_small_max_run() {
        // Use a string longer than the max_run so we can observe insertions
        let input = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 40 'a'
        // Choose a small max_run to force multiple insertions
        let out = soft_wrap(input, 5);

        // soft_wrap inserts zero-width space and soft-hyphen characters
        let has_zwsp = out.contains('\u{200B}');
        let has_shy = out.contains('\u{00AD}');
        assert!(
            has_zwsp || has_shy,
            "soft_wrap did not insert expected break characters: {}",
            out
        );

        // The output should be longer than the input due to inserted characters
        assert!(
            out.len() > input.len(),
            "expected output to be longer after insertions"
        );

        // Ensure that the breaks are not inserted at the very start
        let first_non_a = out.find(|c: char| c != 'a');
        assert!(
            first_non_a.is_some() && first_non_a.unwrap() > 0,
            "break characters should not appear at the start"
        );
    }
}
