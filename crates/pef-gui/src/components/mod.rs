//! A small component library in the Vercel style, built from GPUI primitives.

mod badge;
mod button;
mod checkbox;
mod note;
mod surface;
mod text_input;

pub use badge::{Badge, BadgeTone};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use checkbox::Checkbox;
pub use note::Note;
pub use surface::{card, divider, label, muted, section_title};
pub use text_input::{TextInput, bind_keys as bind_text_input_keys};
