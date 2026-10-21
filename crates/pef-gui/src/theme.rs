//! Design tokens in the style of Vercel's Geist design system: monochrome surfaces,
//! one-pixel hairline borders, a single blue accent, small radii and a 14 px type scale.
//! Both a light and a dark palette are provided and switch at runtime.

use gpui::{App, Global, Hsla, Pixels, Rgba, px, rgb};

/// Light or dark palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Appearance {
    /// White surfaces, near-black text.
    Light,
    /// Near-black surfaces, light text.
    Dark,
}

impl Appearance {
    /// The opposite palette.
    pub const fn toggled(self) -> Appearance {
        match self {
            Appearance::Light => Appearance::Dark,
            Appearance::Dark => Appearance::Light,
        }
    }
}

/// The resolved color palette plus the fixed metrics of the design system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// Which palette this is.
    pub appearance: Appearance,
    /// Window background (`background-100`).
    pub background: Hsla,
    /// Secondary background for sidebars and wells (`background-200` / `gray-100`).
    pub background_secondary: Hsla,
    /// Card and input surface.
    pub surface: Hsla,
    /// Hairline border (`gray-400`).
    pub border: Hsla,
    /// Stronger border for hover and emphasis (`gray-500`).
    pub border_strong: Hsla,
    /// Primary text (`gray-1000`).
    pub text: Hsla,
    /// Secondary text (`gray-900`).
    pub text_secondary: Hsla,
    /// Tertiary text and placeholders (`gray-700`).
    pub text_tertiary: Hsla,
    /// Hover background for rows and tertiary buttons (`gray-200`).
    pub hover: Hsla,
    /// Selected or pressed background (`gray-300`).
    pub pressed: Hsla,
    /// Primary action fill: black in light mode, white in dark mode.
    pub primary: Hsla,
    /// Primary action hover fill.
    pub primary_hover: Hsla,
    /// Text on a primary fill.
    pub primary_foreground: Hsla,
    /// Accent blue (`blue-700`).
    pub accent: Hsla,
    /// Accent hover (`blue-800`).
    pub accent_hover: Hsla,
    /// Accent-tinted background (`blue-100`).
    pub accent_subtle: Hsla,
    /// Accent-tinted border (`blue-400`).
    pub accent_border: Hsla,
    /// Success text (`green-900`).
    pub success: Hsla,
    /// Success background (`green-100`).
    pub success_subtle: Hsla,
    /// Success border (`green-400`).
    pub success_border: Hsla,
    /// Warning text (`amber-900`).
    pub warning: Hsla,
    /// Warning background (`amber-100`).
    pub warning_subtle: Hsla,
    /// Warning border (`amber-400`).
    pub warning_border: Hsla,
    /// Error text (`red-900`).
    pub error: Hsla,
    /// Error background (`red-100`).
    pub error_subtle: Hsla,
    /// Error border (`red-400`).
    pub error_border: Hsla,
    /// Focus ring color.
    pub focus_ring: Hsla,
    /// Modal backdrop.
    pub backdrop: Hsla,
}

/// Radius of controls such as buttons and inputs.
pub const RADIUS_SM: Pixels = px(6.);
/// Radius of cards and notes.
pub const RADIUS_MD: Pixels = px(8.);
/// Radius of dialogs.
pub const RADIUS_LG: Pixels = px(12.);
/// Caption size.
pub const TEXT_XS: Pixels = px(12.);
/// Secondary text size.
pub const TEXT_SM: Pixels = px(13.);
/// Body size.
pub const TEXT_MD: Pixels = px(14.);
/// Emphasised body size.
pub const TEXT_LG: Pixels = px(16.);
/// Section titles.
pub const TEXT_XL: Pixels = px(20.);
/// Page titles.
pub const TEXT_2XL: Pixels = px(24.);
/// Family name of the embedded sans-serif typeface.
pub const FONT_SANS: &str = "Geist";
/// Family name of the embedded monospace typeface.
pub const FONT_MONO: &str = "Geist Mono";
/// Sidebar width.
pub const SIDEBAR_WIDTH: Pixels = px(232.);
/// Header height.
pub const HEADER_HEIGHT: Pixels = px(56.);

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

