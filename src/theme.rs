//! The reading theme: Light (the former sepia palette), Dark, or System
//! (the default: it follows the device's own `prefers-color-scheme`).
//!
//! The choice is [`persisted`](crate::persisted) under `pc_theme`, held by
//! [`theme_signal`] — created once per render and reused after that (through
//! Leptos context), so [`crate::shell::AppShell`]'s own state and a
//! [`theme_menu`] an app builds *before* `AppShell` runs (to compose its own
//! `settings`) refer to the very same signal. "Light" and "Dark" are
//! applied by setting `data-theme` on `<html>`; "System" is the absence of
//! that attribute, so `lab/style/theme.css` follows the device live. The
//! server reads the cookie to set the attribute on the very first render
//! ([`theme_attribute`]); once hydrated, an `Effect` in
//! [`crate::shell::AppShell`] keeps it in sync with later choices. It is
//! chosen from [`theme_menu`], the default of [`crate::shell::AppShell`]'s
//! `settings`.

use std::fmt;
use std::str::FromStr;

use leptos::prelude::*;

use crate::icon::IconName;
use crate::menu::MenuItem;
use crate::persisted;

/// The three reading themes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// Follows the device's `prefers-color-scheme`. No `data-theme`
    /// attribute.
    #[default]
    System,
    /// The former sepia palette: `data-theme="light"`.
    Light,
    /// `data-theme="dark"`.
    Dark,
}

impl Theme {
    /// The `data-theme` attribute value that selects this theme; `None` for
    /// [`Theme::System`] (no attribute: CSS follows the device).
    #[must_use]
    pub fn attribute(self) -> Option<&'static str> {
        match self {
            Theme::System => None,
            Theme::Light => Some("light"),
            Theme::Dark => Some("dark"),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        })
    }
}

/// A theme name did not match `"system"`, `"light"` or `"dark"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseThemeError;

impl FromStr for Theme {
    type Err = ParseThemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(Theme::System),
            "light" => Ok(Theme::Light),
            "dark" => Ok(Theme::Dark),
            _ => Err(ParseThemeError),
        }
    }
}

/// The `data-theme` attribute `<html>` should carry for the first render of
/// a page, from the `pc_theme` cookie of the request being rendered (or
/// `document.cookie`, if this runs in the browser). `None` means `System`
/// (or no stored preference yet): the page carries no attribute, and CSS
/// follows `prefers-color-scheme` before the first paint.
#[must_use]
pub fn theme_attribute() -> Option<&'static str> {
    let key = format!("{}theme", persisted::PREFIX);
    persisted::request_cookies()
        .as_deref()
        .and_then(|cookies| persisted::cookie_value(cookies, &key))
        .and_then(|raw| raw.parse::<Theme>().ok())
        .and_then(Theme::attribute)
}

/// The reading theme: [`persisted`](crate::persisted) under `pc_theme` the
/// first time anything in this render asks for it, then reused (through
/// Leptos context) by every later call in the same owner tree — in
/// particular, an app that calls [`theme_menu`] to compose its own
/// `settings` *before* [`crate::shell::AppShell`] runs still ends up
/// sharing `AppShell`'s own [`crate::shell::ShellState::theme`], since
/// `AppShell` renders as that app's child and context flows down to it.
#[must_use]
pub fn theme_signal() -> RwSignal<Theme> {
    if let Some(signal) = use_context::<RwSignal<Theme>>() {
        return signal;
    }
    let signal = persisted::persisted("theme", Theme::default());
    provide_context(signal);
    signal
}

/// The three choices of [`theme_menu`], in the order they are shown.
const CHOICES: [(Theme, IconName, &str); 3] = [
    (Theme::System, IconName::Monitor, "System"),
    (Theme::Light, IconName::Sun, "Light"),
    (Theme::Dark, IconName::Moon, "Dark"),
];

