//! The design tokens: both palettes exist, differ, and meet WCAG contrast targets.

use pef_gui::theme::{Appearance, Theme, contrast_ratio, relative_luminance};

#[test]
fn palettes_differ_and_toggle() {
    let light = Theme::light();
    let dark = Theme::dark();
    assert_eq!(light.appearance, Appearance::Light);
    assert_eq!(dark.appearance, Appearance::Dark);
    assert_ne!(light.background, dark.background);
    assert_eq!(Appearance::Light.toggled(), Appearance::Dark);
    assert_eq!(Appearance::Dark.toggled(), Appearance::Light);
    assert_eq!(
        Theme::for_appearance(Appearance::Dark).background,
        dark.background
    );
    let names: Vec<&str> = light.colors().iter().map(|(name, _)| *name).collect();
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
    assert_eq!(light.colors().len(), dark.colors().len());
}

#[test]
fn contrast_math_is_sane() {
    let white = gpui::white();
    let black = gpui::black();
    assert!((relative_luminance(white) - 1.0).abs() < 0.001);
    assert!(relative_luminance(black).abs() < 0.001);
    assert!((contrast_ratio(white, black) - 21.0).abs() < 0.01);
    assert!((contrast_ratio(black, white) - contrast_ratio(white, black)).abs() < 0.0001);
    assert!((contrast_ratio(white, white) - 1.0).abs() < 0.0001);
}

#[test]
fn text_meets_wcag_aa_on_every_surface() {
    for theme in [Theme::light(), Theme::dark()] {
        let name = format!("{:?}", theme.appearance);
        for surface in [
            theme.background,
            theme.background_secondary,
            theme.surface,
            theme.hover,
            theme.pressed,
        ] {
            assert!(
                contrast_ratio(theme.text, surface) >= 7.0,
                "{name}: text on surfaces must reach AAA"
            );
            assert!(
                contrast_ratio(theme.text_secondary, surface) >= 4.5,
                "{name}: secondary text must reach AA"
            );
            assert!(
                contrast_ratio(theme.text_tertiary, surface) >= 3.0,
                "{name}: tertiary text must reach AA for large/UI text"
            );
            assert!(
                contrast_ratio(theme.accent, surface) >= 3.0,
                "{name}: accent must be distinguishable"
            );
        }
        assert!(
            contrast_ratio(theme.primary_foreground, theme.primary) >= 7.0,
            "{name}: primary button label"
        );
        assert!(
            contrast_ratio(theme.primary_foreground, theme.primary_hover) >= 4.5,
            "{name}: hovered primary button label"
        );
        assert!(
            contrast_ratio(theme.primary_foreground, theme.error) >= 4.5,
            "{name}: danger button label"
        );
        assert!(
            contrast_ratio(theme.success, theme.success_subtle) >= 4.5,
            "{name}: success note"
        );
        assert!(
            contrast_ratio(theme.warning, theme.warning_subtle) >= 4.5,
            "{name}: warning note"
        );
        assert!(
            contrast_ratio(theme.error, theme.error_subtle) >= 4.5,
            "{name}: error note"
        );
        assert!(
            contrast_ratio(theme.accent, theme.accent_subtle) >= 4.5,
            "{name}: blue badge"
        );
        assert!(
            contrast_ratio(theme.border, theme.background) >= 1.1,
            "{name}: hairline border must be visible"
        );
        assert!(
            contrast_ratio(theme.border_strong, theme.background) >= 1.5,
            "{name}: strong border"
        );
    }
}
