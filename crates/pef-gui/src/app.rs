//! The root view: sidebar navigation, five screens and the confirmation dialog.
//!
//! Views are deliberately thin. Every decision lives in [`Session`]; the view only reads
//! it, forwards clicks and typed text, and shows the result.

use std::path::{Path, PathBuf};

use gpui::{
    AnyElement, App, ClickEvent, Context, Div, Entity, FocusHandle, Focusable, FontWeight,
    IntoElement, KeyBinding, ParentElement, PathPromptOptions, Render, SharedString, Stateful,
    Styled, Window, div, prelude::*, px,
};
use pef_core::{AuditState, Category, ImpactLevel, PolicyValue, catalog, time};

use crate::components::{
    Badge, BadgeTone, Button, ButtonSize, ButtonVariant, Checkbox, Note, TextInput, card, divider,
    label, muted, section_title,
};
use crate::session::{NoticeKind, Session};
use crate::theme::{
    FONT_MONO, FONT_SANS, HEADER_HEIGHT, RADIUS_LG, RADIUS_MD, SIDEBAR_WIDTH, TEXT_2XL, TEXT_LG,
    TEXT_MD, TEXT_SM, TEXT_XS, Theme, theme,
};

#[allow(missing_docs)]
mod actions {
    gpui::actions!(
        privacy_enforcer,
        [
            Quit,
            ToggleTheme,
            ShowAudit,
            ShowPolicies,
            ShowEnforce,
            ShowRestore,
            ShowAbout,
            RunAudit,
            ConfirmDialog,
            CancelDialog
        ]
    );
}
pub use actions::*;

/// Key context of the root view.
pub const KEY_CONTEXT: &str = "PrivacyEnforcer";
/// Key context of the confirmation dialog.
pub const DIALOG_KEY_CONTEXT: &str = "ConfirmDialog";

/// Registers application-wide key bindings and the quit action.
pub fn bind_keys(cx: &mut App) {
    let root = Some(KEY_CONTEXT);
    let dialog = Some(DIALOG_KEY_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("ctrl-1", ShowAudit, root),
        KeyBinding::new("ctrl-2", ShowPolicies, root),
        KeyBinding::new("ctrl-3", ShowEnforce, root),
        KeyBinding::new("ctrl-4", ShowRestore, root),
        KeyBinding::new("ctrl-5", ShowAbout, root),
        KeyBinding::new("ctrl-r", RunAudit, root),
        KeyBinding::new("ctrl-shift-l", ToggleTheme, root),
        KeyBinding::new("enter", ConfirmDialog, dialog),
        KeyBinding::new("escape", CancelDialog, dialog),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
}

/// The screens reachable from the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    /// Read-only comparison of every policy.
    Audit,
    /// Catalog with selection.
    Policies,
    /// Preview and apply the selection.
    Enforce,
    /// Validate and apply a backup.
    Restore,
    /// Boundaries, shortcuts and credits.
    About,
}

impl Screen {
    /// Sidebar order.
    pub const ALL: [Screen; 5] = [
        Screen::Audit,
        Screen::Policies,
        Screen::Enforce,
        Screen::Restore,
        Screen::About,
    ];

    /// Sidebar label.
    pub const fn title(self) -> &'static str {
        match self {
            Screen::Audit => "Audit",
            Screen::Policies => "Policies",
            Screen::Enforce => "Enforce",
            Screen::Restore => "Restore",
            Screen::About => "About",
        }
    }

    /// Header subtitle.
    pub const fn subtitle(self) -> &'static str {
        match self {
            Screen::Audit => {
                "Compare every policy with its desired value. Reads only; never writes."
            }
            Screen::Policies => {
                "Choose exactly which policies to enforce. Nothing is selected for you."
            }
            Screen::Enforce => {
                "Preview the plan, choose a recovery file, then confirm by typing ENFORCE."
            }
            Screen::Restore => {
                "Validate a recovery file, review it, then confirm by typing RESTORE."
            }
            Screen::About => {
                "What this tool does, what it refuses to do, and how to use it safely."
            }
        }
    }

    const fn shortcut(self) -> &'static str {
        match self {
            Screen::Audit => "Ctrl+1",
            Screen::Policies => "Ctrl+2",
            Screen::Enforce => "Ctrl+3",
            Screen::Restore => "Ctrl+4",
            Screen::About => "Ctrl+5",
        }
    }
}

/// A write that is waiting for its typed confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    /// Apply the previewed plan.
    Enforce,
    /// Restore the loaded backup.
    Restore,
}

impl PendingAction {
    /// The word the person must type.
    pub const fn word(self) -> &'static str {
        match self {
            PendingAction::Enforce => "ENFORCE",
            PendingAction::Restore => "RESTORE",
        }
    }
}

/// The root view.
pub struct PrivacyEnforcerApp {
    session: Entity<Session>,
    screen: Screen,
    backup_path: Entity<TextInput>,
    restore_path: Entity<TextInput>,
    confirmation: Entity<TextInput>,
    dialog: Option<PendingAction>,
    dialog_error: Option<String>,
    restore_error: Option<String>,
    preview_error: Option<String>,
    focus_handle: FocusHandle,
}

