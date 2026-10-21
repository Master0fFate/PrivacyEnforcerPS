use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
};

use crate::theme::{TEXT_XS, theme};

/// Color of a badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeTone {
    /// Neutral.
    #[default]
    Gray,
    /// Informational.
    Blue,
    /// Positive.
    Green,
    /// Caution.
    Amber,
    /// Problem.
    Red,
}

/// A small pill with a short label.
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    tone: BadgeTone,
}

impl Badge {
    /// A gray badge.
    pub fn new(label: impl Into<SharedString>) -> Badge {
        Badge {
            label: label.into(),
            tone: BadgeTone::default(),
        }
    }

    /// Sets the tone.
    pub fn tone(mut self, tone: BadgeTone) -> Badge {
        self.tone = tone;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme(cx);
        let (foreground, background, border) = match self.tone {
            BadgeTone::Gray => (theme.text_secondary, theme.hover, theme.border),
            BadgeTone::Blue => (theme.accent, theme.accent_subtle, theme.accent_border),
            BadgeTone::Green => (theme.success, theme.success_subtle, theme.success_border),
            BadgeTone::Amber => (theme.warning, theme.warning_subtle, theme.warning_border),
            BadgeTone::Red => (theme.error, theme.error_subtle, theme.error_border),
        };
        div()
            .flex()
            .items_center()
            .h(px(22.))
            .px(px(8.))
            .rounded_full()
            .border_1()
            .border_color(border)
            .bg(background)
            .text_color(foreground)
            .text_size(TEXT_XS)
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .child(self.label)
    }
}
