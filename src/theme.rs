//! The reading theme: Light (linen page, parchment rail, white cards, bronze accent), Dark, or System
//! (the default: it follows the device's own `prefers-color-scheme`).
//!
//! The choice is [`persisted`](fn@crate::persisted) under `pc_theme`, held by
//! [`theme_signal`] — created once per render and reused after that (through
//! Leptos context), so [`crate::shell::AppShell`]'s own state and whatever
//! an app builds with it *before* `AppShell` runs (its own `settings`
//! content, a [`theme_menu`]) refer to the very same signal. "Light" and
//! "Dark" are applied by setting `data-theme` on `<html>`; "System" is the
//! absence of that attribute, so `lab/style/theme.css` follows the device
//! live. The server reads the cookie to set the attribute on the very
//! first render ([`theme_attribute`]); once hydrated, an `Effect` in
//! [`crate::shell::AppShell`] keeps it in sync with later choices. It is
//! chosen in Settings: the default of `AppShell`'s `settings` is an
//! Appearance section holding the theme choice. [`theme_menu`] stays as a
//! helper for apps that want the theme as a menu instead.

use std::fmt;
use std::str::FromStr;

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{ChoiceCardOption, ChoiceCards};
use crate::catalogue::{Locale, strings};
use crate::icon::IconName;
use crate::language::language_signal;
use crate::menu::MenuItem;
use crate::persisted;

/// The three reading themes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// Follows the device's `prefers-color-scheme`. No `data-theme`
    /// attribute.
    #[default]
    System,
    /// The light palette (linen, parchment rail, bronze): `data-theme="light"`.
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

/// The reading theme: [`persisted`](fn@crate::persisted) under `pc_theme` the
/// first time anything in this render asks for it, then reused (through
/// Leptos context) by every later call in the same owner tree — in
/// particular, an app that reads it to build its own `settings` (or a
/// [`theme_menu`]) *before* [`crate::shell::AppShell`] runs still ends up
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

/// The three choices, in the order [`theme_menu`] shows them.
const CHOICES: [(Theme, IconName); 3] = [
    (Theme::System, IconName::Monitor),
    (Theme::Light, IconName::Sun),
    (Theme::Dark, IconName::Moon),
];

/// `theme`'s own name in `locale`.
fn theme_label(locale: Locale, theme: Theme) -> &'static str {
    match theme {
        Theme::System => td_string!(locale, system),
        Theme::Light => td_string!(locale, light),
        Theme::Dark => td_string!(locale, dark),
    }
}

/// The reading theme as a menu entry: a submenu of System, Light and Dark,
/// each a [`MenuItem::Choice`] bound to [`theme_signal`]. Not what Settings
/// shows — [`crate::shell::AppShell`]'s default `settings` is an Appearance
/// section — but kept as a helper for an app that wants a theme menu (in a
/// [`crate::menu::Menu`] of its own).
///
/// Its labels are in the language current when it is called, and stay so:
/// a [`MenuItem`]'s labels are plain strings, not signals, so a later
/// language switch does not reach them. Do not rebuild the menu to follow
/// one: each [`crate::menu::Menu`] built adds window listeners that are
/// never removed. For a theme choice that follows the language live, use
/// [`ThemePicker`].
#[must_use]
pub fn theme_menu() -> MenuItem {
    let theme = theme_signal();
    let locale = Locale::from(language_signal().get_untracked());
    MenuItem::Submenu {
        label: td_string!(locale, theme),
        icon: Some(IconName::Sun),
        items: CHOICES
            .into_iter()
            .map(|(value, icon)| MenuItem::Choice {
                label: theme_label(locale, value),
                icon: Some(icon),
                selected: Signal::derive(move || theme.get() == value),
                on_select: Callback::new(move |()| theme.set(value)),
            })
            .collect(),
    }
}

/// Settings' theme picker: a [`ChoiceCards`] of picture cards, the classic
/// Light/Dark/System thumbnails (as in macOS or Gemini) — each one a
/// miniature page (`PagePreview`: a title bar, a few text lines) drawn in
/// CSS from that theme's own tokens: Light in the light palette, Dark in
/// the dark palette, System split diagonally, half light half dark. Bound
/// to [`theme_signal`]; choosing one applies at once. The default of
/// [`crate::shell::AppShell`]'s `settings`, in place of the former
/// [`crate::button::Segmented`].
///
/// Its name and card labels follow the current language live: the whole
/// group re-renders on a switch, since a [`ChoiceCardOption`]'s label is a
/// plain string.
#[component]
pub fn ThemePicker() -> impl IntoView {
    let (value, on_change) = theme_binding();
    let t = strings();
    move || {
        let locale = t.locale();
        let options = [Theme::Light, Theme::Dark, Theme::System]
            .into_iter()
            .map(|theme| theme_card(theme, theme_label(locale, theme)))
            .collect::<Vec<_>>();
        view! {
            <ChoiceCards
                aria_label=td_string!(locale, theme)
                options=options
                value=value
                on_change=on_change
            />
        }
    }
}