impl PrivacyEnforcerApp {
    /// Creates the root view over a session.
    pub fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        let backup_path = cx.new(|cx| {
            TextInput::new(cx, "C:\\PrivateBackups\\privacy-before.json").monospace(true)
        });
        let restore_path = cx.new(|cx| {
            TextInput::new(cx, "C:\\PrivateBackups\\privacy-before.json").monospace(true)
        });
        let confirmation = cx.new(|cx| TextInput::new(cx, "Type the word exactly"));
        let session_handle = session.clone();
        cx.observe(&session_handle, |_, _, cx| cx.notify()).detach();
        for input in [&backup_path, &restore_path, &confirmation] {
            cx.observe(input, |_, _, cx| cx.notify()).detach();
        }
        Self {
            session,
            screen: Screen::Audit,
            backup_path,
            restore_path,
            confirmation,
            dialog: None,
            dialog_error: None,
            restore_error: None,
            preview_error: None,
            focus_handle: cx.focus_handle(),
        }
    }

    /// The session entity.
    pub fn session(&self) -> &Entity<Session> {
        &self.session
    }

    /// The visible screen.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// The backup-path input of the enforce screen.
    pub fn backup_path_input(&self) -> &Entity<TextInput> {
        &self.backup_path
    }

    /// The path input of the restore screen.
    pub fn restore_path_input(&self) -> &Entity<TextInput> {
        &self.restore_path
    }

    /// The typed-confirmation input of the dialog.
    pub fn confirmation_input(&self) -> &Entity<TextInput> {
        &self.confirmation
    }

    /// The open confirmation dialog, if any.
    pub fn dialog(&self) -> Option<PendingAction> {
        self.dialog
    }

    /// The message shown inside the dialog after a rejected confirmation.
    pub fn dialog_error(&self) -> Option<&str> {
        self.dialog_error.as_deref()
    }

    /// Focuses the root so keyboard shortcuts work.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
    }

    /// Switches screens.
    pub fn show(&mut self, screen: Screen, cx: &mut Context<Self>) {
        self.screen = screen;
        cx.notify();
    }

    /// Runs the read-only audit.
    pub fn run_audit(&mut self, cx: &mut Context<Self>) {
        self.session.update(cx, |session, cx| {
            session.run_audit();
            cx.notify();
        });
        cx.notify();
    }

    /// Builds the enforcement preview from the selection and shows the enforce screen.
    pub fn preview(&mut self, cx: &mut Context<Self>) {
        self.preview_error = self.session.update(cx, |session, cx| {
            let result = session.preview_enforce().map(|_| ()).err();
            cx.notify();
            result
        });
        self.screen = Screen::Enforce;
        cx.notify();
    }

    /// Loads and validates the backup named in the restore input.
    pub fn load_backup(&mut self, cx: &mut Context<Self>) {
        let path = PathBuf::from(self.restore_path.read(cx).text());
        self.restore_error = if path.as_os_str().is_empty() {
            Some("Enter or choose the path of a backup file first.".into())
        } else {
            self.session.update(cx, |session, cx| {
                let result = session.load_backup(&path).map(|_| ()).err();
                cx.notify();
                result
            })
        };
        cx.notify();
    }

    /// Opens the confirmation dialog for a write.
    pub fn open_dialog(
        &mut self,
        action: PendingAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dialog = Some(action);
        self.dialog_error = None;
        self.confirmation
            .update(cx, |input, cx| input.clear(window, cx));
        self.confirmation
            .read(cx)
            .focus_handle(cx)
            .focus(window, cx);
        cx.notify();
    }

    /// Closes the dialog without writing.
    pub fn cancel_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dialog = None;
        self.dialog_error = None;
        self.confirmation
            .update(cx, |input, cx| input.clear(window, cx));
        self.focus(window, cx);
        cx.notify();
    }

    /// Checks the typed word and performs the pending write.
    pub fn confirm_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(action) = self.dialog else {
            return;
        };
        let typed = self.confirmation.read(cx).text().to_owned();
        if typed != action.word() {
            self.dialog_error = Some(format!(
                "Type {} exactly to confirm. Nothing was written.",
                action.word()
            ));
            cx.notify();
            return;
        }
        let backup_path = PathBuf::from(self.backup_path.read(cx).text());
        let result = self.session.update(cx, |session, cx| {
            let result = match action {
                PendingAction::Enforce => session.apply_enforce(&backup_path, &typed).map(|_| ()),
                PendingAction::Restore => session.apply_restore(&typed).map(|_| ()),
            };
            cx.notify();
            result
        });
        match result {
            Err(message) if self.session.read(cx).notice().is_none() => {
                // A blocker rather than an operation result: keep the dialog open.
                self.dialog_error = Some(message);
            }
            _ => {
                self.dialog = None;
                self.dialog_error = None;
                self.confirmation
                    .update(cx, |input, cx| input.clear(window, cx));
                self.screen = Screen::Audit;
                self.focus(window, cx);
            }
        }
        cx.notify();
    }

    fn toggle_theme(cx: &mut App) {
        cx.update_global::<Theme, _>(|theme, _| {
            *theme = Theme::for_appearance(theme.appearance.toggled());
        });
        cx.refresh_windows();
    }

    fn choose_backup_path(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let suggested = format!(
            "privacy-before-{}.json",
            time::utc_now_rfc3339()
                .replace([':', '-'], "")
                .replace('T', "-")
                .trim_end_matches('Z')
        );
        let receiver = cx.prompt_for_new_path(&default_directory(), Some(&suggested));
        let input = self.backup_path.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Ok(Ok(Some(path))) = receiver.await {
                input
                    .update_in(cx, |input, window, cx| {
                        input.set_text(path.display().to_string(), window, cx);
                    })
                    .ok();
            }
        })
        .detach();
    }

    fn choose_restore_path(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open backup".into()),
        });
        let input = self.restore_path.clone();
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await
                && let Some(path) = paths.into_iter().next()
            {
                input
                    .update_in(cx, |input, window, cx| {
                        input.set_text(path.display().to_string(), window, cx);
                    })
                    .ok();
                this.update(cx, |this, cx| this.load_backup(cx)).ok();
            }
        })
        .detach();
    }

    fn export_audit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let report = serde_json::json!({
            "System": session.status().description,
            "Machine": session.status().machine,
            "Simulated": session.status().simulated,
            "Summary": session.summary(),
            "Findings": session.findings(),
        });
        let text = serde_json::to_string_pretty(&report).unwrap_or_default();
        let receiver = cx.prompt_for_new_path(&default_directory(), Some("privacy-audit.json"));
        let session = self.session.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Ok(Ok(Some(path))) = receiver.await {
                let outcome = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .and_then(|mut file| std::io::Write::write_all(&mut file, text.as_bytes()));
                session.update(cx, |session, cx| {
                    match outcome {
                        Ok(()) => session.set_notice(
                            NoticeKind::Success,
                            "Audit exported",
                            format!(
                                "Saved {}. The report contains the computer name.",
                                path.display()
                            ),
                        ),
                        Err(err) => session.set_notice(
                            NoticeKind::Error,
                            "Export failed",
                            format!("{}: {err}", path.display()),
                        ),
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    // ---------------------------------------------------------------- rendering

    fn render_sidebar(&self, cx: &Context<Self>) -> Div {
        let theme = *theme(cx);
        let session = self.session.read(cx);
        let status = session.status().clone();
        let current = self.screen;
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(SIDEBAR_WIDTH)
            .h_full()
            .bg(theme.background_secondary)
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .px(px(20.))
                    .h(HEADER_HEIGHT)
                    .justify_center()
                    .child(
                        div()
                            .text_size(TEXT_MD)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child("Privacy Enforcer"),
                    )
                    .child(label(cx, format!("v{}", pef_core::VERSION))),
            )
            .child(divider(cx))
            .child(div().flex().flex_col().gap(px(2.)).p(px(12.)).children(
                Screen::ALL.iter().enumerate().map(|(index, screen)| {
                    let screen = *screen;
                    let selected = screen == current;
                    div()
                        .id(("nav", index))
                        .flex()
                        .items_center()
                        .justify_between()
                        .h(px(36.))
                        .px(px(10.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .text_size(TEXT_MD)
                        .text_color(if selected {
                            theme.text
                        } else {
                            theme.text_secondary
                        })
                        .when(selected, |item| {
                            item.bg(theme.pressed).font_weight(FontWeight::MEDIUM)
                        })
                        .hover(move |style| style.bg(theme.hover))
                        .on_click(
                            cx.listener(move |this, _: &ClickEvent, _, cx| this.show(screen, cx)),
                        )
                        .child(screen.title())
                        .child(
                            div()
                                .text_size(TEXT_XS)
                                .text_color(theme.text_tertiary)
                                .child(screen.shortcut()),
                        )
                }),
            ))
            .child(div().flex_1())
            .child(divider(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .p(px(16.))
                    .child(label(cx, "This computer"))
                    .child(
                        div()
                            .text_size(TEXT_SM)
                            .text_color(theme.text)
                            .font_family(FONT_MONO)
                            .child(status.machine.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(6.))
                            .child(if status.simulated {
                                Badge::new("Simulation").tone(BadgeTone::Amber)
                            } else if status.elevated {
                                Badge::new("Administrator").tone(BadgeTone::Green)
                            } else {
                                Badge::new("Not elevated").tone(BadgeTone::Gray)
                            }),
                    )
                    .child(
                        Button::new(
                            "toggle-theme",
                            match theme.appearance {
                                crate::theme::Appearance::Light => "Dark mode",
                                crate::theme::Appearance::Dark => "Light mode",
                            },
                        )
                        .size(ButtonSize::Small)
                        .full_width()
                        .on_click(|_, _, cx| Self::toggle_theme(cx)),
                    ),
            )
    }

    fn render_header(&self, cx: &Context<Self>) -> Div {
        let theme = *theme(cx);
        let description = self.session.read(cx).status().description.clone();
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(16.))
            .flex_shrink_0()
            .px(px(32.))
            .py(px(20.))
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .min_w_0()
                    .child(
                        div()
                            .text_size(TEXT_2XL)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.screen.title()),
                    )
                    .child(muted(cx, self.screen.subtitle())),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .max_w(px(360.))
                    .text_size(TEXT_XS)
                    .text_color(theme.text_tertiary)
                    .text_right()
                    .child(description),
            )
    }

    fn render_banners(&self, cx: &Context<Self>) -> Div {
        let session = self.session.read(cx);
        let status = session.status();
        let mut banners = div().flex().flex_col().gap(px(12.));
        if status.simulated {
            banners = banners.child(
                Note::new(NoticeKind::Warning, "Simulation: no Windows registry on this system")
                    .detail(format!(
                        "{} Values live in memory for this window only and nothing on the computer is changed.",
                        status.description
                    )),
            );
        } else if !status.elevated {
            banners = banners.child(
                Note::new(NoticeKind::Info, "Not elevated: audit and preview only")
                    .detail("Run Privacy Enforcer as administrator to apply or restore. Auditing never needs elevation."),
            );
        }
        if let Some(notice) = session.notice() {
            banners = banners.child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(8.))
                    .child(div().flex_1().child(
                        Note::new(notice.kind, notice.title.clone()).detail(notice.detail.clone()),
                    ))
                    .child(
                        Button::new("dismiss-notice", "Dismiss")
                            .variant(ButtonVariant::Tertiary)
                            .size(ButtonSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session.update(cx, |session, cx| {
                                    session.clear_notice();
                                    cx.notify();
                                });
                            })),
                    ),
            );
        }
        banners
    }

    fn render_audit(&self, cx: &Context<Self>) -> Div {
        let theme = *theme(cx);
        let session = self.session.read(cx);
        let summary = session.summary();
        let audited = session.audited();
        let tile = |title: &'static str, value: usize, tone: BadgeTone| {
            card(cx)
                .flex_1()
                .gap(px(6.))
                .p(px(16.))
                .child(label(cx, title))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(
                            div()
                                .text_size(TEXT_2XL)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text)
                                .child(if audited {
                                    value.to_string()
                                } else {
                                    "–".to_owned()
                                }),
                        )
                        .when(audited && value > 0, |row| {
                            row.child(
                                Badge::new(match tone {
                                    BadgeTone::Green => "stored value matches",
                                    BadgeTone::Red => "could not be read",
                                    _ => "absent or different",
                                })
                                .tone(tone),
                            )
                        }),
                )
        };
        div()
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        Button::new("run-audit", "Run audit")
                            .variant(ButtonVariant::Primary)
                            .on_click(cx.listener(|this, _, _, cx| this.run_audit(cx))),
                    )
                    .child(
                        Button::new("export-audit", "Export JSON…")
                            .disabled(!audited)
                            .on_click(cx.listener(|this, _, window, cx| this.export_audit(window, cx))),
                    )
                    .child(muted(cx, format!(
                        "Reads {} registry values through the 64-bit view. Ctrl+R repeats the audit.",
                        catalog::all().len()
                    ))),
            )
            .child(
                div()
                    .flex()
                    .gap(px(12.))
                    .child(tile("Configured", summary.configured, BadgeTone::Green))
                    .child(tile("Different or not configured", summary.different, BadgeTone::Gray))
                    .child(tile("Unknown", summary.unknown, BadgeTone::Red)),
            )
            .when(!audited, |page| {
                page.child(
                    card(cx).items_center().gap(px(6.)).p(px(32.)).child(
                        div()
                            .text_size(TEXT_LG)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child("No audit yet"),
                    ).child(muted(cx, "Run an audit to see which stored policy values already match. Nothing is written.")),
                )
            })
            .when(audited, |page| {
                page.children(Category::ALL.iter().map(|category| {
                    let category = *category;
                    let rows: Vec<_> = catalog::in_category(category).collect();
                    card(cx)
                        .child(category_header(cx, category, rows.len(), None))
                        .children(rows.into_iter().enumerate().map(|(index, policy)| {
                            let finding = session.finding(policy.id);
                            let (badge, detail): (Badge, String) = match finding {
                                Some(finding) => {
                                    let current = match finding.current {
                                        Some(value) => value.to_string(),
                                        None => "unknown".to_owned(),
                                    };
                                    let detail = format!("current={current}, desired={}", finding.desired);
                                    let badge = match finding.state {
                                        AuditState::Configured => Badge::new("Configured").tone(BadgeTone::Green),
                                        AuditState::DifferentOrNotConfigured => Badge::new("Not configured").tone(BadgeTone::Gray),
                                        AuditState::Unknown => Badge::new("Unknown").tone(BadgeTone::Red),
                                    };
                                    (badge, detail)
                                }
                                None => (Badge::new("Not audited"), String::new()),
                            };
                            let error = finding.and_then(|f| f.error.clone());
                            policy_row(cx, index, policy, None)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_end()
                                        .gap(px(4.))
                                        .flex_shrink_0()
                                        .child(badge)
                                        .child(div().text_size(TEXT_XS).text_color(theme.text_tertiary).font_family(FONT_MONO).child(detail))
                                        .when_some(error, |column, error| {
                                            column.child(div().text_size(TEXT_XS).text_color(theme.error).max_w(px(320.)).child(error))
                                        }),
                                )
                        }))
                }))
            })
    }

    fn render_policies(&self, cx: &Context<Self>) -> Div {
        let theme = *theme(cx);
        let session = self.session.read(cx);
        let selected_count = session.selected_count();
        let session_entity = self.session.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                card(cx)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(12.))
                            .px(px(16.))
                            .py(px(12.))
                            .border_b_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(div().text_size(TEXT_MD).font_weight(FontWeight::MEDIUM).text_color(theme.text).child("Presets"))
                                    .child(muted(cx, "A preset only adds to the selection; the preview still shows every policy by name.")),
                            )
                            .child({
                                let session_entity = session_entity.clone();
                                Button::new("clear-selection", "Clear selection")
                                    .variant(ButtonVariant::Tertiary)
                                    .size(ButtonSize::Small)
                                    .disabled(selected_count == 0)
                                    .on_click(move |_, _, cx| {
                                        session_entity.update(cx, |session, cx| {
                                            session.clear_selection();
                                            cx.notify();
                                        });
                                    })
                            }),
                    )
                    .children(catalog::presets().iter().enumerate().map(|(index, preset)| {
                        let preset_id = preset.id;
                        let session_entity = session_entity.clone();
                        let count = catalog::expand_preset(preset).len();
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .px(px(16.))
                            .py(px(10.))
                            .when(index > 0, |row| row.border_t_1().border_color(theme.border))
                            .child(
                                div().w(px(150.)).flex_shrink_0().child(
                                    Button::new(("preset", index), format!("{} ({count})", preset.title))
                                        .size(ButtonSize::Small)
                                        .full_width()
                                        .on_click(move |_, _, cx| {
                                            session_entity.update(cx, |session, cx| {
                                                let _ = session.select_preset(preset_id);
                                                cx.notify();
                                            });
                                        }),
                                ),
                            )
                            .child(div().flex_1().min_w_0().child(muted(cx, preset.description)))
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(12.))
                    .child(muted(cx, format!("{selected_count} of {} selected", catalog::all().len())))
                    .child(
                        Button::new("preview-selection", "Preview changes")
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Small)
                            .disabled(selected_count == 0)
                            .on_click(cx.listener(|this, _, _, cx| this.preview(cx))),
                    ),
            )
            .children(Category::ALL.iter().map(|category| {
                let category = *category;
                let rows: Vec<_> = catalog::in_category(category).collect();
                let all_selected = session.category_fully_selected(category);
                let session_for_category = session_entity.clone();
                card(cx)
                    .child(category_header(
                        cx,
                        category,
                        rows.len(),
                        Some(
                            Checkbox::new(("category", category as usize), all_selected).on_change(
                                move |checked, _, cx| {
                                    session_for_category.update(cx, |session, cx| {
                                        session.set_category_selected(category, checked);
                                        cx.notify();
                                    });
                                },
                            ),
                        ),
                    ))
                    .children(rows.into_iter().enumerate().map(|(index, policy)| {
                        let selected = session.is_selected(policy.id);
                        let session_for_row = session_entity.clone();
                        let id = policy.id;
                        let audit_badge =
                            session
                                .finding(policy.id)
                                .map(|finding| match finding.state {
                                    AuditState::Configured => {
                                        Badge::new("Configured").tone(BadgeTone::Green)
                                    }
                                    AuditState::DifferentOrNotConfigured => {
                                        Badge::new("Not configured")
                                    }
                                    AuditState::Unknown => {
                                        Badge::new("Unknown").tone(BadgeTone::Red)
                                    }
                                });
                        let source = policy.source;
                        let checkbox = Checkbox::new(("policy", index), selected)
                            .on_change(move |checked, _, cx| {
                                session_for_row.update(cx, |session, cx| {
                                    session.set_selected(id, checked);
                                    cx.notify();
                                });
                            })
                            .into_any_element();
                        policy_row(cx, index, policy, Some(checkbox))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .flex_shrink_0()
                                    .when_some(audit_badge, |row, badge| row.child(badge))
                                    .child(
                                        Button::new(("source", index), "Reference")
                                            .variant(ButtonVariant::Tertiary)
                                            .size(ButtonSize::Small)
                                            .on_click(move |_, _, cx| cx.open_url(source)),
                                    ),
                            )
                    }))
            }))
    }

    fn render_enforce(&self, cx: &Context<Self>) -> Div {
        let theme = *theme(cx);
        let session = self.session.read(cx);
        let selection = session.selection();
        let plan = session.plan().cloned();
        let blocker = session.apply_blocker();
        let titles: Vec<&str> = selection.iter().map(|p| p.title).collect();
        let step = |number: &'static str, title: &'static str| {
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(24.))
                        .rounded_full()
                        .bg(theme.primary)
                        .text_color(theme.primary_foreground)
                        .text_size(TEXT_XS)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(number),
                )
                .child(
                    div()
                        .text_size(TEXT_LG)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text)
                        .child(title),
                )
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                card(cx)
                    .gap(px(12.))
                    .p(px(20.))
                    .child(step("1", "Selection"))
                    .child(if selection.is_empty() {
                        muted(cx, "Nothing selected. Choose policies on the Policies screen.")
                    } else {
                        muted(cx, format!("{} policies: {}", selection.len(), titles.join(", ")))
                    })
                    .child(
                        div().flex().child(
                            Button::new("edit-selection", "Edit selection")
                                .size(ButtonSize::Small)
                                .on_click(cx.listener(|this, _, _, cx| this.show(Screen::Policies, cx))),
                        ),
                    ),
            )
            .child(
                card(cx)
                    .gap(px(12.))
                    .p(px(20.))
                    .child(step("2", "Preview"))
                    .child(muted(cx, "Reads every selected value and lists exactly what would change. Nothing is written."))
                    .child(
                        div().flex().child(
                            Button::new("preview-plan", "Preview changes")
                                .size(ButtonSize::Small)
                                .disabled(selection.is_empty())
                                .on_click(cx.listener(|this, _, _, cx| this.preview(cx))),
                        ),
                    )
                    .when_some(self.preview_error.clone(), |step, error| {
                        step.child(Note::new(NoticeKind::Error, error))
                    })
                    .when_some(plan.clone(), |step, plan| {
                        if plan.changes.is_empty() {
                            step.child(Note::new(NoticeKind::Info, "Nothing to change").detail("Every selected policy already holds its desired value."))
                        } else {
                            step.child(divider(cx)).children(plan.changes.iter().enumerate().map(|(index, change)| {
                                let policy = catalog::get(&change.id).expect("plans only hold catalog policies");
                                change_row(cx, index, policy.title, &change.id, change.before, change.desired, policy.impact_level)
                            }))
                        }
                    }),
            )
            .child(
                card(cx)
                    .gap(px(12.))
                    .p(px(20.))
                    .child(step("3", "Recovery file"))
                    .child(muted(cx, "A new file in an existing private folder. It records the previous value of every change and is written and verified before the first registry write."))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(div().flex_1().child(self.backup_path.clone()))
                            .child(
                                Button::new("choose-backup", "Choose…")
                                    .on_click(cx.listener(|this, _, window, cx| this.choose_backup_path(window, cx))),
                            ),
                    ),
            )
            .child(
                card(cx)
                    .gap(px(12.))
                    .p(px(20.))
                    .child(step("4", "Enforce"))
                    .child(muted(cx, "Machine-wide policies. On organization-managed devices, consult your administrator first. Restart affected apps; some policies need sign-out or restart."))
                    .when_some(blocker.clone(), |step, blocker| step.child(Note::new(NoticeKind::Warning, blocker)))
                    .child(
                        div().flex().child(
                            Button::new(
                                "enforce",
                                match &plan {
                                    Some(plan) if !plan.changes.is_empty() => {
                                        SharedString::from(format!("Enforce {} change(s)…", plan.changes.len()))
                                    }
                                    _ => SharedString::from("Enforce…"),
                                },
                            )
                            .variant(ButtonVariant::Primary)
                            .disabled(blocker.is_some())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_dialog(PendingAction::Enforce, window, cx)
                            })),
                        ),
                    ),
            )
    }

    fn render_restore(&self, cx: &Context<Self>) -> Div {
        let theme = *theme(cx);
        let session = self.session.read(cx);
        let loaded = session.loaded_backup().cloned();
        let blocker = session.restore_blocker();
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                card(cx)
                    .gap(px(12.))
                    .p(px(20.))
                    .child(div().text_size(TEXT_LG).font_weight(FontWeight::MEDIUM).text_color(theme.text).child("Recovery file"))
                    .child(muted(cx, "Only restore your own original file. It is validated against this computer and the catalog; registry paths never come from the file."))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(div().flex_1().child(self.restore_path.clone()))
                            .child(
                                Button::new("choose-restore", "Open…")
                                    .on_click(cx.listener(|this, _, window, cx| this.choose_restore_path(window, cx))),
                            )
                            .child(
                                Button::new("validate-restore", "Validate")
                                    .variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(|this, _, _, cx| this.load_backup(cx))),
                            ),
                    )
                    .when_some(self.restore_error.clone(), |step, error| step.child(Note::new(NoticeKind::Error, error))),
            )
            .when_some(loaded, |page, loaded| {
                page.child(
                    card(cx)
                        .gap(px(12.))
                        .p(px(20.))
                        .child(div().text_size(TEXT_LG).font_weight(FontWeight::MEDIUM).text_color(theme.text).child("Validated"))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(6.))
                                .child(Badge::new(format!("{} change(s)", loaded.backup.changes.len())).tone(BadgeTone::Blue))
                                .child(Badge::new(format!("created {}", loaded.backup.created)))
                                .child(Badge::new(format!("machine {}", loaded.backup.machine))),
                        )
                        .child(muted(cx, format!("{}", loaded.path.display())))
                        .when(loaded.backup.changes.is_empty(), |card| {
                            card.child(Note::new(NoticeKind::Info, "Nothing to restore").detail("The backup records no changes."))
                        })
                        .when(!loaded.backup.changes.is_empty(), |card| {
                            card.child(divider(cx)).children(loaded.backup.changes.iter().rev().enumerate().map(|(index, change)| {
                                let policy = catalog::get(&change.id).expect("validated backups only hold catalog policies");
                                restore_row(cx, index, policy.title, &change.id, change.before, change.desired)
                            }))
                        })
                        .when_some(blocker.clone(), |card, blocker| card.child(Note::new(NoticeKind::Warning, blocker)))
                        .child(
                            div().flex().child(
                                Button::new("restore", format!("Restore {} value(s)…", loaded.backup.changes.len()))
                                    .variant(ButtonVariant::Primary)
                                    .disabled(blocker.is_some())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_dialog(PendingAction::Restore, window, cx)
                                    })),
                            ),
                        ),
                )
            })
    }

    fn render_about(&self, cx: &Context<Self>) -> Div {
        let theme = *theme(cx);
        let bullet = |text: &'static str| {
            div()
                .flex()
                .gap(px(8.))
                .child(div().text_color(theme.text_tertiary).child("•"))
                .child(
                    div()
                        .flex_1()
                        .text_size(TEXT_SM)
                        .text_color(theme.text_secondary)
                        .child(text),
                )
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                card(cx)
                    .gap(px(10.))
                    .p(px(20.))
                    .child(section_title(cx, "Audit-first, reversible, explicit"))
                    .child(muted(cx, "Privacy Enforcer reads and writes documented Windows policy values below HKEY_LOCAL_MACHINE. Every write is planned, backed up, read back and reversible. Nothing is selected on your behalf."))
                    .child(bullet("Audit and preview never write and never need elevation."))
                    .child(bullet("The complete plan is saved and verified before the first registry write."))
                    .child(bullet("A value changed by another program stops the operation instead of being overwritten."))
                    .child(bullet("Restore is idempotent and tolerates a partially applied plan."))
                    .child(bullet("Configured confirms a stored value, not effective enforcement by every app; MDM or Group Policy can override it.")),
            )
            .child(
                card(cx)
                    .gap(px(10.))
                    .p(px(20.))
                    .child(section_title(cx, "What it does not do"))
                    .child(bullet("It does not disable Microsoft Defender, Windows Update, User Account Control, event logs, scheduled tasks, services, authentication or network connectivity."))
                    .child(bullet("It does not guarantee anonymity, eliminate all telemetry, clear past cloud data or isolate apps with firewall rules."))
                    .child(bullet("It does not auto-update, install persistence, download blocklists, phone home or run shell commands.")),
            )
            .child(
                card(cx)
                    .gap(px(10.))
                    .p(px(20.))
                    .child(section_title(cx, "Keyboard"))
                    .child(bullet("Ctrl+1 to Ctrl+5 switch screens. Ctrl+R runs the audit. Ctrl+Shift+L toggles dark mode. Ctrl+Q quits."))
                    .child(bullet("Enter confirms and Escape cancels the confirmation dialog once the word is typed.")),
            )
            .child(
                card(cx)
                    .gap(px(10.))
                    .p(px(20.))
                    .child(section_title(cx, "Credits"))
                    .child(muted(cx, format!("Privacy Enforcer {} by Master0fFate, MIT licensed. Built with GPUI Fast and the Geist typeface (SIL Open Font License 1.1).", pef_core::VERSION)))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(
                                Button::new("open-repo", "Source code")
                                    .size(ButtonSize::Small)
                                    .on_click(|_, _, cx| cx.open_url("https://github.com/Master0fFate/PrivacyEnforcerPS")),
                            )
                            .child(
                                Button::new("open-policy-reference", "Microsoft policy reference")
                                    .size(ButtonSize::Small)
                                    .on_click(|_, _, cx| {
                                        cx.open_url("https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy")
                                    }),
                            ),
                    ),
            )
    }

    fn render_dialog(&self, action: PendingAction, cx: &Context<Self>) -> Stateful<Div> {
        let theme = *theme(cx);
        let session = self.session.read(cx);
        let (title, summary) = match action {
            PendingAction::Enforce => (
                "Confirm enforcement",
                format!(
                    "{} value(s) will be written after the recovery file is saved and verified.",
                    session.plan().map(|p| p.changes.len()).unwrap_or(0)
                ),
            ),
            PendingAction::Restore => (
                "Confirm restore",
                format!(
                    "{} value(s) will be written back to what the recovery file recorded.",
                    session
                        .loaded_backup()
                        .map(|l| l.backup.changes.len())
                        .unwrap_or(0)
                ),
            ),
        };
        div()
            .id("dialog-backdrop")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.backdrop)
            .occlude()
            .child(
                div()
                    .key_context(DIALOG_KEY_CONTEXT)
                    .on_action(cx.listener(|this, _: &ConfirmDialog, window, cx| this.confirm_dialog(window, cx)))
                    .on_action(cx.listener(|this, _: &CancelDialog, window, cx| this.cancel_dialog(window, cx)))
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .w(px(480.))
                    .p(px(24.))
                    .rounded(RADIUS_LG)
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.surface)
                    .shadow_lg()
                    .child(div().text_size(TEXT_LG).font_weight(FontWeight::SEMIBOLD).text_color(theme.text).child(title))
                    .child(muted(cx, summary))
                    .child(muted(cx, "These are machine-wide policies. On organization-managed devices, consult your administrator first. No guarantee of anonymity or zero telemetry."))
                    .child(label(cx, format!("Type {} to confirm only these policies", action.word())))
                    .child(self.confirmation.clone())
                    .when_some(self.dialog_error.clone(), |dialog, error| {
                        dialog.child(div().text_size(TEXT_SM).text_color(theme.error).child(error))
                    })
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(
                                Button::new("dialog-cancel", "Cancel")
                                    .on_click(cx.listener(|this, _, window, cx| this.cancel_dialog(window, cx))),
                            )
                            .child(
                                Button::new("dialog-confirm", action.word())
                                    .variant(ButtonVariant::Danger)
                                    .on_click(cx.listener(|this, _, window, cx| this.confirm_dialog(window, cx))),
                            ),
                    ),
            )
    }
}

