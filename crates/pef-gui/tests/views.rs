//! Headless tests of the root view: navigation, the enforce and restore flows and the
//! confirmation dialog, driven exactly as the buttons and keys would.

use gpui::{AppContext, Focusable, TestAppContext};
use pef_core::{PolicyValue, catalog};
use pef_gui::app::{
    PendingAction, PrivacyEnforcerApp, RunAudit, Screen, ShowPolicies, ShowRestore,
};
use pef_gui::session::{NoticeKind, Session};

fn value(session: &Session, id: &str) -> PolicyValue {
    session.store().read(catalog::get(id).unwrap()).unwrap()
}

#[gpui::test]
fn screens_switch_via_actions_and_methods(cx: &mut TestAppContext) {
    cx.update(pef_gui::init);
    let (view, cx) = cx.add_window_view(|_, cx| {
        let session = cx.new(|_| Session::simulated("test", "TEST-PC"));
        PrivacyEnforcerApp::new(session, cx)
    });
    assert_eq!(view.read_with(cx, |view, _| view.screen()), Screen::Audit);
    let focus = view.read_with(cx, |view, cx| view.focus_handle(cx));
    view.update_in(cx, |view, window, cx| view.focus(window, cx));
    cx.update(|window, cx| focus.dispatch_action(&ShowPolicies, window, cx));
    assert_eq!(
        view.read_with(cx, |view, _| view.screen()),
        Screen::Policies
    );
    cx.update(|window, cx| focus.dispatch_action(&ShowRestore, window, cx));
    assert_eq!(view.read_with(cx, |view, _| view.screen()), Screen::Restore);
    for screen in Screen::ALL {
        view.update(cx, |view, cx| view.show(screen, cx));
        assert_eq!(view.read_with(cx, |view, _| view.screen()), screen);
    }
    assert!(!view.read_with(cx, |view, cx| view.session().read(cx).audited()));
    cx.update(|window, cx| focus.dispatch_action(&RunAudit, window, cx));
    assert!(view.read_with(cx, |view, cx| view.session().read(cx).audited()));
}

#[gpui::test]
fn enforce_and_restore_flows_through_the_dialog(cx: &mut TestAppContext) {
    cx.update(pef_gui::init);
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("privacy-before.json");
    let (view, cx) = cx.add_window_view(|_, cx| {
        let session = cx.new(|_| Session::simulated("test", "TEST-PC"));
        PrivacyEnforcerApp::new(session, cx)
    });
    let session = view.read_with(cx, |view, _| view.session().clone());

    // Nothing selected: the preview reports it and lands on the enforce screen.
    view.update(cx, |view, cx| view.preview(cx));
    assert_eq!(view.read_with(cx, |view, _| view.screen()), Screen::Enforce);
    assert!(session.read_with(cx, |session, _| session.plan().is_none()));

    session.update(cx, |session, cx| {
        session.toggle("app-camera");
        session.toggle("clipboard-sync");
        cx.notify();
    });
    view.update(cx, |view, cx| view.preview(cx));
    assert_eq!(
        session.read_with(cx, |session, _| session.plan().unwrap().changes.len()),
        2
    );

    // The dialog refuses a wrong word and keeps the plan.
    view.update_in(cx, |view, window, cx| {
        view.open_dialog(PendingAction::Enforce, window, cx)
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.dialog()),
        Some(PendingAction::Enforce)
    );
    view.update_in(cx, |view, window, cx| {
        let input = view.confirmation_input().clone();
        input.update(cx, |input, cx| input.set_text("enforce", window, cx));
        view.confirm_dialog(window, cx);
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.dialog()),
        Some(PendingAction::Enforce)
    );
    assert!(view.read_with(cx, |view, _| {
        view.dialog_error()
            .unwrap()
            .starts_with("Type ENFORCE exactly")
    }));
    assert_eq!(
        session.read_with(cx, |session, _| value(session, "app-camera")),
        PolicyValue::present(0)
    );

    // A missing backup path is reported inside the dialog, nothing is written.
    view.update_in(cx, |view, window, cx| {
        let input = view.confirmation_input().clone();
        input.update(cx, |input, cx| input.set_text("ENFORCE", window, cx));
        view.confirm_dialog(window, cx);
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.dialog()),
        Some(PendingAction::Enforce)
    );
    assert!(view.read_with(cx, |view, _| {
        view.dialog_error().unwrap().contains("backup file")
    }));

    // With a path and the exact word the plan is applied, verified and audited.
    view.update_in(cx, |view, window, cx| {
        let path_input = view.backup_path_input().clone();
        path_input.update(cx, |input, cx| {
            input.set_text(backup.display().to_string(), window, cx)
        });
        let input = view.confirmation_input().clone();
        input.update(cx, |input, cx| input.set_text("ENFORCE", window, cx));
        view.confirm_dialog(window, cx);
    });
    assert_eq!(view.read_with(cx, |view, _| view.dialog()), None);
    assert_eq!(view.read_with(cx, |view, _| view.screen()), Screen::Audit);
    assert!(backup.exists());
    session.read_with(cx, |session, _| {
        assert_eq!(value(session, "app-camera"), PolicyValue::present(2));
        assert_eq!(value(session, "clipboard-sync"), PolicyValue::present(0));
        assert_eq!(session.notice().unwrap().kind, NoticeKind::Success);
        assert!(session.audited());
    });
    assert!(view.read_with(cx, |view, cx| {
        view.confirmation_input().read(cx).text().is_empty()
    }));

    // Restore: validate the file through the input, confirm with RESTORE.
    view.update(cx, |view, cx| view.load_backup(cx));
    assert!(
        session.read_with(cx, |session, _| session.loaded_backup().is_none()),
        "empty path"
    );
    view.update_in(cx, |view, window, cx| {
        let input = view.restore_path_input().clone();
        input.update(cx, |input, cx| {
            input.set_text(backup.display().to_string(), window, cx)
        });
        view.load_backup(cx);
    });
    assert_eq!(
        session.read_with(cx, |session, _| session
            .loaded_backup()
            .unwrap()
            .backup
            .changes
            .len()),
        2
    );
    view.update_in(cx, |view, window, cx| {
        view.open_dialog(PendingAction::Restore, window, cx)
    });
    view.update_in(cx, |view, window, cx| {
        let input = view.confirmation_input().clone();
        input.update(cx, |input, cx| input.set_text("RESTORE", window, cx));
        view.confirm_dialog(window, cx);
    });
    assert_eq!(view.read_with(cx, |view, _| view.dialog()), None);
    session.read_with(cx, |session, _| {
        assert_eq!(value(session, "app-camera"), PolicyValue::present(0));
        assert_eq!(value(session, "clipboard-sync"), PolicyValue::present(1));
        assert!(
            session
                .notice()
                .unwrap()
                .title
                .starts_with("2 value(s) restored")
        );
    });
}