/// The Settings entry for the reading theme: a submenu of System, Light and
/// Dark, each a [`MenuItem::Choice`] bound to [`theme_signal`]. The default
/// of [`crate::shell::AppShell`]'s `settings`; an app that overrides it
/// calls this itself, alongside its own items, to keep the theme choice.
#[must_use]
pub fn theme_menu() -> MenuItem {
    let theme = theme_signal();
    MenuItem::Submenu {
        label: "Theme",
        icon: Some(IconName::Sun),
        items: CHOICES
            .into_iter()
            .map(|(value, icon, label)| MenuItem::Choice {
                label,
                icon: Some(icon),
                selected: Signal::derive(move || theme.get() == value),
                on_select: Callback::new(move |()| theme.set(value)),
            })
            .collect(),
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn displays_and_parses_every_theme() {
        for (theme, text) in [
            (Theme::System, "system"),
            (Theme::Light, "light"),
            (Theme::Dark, "dark"),
        ] {
            assert_eq!(theme.to_string(), text);
            assert_eq!(text.parse::<Theme>(), Ok(theme));
        }
        assert!("sepia".parse::<Theme>().is_err());
    }

    #[test]
    fn system_is_the_default_and_has_no_attribute() {
        assert_eq!(Theme::default(), Theme::System);
        assert_eq!(Theme::System.attribute(), None);
        assert_eq!(Theme::Light.attribute(), Some("light"));
        assert_eq!(Theme::Dark.attribute(), Some("dark"));
    }

    fn attribute_for_cookie(value: &str) -> Option<&'static str> {
        Owner::new().with(|| {
            let request = http::Request::builder()
                .header(http::header::COOKIE, format!("pc_theme={value}"))
                .body(())
                .unwrap();
            provide_context(request.into_parts().0);
            theme_attribute()
        })
    }

    #[test]
    fn the_server_reads_the_theme_from_its_cookie() {
        assert_eq!(attribute_for_cookie("light"), Some("light"));
        assert_eq!(attribute_for_cookie("dark"), Some("dark"));
        assert_eq!(
            attribute_for_cookie("system"),
            None,
            "no attribute: CSS follows the device"
        );
    }

    #[test]
    fn a_missing_or_garbled_cookie_is_system() {
        assert_eq!(Owner::new().with(theme_attribute), None, "no cookie at all");
        assert_eq!(attribute_for_cookie("sepia"), None, "not a known theme");
    }

    #[test]
    fn the_theme_menu_is_a_submenu_of_the_three_choices_in_order() {
        Owner::new().with(|| {
            let MenuItem::Submenu { label, items, .. } = theme_menu() else {
                panic!("theme_menu is a Submenu");
            };
            assert_eq!(label, "Theme");
            assert_eq!(items.len(), 3);
            let labels: Vec<&str> = items
                .iter()
                .map(|item| match item {
                    MenuItem::Choice { label, .. } => *label,
                    _ => panic!("every item of theme_menu is a Choice"),
                })
                .collect();
            assert_eq!(labels, ["System", "Light", "Dark"]);
        });
    }

    #[test]
    fn the_theme_menu_marks_exactly_the_current_theme_selected() {
        Owner::new().with(|| {
            theme_signal().set(Theme::Dark);
            let MenuItem::Submenu { items, .. } = theme_menu() else {
                unreachable!()
            };
            let selected: Vec<bool> = items
                .iter()
                .map(|item| match item {
                    MenuItem::Choice { selected, .. } => selected.get(),
                    _ => unreachable!(),
                })
                .collect();
            assert_eq!(selected, [false, false, true]);
        });
    }

    #[test]
    fn choosing_a_theme_menu_item_sets_the_theme() {
        Owner::new().with(|| {
            let MenuItem::Submenu { items, .. } = theme_menu() else {
                unreachable!()
            };
            let MenuItem::Choice { on_select, .. } = &items[2] else {
                unreachable!("index 2 is Dark")
            };
            on_select.run(());
            assert_eq!(theme_signal().get(), Theme::Dark);
        });
    }

    #[test]
    fn theme_signal_is_the_same_instance_across_calls_in_one_owner() {
        Owner::new().with(|| {
            let first = theme_signal();
            first.set(Theme::Light);
            assert_eq!(
                theme_signal().get(),
                Theme::Light,
                "the second call reused it"
            );
        });
    }
}