/// [`ThemePicker`]'s link to [`theme_signal`]: the value its cards show, and
/// the callback that writes the chosen one back (applied at once by
/// [`crate::shell::AppShell`]'s own `Effect`).
fn theme_binding() -> (Signal<Theme>, Callback<Theme>) {
    let theme = theme_signal();
    (
        Signal::derive(move || theme.get()),
        Callback::new(move |value| theme.set(value)),
    )
}

/// One option of [`ThemePicker`]: `theme`'s own value, `label`, and its
/// [`PagePreview`] picture.
fn theme_card(theme: Theme, label: &'static str) -> ChoiceCardOption<Theme> {
    ChoiceCardOption {
        picture: theme_preview(theme),
        label,
        value: theme,
    }
}

/// The picture slot of a [`ThemePicker`] card: [`Theme::Light`] and
/// [`Theme::Dark`] are one [`PagePreview`] each, in their own palette;
/// [`Theme::System`] overlays a light and a dark one, each clipped to a
/// diagonal half, so the card reads as half light, half dark.
fn theme_preview(theme: Theme) -> ViewFn {
    match theme {
        Theme::Light => (move || view! { <PagePreview theme="light" /> }.into_any()).into(),
        Theme::Dark => (move || view! { <PagePreview theme="dark" /> }.into_any()).into(),
        Theme::System => (move || {
            view! {
                <PagePreview theme="light" clip="polygon(0 0, 100% 0, 0 100%)" />
                <PagePreview theme="dark" clip="polygon(100% 0, 100% 100%, 0 100%)" />
            }
            .into_any()
        })
        .into(),
    }
}

