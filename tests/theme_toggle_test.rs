#[test]
fn test_ghost_in_the_shell_toggle() {
    use gpui_hn_app::utils::theme::toggle_dark_light;

    let dark = "Ghost in the Shell Dark";
    let light = "Ghost in the Shell Light";

    // Test Dark -> Light
    let toggled_light = toggle_dark_light(dark, Some(true));
    assert_eq!(toggled_light, light, "Failed to toggle Dark -> Light");

    // Test Light -> Dark
    let toggled_dark = toggle_dark_light(light, Some(false));
    assert_eq!(toggled_dark, dark, "Failed to toggle Light -> Dark");
}

#[test]
fn test_ghost_in_the_shell_lowercase_toggle() {
    use gpui_hn_app::utils::theme::toggle_dark_light;

    // Simulate user error or manual config with lowercase 's'
    let dark = "Ghost in the shell Dark";
    let light = "Ghost in the shell Light";

    // Test Dark -> Light
    let toggled_light = toggle_dark_light(dark, Some(true));
    assert_eq!(
        toggled_light, light,
        "Failed to toggle lowercase Dark -> Light"
    );

    // Test Light -> Dark
    let toggled_dark = toggle_dark_light(light, Some(false));
    assert_eq!(
        toggled_dark, dark,
        "Failed to toggle lowercase Light -> Dark"
    );
}