fn default_directory() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .filter(|path| Path::new(path).is_dir())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn category_header(cx: &App, category: Category, count: usize, control: Option<Checkbox>) -> Div {
    let theme = *theme(cx);
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .px(px(16.))
        .py(px(12.))
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.background_secondary)
        .rounded_t(RADIUS_MD)
        .when_some(control, |header, control| header.child(control))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(TEXT_MD)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text)
                        .child(category.label()),
                )
                .child(muted(cx, category.description())),
        )
        .child(Badge::new(format!("{count}")))
}

fn impact_badge(level: ImpactLevel) -> Badge {
    match level {
        ImpactLevel::Moderate => Badge::new("Moderate impact"),
        ImpactLevel::High => Badge::new("High impact").tone(BadgeTone::Amber),
    }
}

fn policy_row(
    cx: &App,
    index: usize,
    policy: &'static pef_core::PrivacyPolicy,
    leading: Option<AnyElement>,
) -> Div {
    let theme = *theme(cx);
    div()
        .flex()
        .items_start()
        .gap(px(12.))
        .px(px(16.))
        .py(px(12.))
        .when(index > 0, |row| row.border_t_1().border_color(theme.border))
        .when_some(leading, |row, leading| {
            row.child(div().pt(px(2.)).child(leading))
        })
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(TEXT_MD)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text)
                                .child(policy.title),
                        )
                        .child(
                            div()
                                .text_size(TEXT_XS)
                                .text_color(theme.text_tertiary)
                                .font_family(FONT_MONO)
                                .child(policy.id),
                        )
                        .child(impact_badge(policy.impact_level)),
                )
                .child(muted(cx, policy.impact))
                .child(
                    div()
                        .text_size(TEXT_XS)
                        .text_color(theme.text_tertiary)
                        .font_family(FONT_MONO)
                        .child(format!("{} = {}", policy.registry_path(), policy.desired)),
                ),
        )
}

