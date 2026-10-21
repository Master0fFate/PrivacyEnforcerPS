//! Privacy Enforcer desktop entry point.

#![forbid(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() {
    pef_gui::run();
}
