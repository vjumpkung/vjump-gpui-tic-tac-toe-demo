//! The "Highlighter Notebook" theme: embedded fonts, the Vjump light and dark
//! theme files, and the brand colors the view reads from the active theme.
//!
//! Every color lives in `themes/vjump.json`; nothing here names a literal.

use std::borrow::Cow;

use gpui_kit::component::{ActiveTheme, Theme, ThemeRegistry};
use gpui_kit::{App, Hsla};

const LIGHT_THEME_NAME: &str = "Vjump Light";
const DARK_THEME_NAME: &str = "Vjump Dark";

const FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/PlusJakartaSans-Regular.ttf"),
    include_bytes!("../assets/fonts/PlusJakartaSans-Medium.ttf"),
    include_bytes!("../assets/fonts/PlusJakartaSans-SemiBold.ttf"),
    include_bytes!("../assets/fonts/PlusJakartaSans-Bold.ttf"),
];

/// Registers the font, loads the Vjump themes and activates the one matching
/// the system appearance. After this, `Theme::change` reloads the brand
/// themes, so the light/dark toggle keeps the notebook look.
pub fn install(cx: &mut App) {
    cx.text_system()
        .add_fonts(FONTS.iter().map(|bytes| Cow::Borrowed(*bytes)).collect())
        .expect("failed to register Plus Jakarta Sans");

    ThemeRegistry::global_mut(cx)
        .load_themes_from_str(include_str!("../themes/vjump.json"))
        .expect("failed to load the Vjump themes");

    let registry = ThemeRegistry::global(cx);
    let light = registry
        .themes()
        .get(LIGHT_THEME_NAME)
        .cloned()
        .expect("the Vjump light theme is registered");
    let dark = registry
        .themes()
        .get(DARK_THEME_NAME)
        .cloned()
        .expect("the Vjump dark theme is registered");

    Theme::update(cx, |theme| {
        theme.light_theme = light;
        theme.dark_theme = dark;
    });
    Theme::sync_system_appearance(None, cx);
}

/// The three highlighter pens and the inks, resolved from the active theme.
#[derive(Clone, Copy)]
pub(crate) struct Brand {
    /// Note card fill: the board cells, score tiles and idle controls.
    pub(crate) card: Hsla,
    /// Graphite ink: the fixed text color on every highlighter fill.
    pub(crate) ink: Hsla,
    /// Pink ink: the one highlighter-family color that may be text.
    pub(crate) pink_ink: Hsla,
    pub(crate) highlight_yellow: Hsla,
    pub(crate) highlight_cyan: Hsla,
    pub(crate) highlight_pink: Hsla,
}

pub(crate) fn brand(cx: &App) -> Brand {
    let theme = cx.theme();
    Brand {
        card: theme.group_box,
        ink: theme.primary_foreground,
        pink_ink: theme.ring,
        highlight_yellow: theme.primary,
        highlight_cyan: theme.cyan,
        highlight_pink: theme.magenta,
    }
}