#[gpui::test]
fn cancelling_the_dialog_writes_nothing(cx: &mut TestAppContext) {
    cx.update(pef_gui::init);
    let (view, cx) = cx.add_window_view(|_, cx| {
        let session = cx.new(|_| Session::simulated("test", "TEST-PC"));
        PrivacyEnforcerApp::new(session, cx)
    });
    let session = view.read_with(cx, |view, _| view.session().clone());
    session.update(cx, |session, cx| {
        session.select_preset("strict").unwrap();
        cx.notify();
    });
    view.update(cx, |view, cx| view.preview(cx));
    view.update_in(cx, |view, window, cx| {
        view.open_dialog(PendingAction::Enforce, window, cx);
        let input = view.confirmation_input().clone();
        input.update(cx, |input, cx| input.set_text("ENFORCE", window, cx));
        view.cancel_dialog(window, cx);
    });
    assert_eq!(view.read_with(cx, |view, _| view.dialog()), None);
    assert!(view.read_with(cx, |view, cx| {
        view.confirmation_input().read(cx).text().is_empty()
    }));
    session.read_with(cx, |session, _| {
        assert_eq!(value(session, "app-camera"), PolicyValue::present(0));
        assert!(session.plan().is_some(), "cancel keeps the preview");
        assert!(session.notice().is_none());
    });
}

#[gpui::test]
fn every_screen_renders_in_every_state(cx: &mut TestAppContext) {
    cx.update(pef_gui::init);
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("b.json");
    let (view, cx) = cx.add_window_view(|_, cx| {
        let session = cx.new(|_| Session::simulated("test", "TEST-PC"));
        PrivacyEnforcerApp::new(session, cx)
    });
    let session = view.read_with(cx, |view, _| view.session().clone());
    // Empty state, then audited, selected, previewed, applied, loaded and the dialog.
    for screen in Screen::ALL {
        view.update(cx, |view, cx| view.show(screen, cx));
    }
    view.update(cx, |view, cx| view.run_audit(cx));
    session.update(cx, |session, cx| {
        session.select_preset("baseline").unwrap();
        cx.notify();
    });
    for screen in Screen::ALL {
        view.update(cx, |view, cx| view.show(screen, cx));
    }
    view.update(cx, |view, cx| view.preview(cx));
    view.update_in(cx, |view, window, cx| {
        view.backup_path_input().clone().update(cx, |input, cx| {
            input.set_text(backup.display().to_string(), window, cx)
        });
        view.open_dialog(PendingAction::Enforce, window, cx);
        view.confirmation_input()
            .clone()
            .update(cx, |input, cx| input.set_text("ENFORCE", window, cx));
        view.confirm_dialog(window, cx);
    });
    view.update_in(cx, |view, window, cx| {
        view.restore_path_input().clone().update(cx, |input, cx| {
            input.set_text(backup.display().to_string(), window, cx)
        });
        view.load_backup(cx);
        view.show(Screen::Restore, cx);
        view.open_dialog(PendingAction::Restore, window, cx);
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.dialog()),
        Some(PendingAction::Restore)
    );
    view.update_in(cx, |view, window, cx| view.cancel_dialog(window, cx));
    for screen in Screen::ALL {
        view.update(cx, |view, cx| view.show(screen, cx));
    }
    cx.run_until_parked();
}
