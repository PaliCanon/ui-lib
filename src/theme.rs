//! The reading theme: Light (cream page, parchment rail and cards, deep gold accent), Dark, or System
//! (the default: it follows the device's own `prefers-color-scheme`).
//!
//! The choice is [`persisted`](fn@crate::persisted) under `pc_theme`, held by
//! [`theme_signal`] — created once per render and reused after that (through
//! Leptos context), so [`crate::shell::AppShell`]'s own state and whatever
//! an app builds with it *before* `AppShell` runs (its own `settings`
//! content, a [`theme_menu`]) refer to the very same signal. "Light" and
//! "Dark" are applied by setting `data-theme` on `<html>`; "System" is the
//! absence of that attribute, so `style/theme.css` follows the device
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
use crate::store::{Store, parse_stored};

/// The three reading themes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// Follows the device's `prefers-color-scheme`. No `data-theme`
    /// attribute.
    #[default]
    System,
    /// The light palette (cream, parchment rail and cards, deep gold): `data-theme="light"`.
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
/// a page, from the `pc_theme` value of the current [`Store`]: the request's
/// cookie on the server, `document.cookie` or `localStorage` in the browser.
/// `None` means `System` (or no stored preference yet): the page carries no
/// attribute, and CSS follows `prefers-color-scheme` before the first paint.
#[must_use]
pub fn theme_attribute() -> Option<&'static str> {
    theme_attribute_in(Store::current())
}

