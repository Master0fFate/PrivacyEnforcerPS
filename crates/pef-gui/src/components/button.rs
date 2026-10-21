use gpui::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px,
};

use crate::theme::{RADIUS_SM, TEXT_MD, TEXT_SM, theme};

/// Visual weight of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Solid fill for the one main action on a screen.
    Primary,
    /// Outlined, for secondary actions.
    #[default]
    Secondary,
    /// Borderless, for low-emphasis actions.
    Tertiary,
    /// Red fill for destructive confirmations.
    Danger,
}

/// Height of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    /// 32 px.
    Small,
    /// 40 px.
    #[default]
    Medium,
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A clickable button.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    full_width: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// A secondary, medium button.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
        Button {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            disabled: false,
            full_width: false,
            on_click: None,
        }
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Button {
        self.variant = variant;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: ButtonSize) -> Button {
        self.size = size;
        self
    }

    /// Disables the button: muted colors, no pointer, no click.
    pub fn disabled(mut self, disabled: bool) -> Button {
        self.disabled = disabled;
        self
    }

    /// Stretches the button to its container.
    pub fn full_width(mut self) -> Button {
        self.full_width = true;
        self
    }

    /// Sets the click handler.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Button {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *theme(cx);
        let (background, foreground, border, hover_background) = match self.variant {
            ButtonVariant::Primary => (
                theme.primary,
                theme.primary_foreground,
                theme.primary,
                theme.primary_hover,
            ),
            ButtonVariant::Secondary => (theme.surface, theme.text, theme.border, theme.hover),
            ButtonVariant::Tertiary => (
                gpui::transparent_black(),
                theme.text_secondary,
                gpui::transparent_black(),
                theme.hover,
            ),
            ButtonVariant::Danger => (
                theme.error,
                theme.primary_foreground,
                theme.error,
                theme.error.opacity(0.85),
            ),
        };
        let (height, padding_x, text_size) = match self.size {
            ButtonSize::Small => (px(32.), px(10.), TEXT_SM),
            ButtonSize::Medium => (px(40.), px(14.), TEXT_MD),
        };
        let disabled = self.disabled;
        let on_click = self.on_click;
        div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .h(height)
            .px(padding_x)
            .rounded(RADIUS_SM)
            .border_1()
            .border_color(border)
            .bg(background)
            .text_color(foreground)
            .text_size(text_size)
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .when(self.full_width, |button| button.w_full())
            .when(disabled, |button| button.opacity(0.5))
            .when(!disabled, |button| {
                button
                    .cursor_pointer()
                    .hover(move |style| style.bg(hover_background))
                    .active(move |style| style.bg(theme.pressed))
                    .when_some(on_click, |button, handler| {
                        button.on_click(move |event, window, cx| handler(event, window, cx))
                    })
            })
            .child(self.label)
    }
}
