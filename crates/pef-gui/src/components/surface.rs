use gpui::{App, Div, FontWeight, ParentElement, SharedString, Styled, div, px};

use crate::theme::{RADIUS_MD, TEXT_SM, TEXT_XL, TEXT_XS, theme};

/// A bordered container on the surface color.
pub fn card(cx: &App) -> Div {
    let theme = theme(cx);
    div()
        .flex()
        .flex_col()
        .w_full()
        .rounded(RADIUS_MD)
        .border_1()
        .border_color(theme.border)
        .bg(theme.surface)
}

/// A one-pixel horizontal rule.
pub fn divider(cx: &App) -> Div {
    div().w_full().h(px(1.)).bg(theme(cx).border)
}

/// A page or section title.
pub fn section_title(cx: &App, text: impl Into<SharedString>) -> Div {
    div()
        .text_color(theme(cx).text)
        .text_size(TEXT_XL)
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

/// Small uppercase-style label text.
pub fn label(cx: &App, text: impl Into<SharedString>) -> Div {
    div()
        .text_color(theme(cx).text_tertiary)
        .text_size(TEXT_XS)
        .font_weight(FontWeight::MEDIUM)
        .child(text.into())
}

/// Secondary body text.
pub fn muted(cx: &App, text: impl Into<SharedString>) -> Div {
    div()
        .text_color(theme(cx).text_secondary)
        .text_size(TEXT_SM)
        .child(text.into())
}
