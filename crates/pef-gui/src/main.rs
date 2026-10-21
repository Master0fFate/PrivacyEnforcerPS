//! Privacy Enforcer desktop entry point.

#![forbid(unsafe_code)]
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    pef_gui::run();
}
