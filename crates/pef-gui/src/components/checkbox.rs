use gpui::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, prelude::*, px,
};

use crate::theme::theme;

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// A square check control in the Vercel style: hairline border, solid fill when checked.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Checkbox {
    /// A checkbox with the given state.
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Checkbox {
        Checkbox {
            id: id.into(),
            checked,
            disabled: false,
            on_change: None,
        }
    }

    /// Disables interaction.
    pub fn disabled(mut self, disabled: bool) -> Checkbox {
        self.disabled = disabled;
        self
    }

    /// Called with the new state when clicked.
    pub fn on_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Checkbox {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *theme(cx);
        let checked = self.checked;
        let disabled = self.disabled;
        let on_change = self.on_change;
        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .size(px(16.))
            .flex_shrink_0()
            .rounded(px(4.))
            .border_1()
            .border_color(if checked {
                theme.primary
            } else {
                theme.border_strong
            })
            .bg(if checked {
                theme.primary
            } else {
                theme.surface
            })
            .text_color(theme.primary_foreground)
            .text_size(px(11.))
            .font_weight(FontWeight::BOLD)
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.border_color(theme.text_secondary))
                    .when_some(on_change, |this, handler| {
                        this.on_click(move |_: &ClickEvent, window, cx| {
                            handler(!checked, window, cx)
                        })
                    })
            })
            .when(checked, |this| this.child("✓"))
    }
}
