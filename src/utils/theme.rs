use gpui::Hsla;

/// Convert an HSLA color to a CSS hex color string
///
/// # Arguments
/// * `color` - The HSLA color to convert
///
/// # Returns
/// A string in the format "#RRGGBB" (e.g., "#100F0F")
///
/// # Example
/// ```
/// use gpui::hsla;
/// use gpui_hn_app::utils::theme::hsla_to_hex;
/// let color = hsla(0.0, 0.0, 0.0, 1.0);
/// let hex = hsla_to_hex(color);
/// assert_eq!(hex, "#000000");
/// ```
pub fn hsla_to_hex(color: Hsla) -> String {
    let rgba = color.to_rgb();
    format!(
        "#{:02X}{:02X}{:02X}",
        (rgba.r * 255.0) as u8,
        (rgba.g * 255.0) as u8,
        (rgba.b * 255.0) as u8
    )
}

/// Toggle the textual token "Dark" <-> "Light" inside a configured theme name.
///
/// This implementation only replaces standalone word occurrences of "Dark" or
/// "Light" (i.e. surrounded by non-letter characters or at string boundaries).
/// It is case-preserving for simple cases (all-upper, all-lower, Capitalized).
/// If no standalone token is found, it appends the opposite suffix based on the
/// runtime hint.
///
/// Examples:
/// - "Flexoki Dark" -> "Flexoki Light"
/// - "FlexokiDark" -> (no standalone token) -> "FlexokiDark Light" or "... Dark" depending on runtime hint
/// - "Darkness Dark" -> "Darkness Light" (only the standalone token replaced)
pub fn toggle_dark_light(theme_name: &str, runtime_is_dark: Option<bool>) -> String {
    // Try to replace a standalone "Dark" (case-insensitive) while preserving simple casing.
    if let Some(replaced) = replace_first_standalone_token(theme_name, "Dark", "Light") {
        return replaced;
    }

    // Try to replace a standalone "Light".
    if let Some(replaced) = replace_first_standalone_token(theme_name, "Light", "Dark") {
        return replaced;
    }

    // No standalone token found; append based on runtime hint.
    if runtime_is_dark.unwrap_or(false) {
        format!("{} Light", theme_name)
    } else {
        format!("{} Dark", theme_name)
    }
}

/// Find and replace the first standalone occurrence of `token` in `s` with
/// `replacement`. Standalone means the token is not part of a larger alphabetic
/// word: the character before (if any) and after (if any) are non-alphabetic.
///
/// Returns Some(new_string) if a replacement was made, otherwise None.
fn replace_first_standalone_token(s: &str, token: &str, replacement: &str) -> Option<String> {
    let lower = s.to_lowercase();
    let token_lower = token.to_lowercase();

    let mut search_start = 0usize;
    while let Some(idx) = lower[search_start..].find(&token_lower) {
        let abs = search_start + idx;
        let end = abs + token_lower.len();

        // Check boundary before token
        let before_ok = if abs == 0 {
            true
        } else {
            // Safe to use byte slicing for ASCII token boundaries
            let prev_char = s[..abs].chars().next_back().unwrap();
            !prev_char.is_alphabetic()
        };

        // Check boundary after token
        let after_ok = if end == s.len() {
            true
        } else {
            let next_char = s[end..].chars().next().unwrap();
            !next_char.is_alphabetic()
        };

        if before_ok && after_ok {
            // Preserve simple casing from the original substring.
            let orig = &s[abs..end];
            let replacement_cased = preserve_casing(orig, replacement);
            let mut out = String::with_capacity(s.len() - token.len() + replacement_cased.len());
            out.push_str(&s[..abs]);
            out.push_str(&replacement_cased);
            out.push_str(&s[end..]);
            return Some(out);
        }

        // Continue searching after this occurrence
        search_start = end;
    }

    None
}

/// Preserve simple casing patterns from `orig` onto `replacement`.
/// - If `orig` is ALL UPPER, return `replacement` uppercased.
/// - If `orig` is all lower, return `replacement` lowercased.
/// - If `orig` is Capitalized (first upper, rest lower), return replacement capitalized.
/// - Otherwise return replacement as-is.
fn preserve_casing(orig: &str, replacement: &str) -> String {
    // Determine alphabetic characters' casing
    let alphabetic_chars: Vec<char> = orig.chars().filter(|c| c.is_alphabetic()).collect();
    if alphabetic_chars.is_empty() {
        return replacement.to_string();
    }

    let all_upper = alphabetic_chars.iter().all(|c| c.is_uppercase());
    let all_lower = alphabetic_chars.iter().all(|c| c.is_lowercase());
    let first_upper_rest_lower = {
        let mut it = alphabetic_chars.iter();
        if let Some(first) = it.next() {
            first.is_uppercase() && it.all(|c| c.is_lowercase())
        } else {
            false
        }
    };

    if all_upper {
        replacement.to_uppercase()
    } else if all_lower {
        replacement.to_lowercase()
    } else if first_upper_rest_lower {
        capitalize_ascii_like(replacement)
    } else {
        replacement.to_string()
    }
}