fn change_row(
    cx: &App,
    index: usize,
    title: &'static str,
    id: &str,
    before: PolicyValue,
    desired: u32,
    level: ImpactLevel,
) -> Div {
    let theme = *theme(cx);
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .py(px(8.))
        .when(index > 0, |row| row.border_t_1().border_color(theme.border))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(TEXT_MD)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(TEXT_XS)
                        .text_color(theme.text_tertiary)
                        .font_family(FONT_MONO)
                        .child(id.to_owned()),
                ),
        )
        .child(impact_badge(level))
        .child(
            div()
                .text_size(TEXT_SM)
                .font_family(FONT_MONO)
                .text_color(theme.text_secondary)
                .child(format!("{before} → {desired}")),
        )
}

fn restore_row(
    cx: &App,
    index: usize,
    title: &'static str,
    id: &str,
    before: PolicyValue,
    desired: u32,
) -> Div {
    let theme = *theme(cx);
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .py(px(8.))
        .when(index > 0, |row| row.border_t_1().border_color(theme.border))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(TEXT_MD)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(TEXT_XS)
                        .text_color(theme.text_tertiary)
                        .font_family(FONT_MONO)
                        .child(id.to_owned()),
                ),
        )
        .child(
            div()
                .text_size(TEXT_SM)
                .font_family(FONT_MONO)
                .text_color(theme.text_secondary)
                .child(format!("{desired} → {before}")),
        )
}