impl Theme {
    /// The light palette.
    pub fn light() -> Theme {
        Theme {
            appearance: Appearance::Light,
            background: color(0xffffff),
            background_secondary: color(0xfafafa),
            surface: color(0xffffff),
            border: color(0xeaeaea),
            border_strong: color(0xc9c9c9),
            text: color(0x171717),
            text_secondary: color(0x666666),
            text_tertiary: color(0x7d7d7d),
            hover: color(0xf2f2f2),
            pressed: color(0xe6e6e6),
            primary: color(0x171717),
            primary_hover: color(0x383838),
            primary_foreground: color(0xffffff),
            accent: color(0x0068d6),
            accent_hover: color(0x0054ad),
            accent_subtle: color(0xf0f7ff),
            accent_border: color(0xb7d9ff),
            success: color(0x297a3a),
            success_subtle: color(0xeefaf0),
            success_border: color(0xb4e0c2),
            warning: color(0xa35200),
            warning_subtle: color(0xfff6e5),
            warning_border: color(0xffd38a),
            error: color(0xcb2a2f),
            error_subtle: color(0xfff0f0),
            error_border: color(0xffc7c7),
            focus_ring: color(0x0068d6),
            backdrop: Hsla {
                h: 0.,
                s: 0.,
                l: 0.,
                a: 0.4,
            },
        }
    }

    /// The dark palette.
    pub fn dark() -> Theme {
        Theme {
            appearance: Appearance::Dark,
            background: color(0x0a0a0a),
            background_secondary: color(0x000000),
            surface: color(0x0a0a0a),
            border: color(0x2e2e2e),
            border_strong: color(0x454545),
            text: color(0xededed),
            text_secondary: color(0xa1a1a1),
            text_tertiary: color(0x8f8f8f),
            hover: color(0x1a1a1a),
            pressed: color(0x292929),
            primary: color(0xededed),
            primary_hover: color(0xcccccc),
            primary_foreground: color(0x0a0a0a),
            accent: color(0x3291ff),
            accent_hover: color(0x52a8ff),
            accent_subtle: color(0x0f1b2d),
            accent_border: color(0x10325a),
            success: color(0x62c073),
            success_subtle: color(0x0b1f12),
            success_border: color(0x1f4d2c),
            warning: color(0xff990a),
            warning_subtle: color(0x27190a),
            warning_border: color(0x5a3a0a),
            error: color(0xff6166),
            error_subtle: color(0x2a0c0c),
            error_border: color(0x5c1f1f),
            focus_ring: color(0x3291ff),
            backdrop: Hsla {
                h: 0.,
                s: 0.,
                l: 0.,
                a: 0.6,
            },
        }
    }

    /// The palette for an appearance.
    pub fn for_appearance(appearance: Appearance) -> Theme {
        match appearance {
            Appearance::Light => Theme::light(),
            Appearance::Dark => Theme::dark(),
        }
    }

    /// Every (name, color) pair, for tests and documentation.
    pub fn colors(&self) -> Vec<(&'static str, Hsla)> {
        vec![
            ("background", self.background),
            ("background_secondary", self.background_secondary),
            ("surface", self.surface),
            ("border", self.border),
            ("border_strong", self.border_strong),
            ("text", self.text),
            ("text_secondary", self.text_secondary),
            ("text_tertiary", self.text_tertiary),
            ("hover", self.hover),
            ("pressed", self.pressed),
            ("primary", self.primary),
            ("primary_hover", self.primary_hover),
            ("primary_foreground", self.primary_foreground),
            ("accent", self.accent),
            ("accent_hover", self.accent_hover),
            ("accent_subtle", self.accent_subtle),
            ("accent_border", self.accent_border),
            ("success", self.success),
            ("success_subtle", self.success_subtle),
            ("success_border", self.success_border),
            ("warning", self.warning),
            ("warning_subtle", self.warning_subtle),
            ("warning_border", self.warning_border),
            ("error", self.error),
            ("error_subtle", self.error_subtle),
            ("error_border", self.error_border),
            ("focus_ring", self.focus_ring),
        ]
    }
}

impl Global for Theme {}

/// The active theme. Panics if [`install`] was not called.
pub fn theme(cx: &App) -> &Theme {
    cx.global::<Theme>()
}

/// Installs a theme as the global.
pub fn install(cx: &mut App, appearance: Appearance) {
    cx.set_global(Theme::for_appearance(appearance));
}

/// WCAG relative luminance of a color.
pub fn relative_luminance(color: Hsla) -> f32 {
    let Rgba { r, g, b, .. } = color.to_rgb();
    let channel = |c: f32| {
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// WCAG contrast ratio between two colors (1.0 to 21.0).
pub fn contrast_ratio(a: Hsla, b: Hsla) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (light, dark) = if la > lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}