/// One miniature page: a title bar and a few text lines, in `theme`'s own
/// tokens (`"light"` or `"dark"`) forced on this element through
/// `data-theme`, regardless of the real page's own theme — the same
/// subtree override the gallery's Theme page uses to show Light and Dark
/// side by side (`lab/style/theme.css`'s `[data-theme="…"]` rules, which
/// match any element, not only `:root`). `clip`, given, clips it to one
/// diagonal half: [`theme_preview`]'s System card overlays a light and a
/// dark one this way. Spans throughout, displayed as blocks: it sits inside
/// a card's `<button>`, which holds phrasing content only.
#[component]
fn PagePreview(theme: &'static str, #[prop(optional)] clip: Option<&'static str>) -> impl IntoView {
    let style = clip.map(|clip| format!("clip-path: {clip};"));
    view! {
        <span data-theme=theme class="absolute inset-0 flex flex-col bg-page" style=style>
            <span class="block h-2/5 w-full border-b border-line bg-card"></span>
            <span class="flex flex-1 flex-col justify-center gap-1 px-1.5">
                <span class="block h-[2px] w-3/4 rounded-full bg-ink"></span>
                <span class="block h-[2px] w-1/2 rounded-full bg-muted"></span>
                <span class="block h-[2px] w-2/3 rounded-full bg-muted"></span>
            </span>
        </span>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::button::choice_cards_test::card;
    use crate::language::{Language, language_signal};

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
    fn the_theme_menu_reads_its_labels_in_the_language_current_when_called() {
        Owner::new().with(|| {
            language_signal().set(Language::Vi);
            let MenuItem::Submenu { label, items, .. } = theme_menu() else {
                panic!("theme_menu is a Submenu");
            };
            assert_eq!(label, "Chủ đề");
            let labels: Vec<&str> = items
                .iter()
                .map(|item| match item {
                    MenuItem::Choice { label, .. } => *label,
                    _ => unreachable!(),
                })
                .collect();
            assert_eq!(labels, ["Hệ thống", "Sáng", "Tối"]);
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

    #[test]
    fn the_theme_picker_is_a_radiogroup_of_light_dark_system_cards() {
        let html = Owner::new().with(|| {
            theme_signal().set(Theme::Dark);
            view! { <ThemePicker /> }.to_html()
        });
        assert!(html.contains(r#"role="radiogroup""#), "{html}");
        assert!(html.contains(r#"aria-label="Theme""#), "{html}");
        assert_eq!(html.matches(r#"role="radio""#).count(), 3, "{html}");
        assert!(html.contains("Light"), "{html}");
        assert!(html.contains("Dark"), "{html}");
        assert!(html.contains("System"), "{html}");
        assert_eq!(html.matches(r#"aria-checked="true""#).count(), 1, "{html}");
    }

    /// The picker carries `theme` as its name and each of `cards` as a label.
    fn assert_picker_in(html: &str, theme: &str, cards: [&str; 3]) {
        let name = format!(r#"aria-label="{theme}""#);
        assert!(html.contains(&name), "{html}");
        for label in cards {
            assert!(
                html.contains(&format!(">{label}</span>")),
                "{label}: {html}"
            );
        }
    }

    #[test]
    fn the_theme_picker_speaks_french_and_vietnamese() {
        for (language, theme, cards) in [
            (Language::Fr, "Thème", ["Clair", "Sombre", "Système"]),
            (Language::Vi, "Chủ đề", ["Sáng", "Tối", "Hệ thống"]),
        ] {
            let html = Owner::new().with(|| {
                language_signal().set(language);
                view! { <ThemePicker /> }.to_html()
            });
            assert_picker_in(&html, theme, cards);
        }
    }

    #[test]
    fn switching_the_language_re_renders_the_theme_picker() {
        let html = Owner::new().with(|| {
            let picker = view! { <ThemePicker /> };
            language_signal().set(Language::Fr);
            picker.to_html()
        });
        assert_picker_in(&html, "Thème", ["Clair", "Sombre", "Système"]);
        assert!(!html.contains(">Light</span>"), "{html}");
        assert!(!html.contains(r#"aria-label="Theme""#), "{html}");
    }

    #[test]
    fn light_and_dark_cards_force_their_own_palette_and_system_overlays_both() {
        let html = Owner::new().with(|| view! { <ThemePicker /> }.to_html());
        // One plain preview each for Light and Dark, plus System's own
        // light-clipped and dark-clipped overlay: two of each in all.
        assert_eq!(html.matches(r#"data-theme="light""#).count(), 2, "{html}");
        assert_eq!(html.matches(r#"data-theme="dark""#).count(), 2, "{html}");
        assert_eq!(html.matches("clip-path:").count(), 2, "{html}");
    }

    #[test]
    fn previews_are_phrasing_content_and_only_clipped_ones_carry_a_style() {
        let html = Owner::new().with(|| view! { <ThemePicker /> }.to_html());
        for label in ["Light", "Dark", "System"] {
            let card = card(&html, &format!(">{label}</span>"));
            assert!(
                !card.contains("<div"),
                "a button holds phrasing content: {card}"
            );
        }
        for label in ["Light", "Dark"] {
            let card = card(&html, &format!(">{label}</span>"));
            assert!(!card.contains("style="), "{label} is not clipped: {card}");
        }
        let system = card(&html, ">System</span>");
        assert_eq!(
            system.matches(r#"style="clip-path:"#).count(),
            2,
            "{system}"
        );
    }

    #[test]
    fn the_picker_checks_the_card_of_the_current_theme() {
        for (theme, label) in [
            (Theme::Light, "Light"),
            (Theme::Dark, "Dark"),
            (Theme::System, "System"),
        ] {
            let html = Owner::new().with(|| {
                theme_signal().set(theme);
                view! { <ThemePicker /> }.to_html()
            });
            for other in ["Light", "Dark", "System"] {
                let card = card(&html, &format!(">{other}</span>"));
                let opening = &card[..card.find('>').unwrap()];
                let expected = format!(r#"aria-checked="{}""#, other == label);
                assert!(
                    opening.contains(&expected),
                    "{theme}: {other} {expected}: {html}"
                );
            }
        }
    }

    #[test]
    fn choosing_a_card_writes_the_theme_signal() {
        Owner::new().with(|| {
            let (value, choose) = theme_binding();
            for theme in [Theme::Dark, Theme::Light, Theme::System] {
                choose.run(theme);
                assert_eq!(theme_signal().get(), theme, "written to theme_signal");
                assert_eq!(value.get(), theme, "and read back by the picker");
            }
        });
    }
}