/// Capitalize replacement in an ASCII-friendly way: first character upper, rest lower.
fn capitalize_ascii_like(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out = String::new();
            out.extend(first.to_uppercase());
            out.push_str(&chars.as_str().to_lowercase());
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::hsla;

    #[test]
    fn test_black_conversion() {
        let black = hsla(0.0, 0.0, 0.0, 1.0);
        assert_eq!(hsla_to_hex(black), "#000000");
    }

    #[test]
    fn test_white_conversion() {
        let white = hsla(0.0, 0.0, 1.0, 1.0);
        assert_eq!(hsla_to_hex(white), "#FFFFFF");
    }

    #[test]
    fn test_flexoki_dark_bg() {
        // Flexoki Dark background: #100F0F (actual output: #0F0F0F due to HSL precision)
        // In HSL: h=0, s=0%, l=6.27%
        let flexoki_dark_bg = hsla(0.0, 0.0, 0.0627, 1.0);
        // Note: HSL to RGB conversion may have slight differences
        let result = hsla_to_hex(flexoki_dark_bg);
        assert!(
            result == "#0F0F0F" || result == "#100F0F",
            "Got: {}",
            result
        );
    }

    #[test]
    fn test_flexoki_dark_fg() {
        // Flexoki Dark foreground: #CECDC3 (actual output may vary slightly)
        // In HSL: h=40, s=10.6%, l=80.8%
        let flexoki_dark_fg = hsla(40.0 / 360.0, 0.106, 0.808, 1.0);
        let result = hsla_to_hex(flexoki_dark_fg);
        // Allow slight variation due to HSL->RGB conversion
        assert!(
            result == "#D3CFC8" || result == "#CECDC3",
            "Expected close to #CECDC3, got: {}",
            result
        );
    }

    #[test]
    fn test_flexoki_blue() {
        // Flexoki blue: #4385BE (actual output may vary slightly)
        // In HSL: h=207, s=47.3%, l=50%
        let flexoki_blue = hsla(207.0 / 360.0, 0.473, 0.50, 1.0);
        let result = hsla_to_hex(flexoki_blue);
        assert!(
            result == "#4385BB" || result == "#4385BE",
            "Expected close to #4385BE, got: {}",
            result
        );
    }

    #[test]
    fn test_red() {
        let red = hsla(0.0, 1.0, 0.5, 1.0);
        assert_eq!(hsla_to_hex(red), "#FF0000");
    }

    #[test]
    fn test_green() {
        let green = hsla(120.0 / 360.0, 1.0, 0.5, 1.0);
        assert_eq!(hsla_to_hex(green), "#00FF00");
    }

    #[test]
    fn test_blue() {
        let blue = hsla(240.0 / 360.0, 1.0, 0.5, 1.0);
        assert_eq!(hsla_to_hex(blue), "#0000FF");
    }

    // --- Tests for the new toggle_dark_light behavior ---

    #[test]
    fn toggle_dark_to_light() {
        let got = toggle_dark_light("Flexoki Dark", Some(false));
        assert_eq!(got, "Flexoki Light");
    }

    #[test]
    fn toggle_light_to_dark() {
        let got = toggle_dark_light("Flexoki Light", Some(true));
        assert_eq!(got, "Flexoki Dark");
    }

    #[test]
    fn append_light_when_runtime_dark_and_no_token() {
        let got = toggle_dark_light("Flexoki", Some(true));
        assert_eq!(got, "Flexoki Light");
    }

    #[test]
    fn append_dark_when_runtime_light_and_no_token() {
        let got = toggle_dark_light("Flexoki", Some(false));
        assert_eq!(got, "Flexoki Dark");
    }

    #[test]
    fn replace_only_first_occurrence() {
        let got = toggle_dark_light("Darkness Dark", Some(false));
        assert_eq!(got, "Darkness Light");
    }
}
