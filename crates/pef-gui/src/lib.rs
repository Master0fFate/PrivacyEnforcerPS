//! The Privacy Enforcer desktop application.
//!
//! Built with GPUI Fast (longbridge's performance fork of Zed's GPUI) and a small
//! component library in the style of Vercel's design system. The session model in
//! [`session`] holds all state and is tested without any user-interface code; the views
//! in [`app`] are thin.

#![forbid(unsafe_code)]

pub mod app;
pub mod components;
pub mod fonts;
pub mod session;
pub mod theme;

use gpui::{App, Bounds, TitlebarOptions, WindowBounds, WindowOptions, prelude::*, px, size};

use crate::theme::Appearance;

/// Registers fonts, the theme and key bindings. Safe to call in tests.
pub fn init(cx: &mut App) {
    if let Err(err) = fonts::load(cx) {
        eprintln!("{err}");
    }
    let appearance = match cx.window_appearance() {
        gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark => Appearance::Dark,
        _ => Appearance::Light,
    };
    theme::install(cx, appearance);
    components::bind_text_input_keys(cx);
    app::bind_keys(cx);
}

/// Starts the application.
pub fn run() {
    gpui_platform::application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(1120.), px(760.)), cx);
        let session = cx.new(|_| session::Session::from_platform());
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Privacy Enforcer".into()),
                    ..Default::default()
                }),
                window_min_size: Some(size(px(880.), px(560.))),
                app_id: Some("io.github.master0ffate.privacy-enforcer".into()),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| app::PrivacyEnforcerApp::new(session, cx));
                view.update(cx, |view, cx| view.focus(window, cx));
                view
            },
        )
        .expect("the main window opens");
        cx.activate(true);
    });
}
