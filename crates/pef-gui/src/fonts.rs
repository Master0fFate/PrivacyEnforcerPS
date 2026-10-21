//! The embedded Geist typeface (SIL Open Font License 1.1; see `assets/fonts/OFL.txt`).

use std::borrow::Cow;

use gpui::App;

/// Registers Geist Sans and Geist Mono with the text system.
///
/// Failure is not fatal: the platform's default font is used instead.
pub fn load(cx: &App) -> Result<(), String> {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-SemiBold.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Regular.ttf")),
    ];
    cx.text_system()
        .add_fonts(fonts)
        .map_err(|err| format!("could not load the embedded fonts: {err:#}"))
}
