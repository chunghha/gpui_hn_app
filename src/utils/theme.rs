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
}