/// [`theme_attribute`] over a given store, for `csr::mount`, which
/// has no context yet. A garbled value maps as a missing one: no attribute.
pub(crate) fn theme_attribute_in(store: Store) -> Option<&'static str> {
    let key = format!("{}theme", persisted::PREFIX);
    parse_stored::<Theme>(store.read(&key).as_deref()).and_then(Theme::attribute)
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
/// side by side (`style/theme.css`'s `[data-theme="…"]` rules, which
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
    fn the_theme_attribute_reads_through_the_current_store() {
        Owner::new().with(|| {
            let request = http::Request::builder()
                .header(http::header::COOKIE, "pc_theme=dark")
                .body(())
                .unwrap();
            provide_context(request.into_parts().0);
            assert_eq!(theme_attribute_in(Store::Cookie), Some("dark"));
            assert_eq!(theme_attribute_in(Store::Local), None, "not the cookie");
            provide_context(Store::Local);
            assert_eq!(theme_attribute(), None, "the provided store is read");
        });
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

/// The AA guarantee of `--ps-gold-text`, computed from `style/theme.css`
/// itself: every theme block must define the token and reach 4.5:1
/// (WCAG 2.x, normal text) against each background text sits on.
#[cfg(test)]
mod gold_text_contrast {
    const CSS: &str = include_str!("../style/theme.css");
    const BACKGROUNDS: [&str; 4] = ["--ps-page", "--ps-rail", "--ps-chip", "--ps-card"];
    /// The three blocks: (name, the selector line that opens it).
    const BLOCKS: [(&str, &str); 3] = [
        ("dark :root", ":root,\n:root[data-theme=\"dark\"]"),
        ("light data-theme", ":root[data-theme=\"light\"],"),
        ("light media query", ":root:not([data-theme]) {"),
    ];

    fn block<'a>(css: &'a str, opener: &str) -> &'a str {
        let start = css
            .find(opener)
            .unwrap_or_else(|| panic!("no block {opener}"));
        let rest = &css[start..];
        &rest[..rest.find('}').expect("block end")]
    }

    fn hex(block: &str, token: &str) -> Option<[f64; 3]> {
        let at = block.find(&format!("{token}:"))?;
        let value = block[at + token.len() + 1..].split(';').next()?.trim();
        let h = value.strip_prefix('#').filter(|h| h.len() == 6)?;
        let ch = |i| u8::from_str_radix(&h[i..i + 2], 16).ok().map(f64::from);
        Some([ch(0)?, ch(2)?, ch(4)?])
    }

    fn luminance([r, g, b]: [f64; 3]) -> f64 {
        let lin = |c: f64| {
            let c = c / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
    }

    fn ratio(a: [f64; 3], b: [f64; 3]) -> f64 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    /// 079: the tone colours are text (a badge's label, a callout's icon
    /// and an error under a field), so each reaches AA on every opaque
    /// surface of every theme block. Their -soft fills are a tint of a few
    /// percent over those surfaces, which moves the ratio by less than the
    /// margin these keep.
    #[test]
    fn tone_text_reaches_aa_against_every_background_in_every_theme_block() {
        for (name, opener) in BLOCKS {
            let b = block(CSS, opener);
            // --ps-danger itself is a fill and an edge here; its text is
            // --ps-danger-text (dark --ps-danger is 4.39:1 on a card).
            for tone in [
                "--ps-success",
                "--ps-info",
                "--ps-warning",
                "--ps-danger-text",
            ] {
                let fg = hex(b, tone).unwrap_or_else(|| panic!("{name}: no {tone}"));
                for bg in BACKGROUNDS {
                    let Some(back) = hex(b, bg) else { continue };
                    let r = ratio(fg, back);
                    assert!(r >= 4.5, "{name}: {tone} on {bg} is {r:.2}:1");
                }
            }
        }
    }

    /// `rgb(r g b / a)` as its channels and alpha.
    fn rgba(block: &str, token: &str) -> Option<([f64; 3], f64)> {
        let at = block.find(&format!("{token}:"))?;
        let value = block[at + token.len() + 1..].split(';').next()?.trim();
        let inner = value.strip_prefix("rgb(")?.strip_suffix(')')?;
        let (rgb, alpha) = inner.split_once('/')?;
        let mut ch = rgb.split_whitespace().map(|c| c.parse::<f64>().ok());
        let rgb = [ch.next()??, ch.next()??, ch.next()??];
        Some((rgb, alpha.trim().parse().ok()?))
    }

    /// `soft` laid over the opaque `base`, as the browser composites it.
    fn over((fg, a): ([f64; 3], f64), base: [f64; 3]) -> [f64; 3] {
        [0, 1, 2].map(|i| fg[i] * a + base[i] * (1.0 - a))
    }

    /// 079 review: the pairs the components PAINT — a badge's tone text on
    /// its soft fill, a callout's ink on it, the accent's gold on the mark,
    /// a neutral badge's muted text on the chip — each fill composited over
    /// the card and the page, at AA (badges are 12 px text: 4.5:1).
    #[test]
    fn the_painted_tone_pairs_reach_aa_in_every_theme_block() {
        let pairs = [
            ("--ps-success", "--ps-success-soft"),
            ("--ps-info", "--ps-info-soft"),
            ("--ps-warning", "--ps-warning-soft"),
            ("--ps-danger-text", "--ps-danger-soft"),
            ("--ps-gold-text", "--ps-accent-soft"),
            ("--ps-ink", "--ps-success-soft"),
            ("--ps-ink", "--ps-info-soft"),
            ("--ps-ink", "--ps-warning-soft"),
            ("--ps-ink", "--ps-danger-soft"),
            ("--ps-ink", "--ps-mark"),
        ];
        for (name, opener) in BLOCKS {
            let b = block(CSS, opener);
            for base in ["--ps-card", "--ps-page"] {
                let under = hex(b, base).unwrap_or_else(|| panic!("{name}: no {base}"));
                for (text, fill) in pairs {
                    let fg = hex(b, text).unwrap_or_else(|| panic!("{name}: no {text}"));
                    let soft =
                        rgba(b, fill).unwrap_or_else(|| panic!("{name}: {fill} is not rgb(… / a)"));
                    let r = ratio(fg, over(soft, under));
                    assert!(r >= 4.5, "{name}: {text} on {fill} over {base} is {r:.2}:1");
                }
            }
            let (muted, chip) = (
                hex(b, "--ps-muted").expect("muted"),
                hex(b, "--ps-chip").expect("chip"),
            );
            let r = ratio(muted, chip);
            assert!(r >= 4.5, "{name}: --ps-muted on --ps-chip is {r:.2}:1");
        }
    }

    #[test]
    fn gold_text_reaches_aa_against_every_background_in_every_theme_block() {
        for (name, opener) in BLOCKS {
            let b = block(CSS, opener);
            let gold =
                hex(b, "--ps-gold-text").unwrap_or_else(|| panic!("{name}: no --ps-gold-text"));
            let mut checked = 0;
            for bg in BACKGROUNDS {
                let Some(bgc) = hex(b, bg) else { continue };
                let r = ratio(gold, bgc);
                assert!(r >= 4.5, "{name}: gold-text on {bg} is {r:.2}:1, below 4.5");
                checked += 1;
            }
            assert_eq!(
                checked,
                BACKGROUNDS.len(),
                "{name}: a background is missing"
            );
        }
    }

    /// Every `--ps-*` declaration of a block, in order: (token, value).
    fn declarations(block: &str) -> Vec<(&str, &str)> {
        block
            .split(';')
            .filter_map(|part| {
                let (name, value) = part[part.find("--ps-")?..].split_once(':')?;
                Some((name.trim(), value.trim()))
            })
            .collect()
    }

    const LIGHT_BLOCKS: [(&str, &str); 2] = [BLOCKS[1], BLOCKS[2]];
    const RESTING: [&str; 5] = [
        "--ps-page",
        "--ps-rail",
        "--ps-card",
        "--ps-chip",
        "--ps-field",
    ];
    const RESTING_TEXT: [&str; 7] = [
        "--ps-heading",
        "--ps-ink",
        "--ps-muted",
        "--ps-subtle",
        "--ps-gold-text",
        "--ps-danger",
        "--ps-success",
    ];

    /// The ratio of two tokens of a block, `None` when one is not a hex value
    /// (the dark block's `primary` is a `var()`).
    fn pair(b: &str, text: &str, surface: &str) -> Option<f64> {
        Some(ratio(hex(b, text)?, hex(b, surface)?))
    }

    fn assert_aa(name: &str, b: &str, texts: &[&str], surfaces: &[&str]) {
        for text in texts {
            for surface in surfaces {
                let r = pair(b, text, surface)
                    .unwrap_or_else(|| panic!("{name}: {text} or {surface} is not a hex value"));
                assert!(
                    r >= 4.5,
                    "{name}: {text} on {surface} is {r:.2}:1, below 4.5"
                );
            }
        }
    }

    /// The light rule opens with the root selector and the subtree selector
    /// (a `<span data-theme="light">` nested in a dark page needs the latter).
    const LIGHT_RULE_OPENER: &str = ":root[data-theme=\"light\"],\n[data-theme=\"light\"] {";

    #[test]
    fn the_light_rule_holds_both_selectors() {
        assert!(
            CSS.contains(LIGHT_RULE_OPENER),
            "theme.css lost a light selector"
        );
    }

    /// The design's light tokens (task 065, palicanon.app's cream): the one
    /// place a value change is reviewed. Names carry the `--ps-` prefix and
    /// values are written as in `theme.css`.
    const LIGHT_TOKENS: [(&str, &str); 35] = [
        ("--ps-page", "#fdf6e3"),
        ("--ps-rail", "#f5ecd7"),
        ("--ps-primary", "#f0e5cc"),
        ("--ps-primary-hover", "#e8dab9"),
        ("--ps-on-primary", "#6e4a08"),
        ("--ps-card", "#f5ecd7"),
        ("--ps-card-edge", "#e3d4b4"),
        ("--ps-chip", "#f0e5cc"),
        ("--ps-chip-hover", "#e8dab9"),
        ("--ps-line", "#ede0c8"),
        ("--ps-field", "#fffaf0"),
        ("--ps-heading", "#2c1810"),
        ("--ps-ink", "#3d2a1e"),
        ("--ps-muted", "#5c4a3d"),
        ("--ps-subtle", "#6f6152"),
        ("--ps-gold", "#84590a"),
        ("--ps-gold-hover", "#6e4a08"),
        ("--ps-gold-text", "#7e5509"),
        ("--ps-on-gold", "#fffaf0"),
        ("--ps-mark", "rgb(232 163 23 / 0.22)"),
        ("--ps-danger", "#b03a3a"),
        ("--ps-danger-soft", "rgb(176 58 58 / 0.1)"),
        ("--ps-danger-text", "#a33333"),
        ("--ps-accent-soft", "rgb(232 163 23 / 0.12)"),
        ("--ps-success", "#286b40"),
        ("--ps-success-soft", "rgb(40 107 64 / 0.1)"),
        ("--ps-info", "#2d5a82"),
        ("--ps-info-soft", "rgb(45 90 130 / 0.1)"),
        ("--ps-warning", "#8a5600"),
        ("--ps-warning-soft", "rgb(138 86 0 / 0.1)"),
        ("--ps-glass", "rgb(245 236 215 / 0.8)"),
        ("--ps-glow", "rgb(184 125 15 / 0.18)"),
        ("--ps-halo", "rgb(184 125 15 / 0.1)"),
        ("--ps-card-from", "#fffaf0"),
        ("--ps-card-to", "#f5ecd7"),
    ];

    fn squeeze(value: &str) -> String {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Both light blocks declare exactly the design's 35 names (078 added --ps-success, 079 the eight other tone tokens) (no extra,
    /// none missing, none declared twice) with exactly its values.
    #[test]
    fn each_light_block_is_the_designs_table() {
        let mut expected: Vec<&str> = LIGHT_TOKENS.iter().map(|(n, _)| *n).collect();
        expected.sort_unstable();
        for (name, opener) in LIGHT_BLOCKS {
            let decls = declarations(block(CSS, opener));
            let mut names: Vec<&str> = decls.iter().map(|(n, _)| *n).collect();
            names.sort_unstable();
            assert_eq!(names, expected, "{name}: not the design's token names");
            for (token, want) in LIGHT_TOKENS {
                let (_, got) = decls.iter().find(|(n, _)| *n == token).expect("present");
                assert_eq!(
                    squeeze(got),
                    want,
                    "{name}: {token} differs from the design"
                );
            }
        }
    }

    /// A second light rule appended later would win the cascade while the
    /// tests above read only the first: each opener appears exactly once.
    #[test]
    fn there_is_one_light_rule() {
        for opener in [LIGHT_RULE_OPENER, BLOCKS[2].1] {
            assert_eq!(
                CSS.matches(opener).count(),
                1,
                "theme.css holds {opener:?} other than once"
            );
        }
    }

    #[test]
    fn the_two_light_blocks_are_equal() {
        let data_theme = declarations(block(CSS, LIGHT_BLOCKS[0].1));
        let media = declarations(block(CSS, LIGHT_BLOCKS[1].1));
        for (token, value) in &data_theme {
            let other = media.iter().find(|(t, _)| t == token);
            assert_eq!(
                other.map(|(_, v)| v),
                Some(value),
                "{token}: the media query block differs from the data-theme block"
            );
        }
        assert_eq!(
            media.len(),
            data_theme.len(),
            "the two light blocks hold the same token set"
        );
    }

    #[test]
    fn text_reaches_aa_on_every_resting_surface() {
        for (name, opener) in LIGHT_BLOCKS {
            let b = block(CSS, opener);
            assert_aa(name, b, &RESTING_TEXT, &RESTING);
            assert_aa(
                name,
                b,
                &["--ps-on-gold"],
                &["--ps-gold", "--ps-gold-hover"],
            );
            assert_aa(name, b, &["--ps-on-primary"], &["--ps-primary"]);
        }
    }

    /// Text on the hover surfaces (`chip-hover`, `primary-hover`). `subtle` and
    /// `danger` are not asserted here: no text of those colours sits on a hover
    /// surface.
    #[test]
    fn text_on_hover_surfaces_reaches_aa() {
        let texts = [
            "--ps-heading",
            "--ps-ink",
            "--ps-muted",
            "--ps-gold-text",
            "--ps-on-primary",
        ];
        for (name, opener) in LIGHT_BLOCKS {
            let b = block(CSS, opener);
            assert_aa(name, b, &texts, &["--ps-chip-hover", "--ps-primary-hover"]);
        }
    }

    /// Not an assertion: the dark block is unchanged by the cream palette, so
    /// its matrix is printed (`--nocapture`) for whoever wants to read it.
    #[test]
    fn dark_matrix_is_printed() {
        let b = block(CSS, BLOCKS[0].1);
        for text in RESTING_TEXT {
            for surface in RESTING {
                match pair(b, text, surface) {
                    Some(r) => eprintln!("dark {text} on {surface}: {r:.2}:1 {}", flag(r)),
                    None => eprintln!("dark {text} on {surface}: not a hex value"),
                }
            }
        }
        for (text, surface) in [
            ("--ps-on-gold", "--ps-gold"),
            ("--ps-on-gold", "--ps-gold-hover"),
            ("--ps-on-primary", "--ps-primary"),
        ] {
            match pair(b, text, surface) {
                Some(r) => eprintln!("dark {text} on {surface}: {r:.2}:1 {}", flag(r)),
                None => eprintln!("dark {text} on {surface}: not a hex value (var)"),
            }
        }
    }

    fn flag(r: f64) -> &'static str {
        if r >= 4.5 { "" } else { "BELOW 4.5" }
    }

    #[test]
    fn a_missing_token_fails_the_lookup() {
        let b = block(CSS, BLOCKS[0].1);
        assert!(hex(b, "--ps-gold-text-nope").is_none());
        assert!(hex("--ps-page: #000000;", "--ps-gold-text").is_none());
    }

    #[test]
    fn the_old_gold_fails_aa_on_text() {
        let old = [158.0, 123.0, 34.0]; // #9e7b22, the former --ps-gold
        let b = block(CSS, BLOCKS[1].1);
        let page = hex(b, "--ps-page").expect("page");
        let r = ratio(old, page);
        assert!(r < 4.5, "old gold on light page is {r:.2}:1");
    }
}
