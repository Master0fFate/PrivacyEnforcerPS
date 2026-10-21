use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};

use crate::session::NoticeKind;
use crate::theme::{RADIUS_MD, TEXT_SM, theme};

/// A callout with a headline and a sentence or two, toned like the Vercel `Note`.
#[derive(IntoElement)]
pub struct Note {
    kind: NoticeKind,
    title: SharedString,
    detail: Option<SharedString>,
}

impl Note {
    /// A note of the given tone.
    pub fn new(kind: NoticeKind, title: impl Into<SharedString>) -> Note {
        Note {
            kind,
            title: title.into(),
            detail: None,
        }
    }

    /// Adds the longer explanation.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Note {
        self.detail = Some(detail.into());
        self
    }
}

impl RenderOnce for Note {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme(cx);
        let (foreground, background, border) = match self.kind {
            NoticeKind::Info => (theme.text, theme.background_secondary, theme.border),
            NoticeKind::Success => (theme.success, theme.success_subtle, theme.success_border),
            NoticeKind::Warning => (theme.warning, theme.warning_subtle, theme.warning_border),
            NoticeKind::Error => (theme.error, theme.error_subtle, theme.error_border),
        };
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .w_full()
            .px(px(14.))
            .py(px(12.))
            .rounded(RADIUS_MD)
            .border_1()
            .border_color(border)
            .bg(background)
            .child(
                div()
                    .text_color(foreground)
                    .text_size(TEXT_SM)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.title),
            )
            .when_some(self.detail, |note, detail| {
                note.child(
                    div()
                        .text_color(theme.text_secondary)
                        .text_size(TEXT_SM)
                        .child(detail),
                )
            })
    }
}
