use html2text::from_read;

/// Extracts readable text from an HTML string.
/// Strips tags and decodes basic entities using `html2text` crate.
pub fn extract_text_from_html(html: &str) -> String {
    // html2text emits wrapped lines; we can join them for now.
    let mut bytes = html.as_bytes();
    from_read(&mut bytes, 80).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_text() {
        let html = "<p>Hello <strong>World</strong> &amp; friends</p>";
        let out = extract_text_from_html(html);
        assert!(out.contains("Hello"));
        assert!(out.contains("World"));
        assert!(out.contains("& friends") || out.contains("& friends"));
    }
}
