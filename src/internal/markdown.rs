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
pub fn render_markdown(text: &str, style: MarkdownStyle) -> impl IntoElement {
    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, text, &options);

    div()
        .flex()
        .flex_col()
        .gap_3()
        .w_full()
        .children(iter_nodes(root, &style))
}

fn iter_nodes<'a>(node: &'a AstNode<'a>, style: &MarkdownStyle) -> Vec<gpui::AnyElement> {
    let mut elements = Vec::new();

    for child in node.children() {
        if let Some(element) = render_node(child, style) {
            elements.push(element);
        }
    }

    elements
}

fn render_node<'a>(node: &'a AstNode<'a>, style: &MarkdownStyle) -> Option<gpui::AnyElement> {
    match &node.data.borrow().value {
        NodeValue::Paragraph => {
            let text = collect_text(node);
            if text.trim().is_empty() {
                return None;
            }

            Some(
                div()
                    .text_color(style.text_color)
                    .text_base()
                    .font_family(style.font_sans.clone())
                    .child(render_inline_content(node, style))
                    .into_any_element(),
            )
        }
        NodeValue::Heading(heading) => {
            let text = collect_text(node);
            let (size, weight) = match heading.level {
                1 => (px(24.0), FontWeight(700.0)),
                2 => (px(20.0), FontWeight(600.0)),
                3 => (px(18.0), FontWeight(600.0)),
                _ => (px(16.0), FontWeight(600.0)),
            };

            Some(
                div()
                    .text_color(style.text_color)
                    .text_size(size)
                    .font_weight(weight)
                    .font_family(style.font_sans.clone())
                    .mt_2()
                    .mb_1()
                    .child(text)
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
                    .child(code)
                    .into_any_element(),
            )
        }
        NodeValue::List(_) => {
            let items: Vec<_> = node
                .children()
                .filter_map(|child| render_node(child, style))
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
            let content = collect_text(node);

            Some(
                div()
                    .flex()
                    .gap_2()
                    .text_color(style.text_color)
                    .font_family(style.font_sans.clone())
                    .child("•")
                    .child(content)
                    .into_any_element(),
            )
        }
        _ => None,
    }
}

fn render_inline_content<'a>(node: &'a AstNode<'a>, style: &MarkdownStyle) -> gpui::AnyElement {
    let mut parts = Vec::new();
    collect_inline_parts(node, &mut parts, style);

    div()
        .flex()
        .flex_wrap()
        .gap_1()
        .children(parts)
        .into_any_element()
}

fn collect_inline_parts<'a>(
    node: &'a AstNode<'a>,
    parts: &mut Vec<gpui::AnyElement>,
    style: &MarkdownStyle,
) {
    match &node.data.borrow().value {
        NodeValue::Text(text) => {
            let content = text.clone().to_string();
            parts.push(div().child(content).into_any_element());
        }
        NodeValue::Strong => {
            let text = collect_text(node);
            parts.push(
                div()
                    .font_weight(FontWeight(700.0))
                    .child(text)
                    .into_any_element(),
            );
        }
        NodeValue::Emph => {
            let text = collect_text(node);
            parts.push(div().italic().child(text).into_any_element());
        }
        NodeValue::Code(code) => {
            let code_text = code.literal.clone();
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
                    .child(code_text)
                    .into_any_element(),
            );
        }
        NodeValue::Link(link) => {
            let url = link.url.clone();
            let text = collect_text(node);
            let display = if text.trim().is_empty() { url } else { text };

            parts.push(
                div()
                    .text_color(style.link_color)
                    .underline()
                    .child(display)
                    .into_any_element(),
            );
        }
        _ => {
            for child in node.children() {
                collect_inline_parts(child, parts, style);
            }
        }
    }
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