impl Focusable for PrivacyEnforcerApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PrivacyEnforcerApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *theme(cx);
        let content = match self.screen {
            Screen::Audit => self.render_audit(cx),
            Screen::Policies => self.render_policies(cx),
            Screen::Enforce => self.render_enforce(cx),
            Screen::Restore => self.render_restore(cx),
            Screen::About => self.render_about(cx),
        };
        div()
            .id("root")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &ShowAudit, _, cx| this.show(Screen::Audit, cx)))
            .on_action(cx.listener(|this, _: &ShowPolicies, _, cx| this.show(Screen::Policies, cx)))
            .on_action(cx.listener(|this, _: &ShowEnforce, _, cx| this.show(Screen::Enforce, cx)))
            .on_action(cx.listener(|this, _: &ShowRestore, _, cx| this.show(Screen::Restore, cx)))
            .on_action(cx.listener(|this, _: &ShowAbout, _, cx| this.show(Screen::About, cx)))
            .on_action(cx.listener(|this, _: &RunAudit, _, cx| this.run_audit(cx)))
            .on_action(|_: &ToggleTheme, _, cx| Self::toggle_theme(cx))
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .bg(theme.background)
            .text_color(theme.text)
            .text_size(TEXT_MD)
            .font_family(FONT_SANS)
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.render_header(cx))
                    .child(
                        div()
                            // One scroll position per screen, so switching never lands mid-page.
                            .id(("content", self.screen as usize))
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(20.))
                                    .px(px(32.))
                                    .py(px(24.))
                                    .max_w(px(1040.))
                                    .child(self.render_banners(cx))
                                    .child(content),
                            ),
                    ),
            )
            .when_some(self.dialog, |root, action| {
                root.child(self.render_dialog(action, cx))
            })
    }
}
