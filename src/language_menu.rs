//! The shell's language button: the globe and the current language's own
//! name ([`Language::name`]), just above Settings.
//!
//! - **Desktop**: [`RailLanguageEntry`] in the rail opens [`LanguageMenu`],
//!   a [`Menu`] of one [`MenuItem::Choice`] per [`Language::ALL`] (English ·
//!   Français · Tiếng Việt), the current one checked. Like the Settings
//!   dialog, the menu is a sibling of the rail, fixed beside its bottom: the
//!   rail's `overflow-hidden` would clip it.
//! - **Mobile**: [`MobileLanguage`], one row (the globe and the current
//!   name) drilling down to the same three choices, laid out like a
//!   [`DrillMenu`](crate::menu::DrillMenu) but built once: a pick keeps
//!   focus on the chosen row and the list drilled.
//!
//! Choosing a language sets [`crate::language::language_signal`]: every
//! string follows, live, and the choice is kept in the `pc_lang` cookie.
//! The names never change with the page's language (each is its own
//! endonym), so neither menu is ever rebuilt: only the entry's (and the
//! row's) name and the entry's "Language" label follow.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Disclosed, DisclosureButton, Size, Variant, classes};
use crate::catalogue::{Locale, strings};
use crate::icon::{Icon, IconName};
use crate::language::Language;
use crate::menu::{BackRow, Menu, MenuItem, Path, item_row, leaf_item};

/// One [`MenuItem::Choice`] per language, in [`Language::ALL`]'s order:
/// `current`'s is selected, choosing one sets `current`.
pub(crate) fn language_choices(current: RwSignal<Language>) -> Vec<MenuItem> {
    Language::ALL
        .into_iter()
        .map(|choice| MenuItem::Choice {
            label: choice.name(),
            icon: None,
            selected: Signal::derive(move || current.get() == choice),
            on_select: Callback::new(move |()| current.set(choice)),
        })
        .collect()
}

/// The rail's language entry: the globe, and the current language's name
/// fading like the Settings label while the rail is collapsed. Named
/// "Language" in the current language: rebuilt when it changes, since a
/// [`DisclosureButton`]'s label is fixed once built.
#[component]
pub(crate) fn RailLanguageEntry(
    controls: Disclosed,
    current: RwSignal<Language>,
    #[prop(into)] shown: Signal<bool>,
) -> impl IntoView {
    let t = strings();
    move || {
        let label = td_string!(t.locale(), language);
        view! {
            <DisclosureButton
                controls=controls
                variant=Variant::Nav
                label_closed=label
                label_open=label
            >
                <Icon name=IconName::Globe />
                <span
                    class="whitespace-nowrap transition-opacity duration-150"
                    class:opacity-0=move || !shown.get()
                >
                    {move || current.get().name()}
                </span>
            </DisclosureButton>
        }
    }
}

/// The desktop language [`Menu`], fixed beside the rail's bottom, just
/// above the Settings entry; its left edge follows `rail_width` so it
/// stays flush whether the rail is open or collapsed.
#[component]
pub(crate) fn LanguageMenu(
    controls: Disclosed,
    current: RwSignal<Language>,
    #[prop(into)] rail_width: Signal<&'static str>,
) -> impl IntoView {
    // Named once, in the language it opens in: rebuilding the menu would
    // detach the very row a click just chose, and the popover would take
    // that click for an outside one.
    let aria_label = td_string!(Locale::from(current.get_untracked()), language);
    let left = move || format!("calc({} + 0.5rem)", rail_width.get());
    view! {
        <div class="fixed bottom-16 z-30 hidden sm:block" style:left=left>
            <Menu controls=controls aria_label=aria_label items=language_choices(current) />
        </div>
    }
}

/// The mobile navigation's language row, drilling down to the three
/// choices. Built once: a pick changes what is shown, never what is built
/// — rebuilding would drop the row just tapped (focus would fall to the
/// page) and reset the drill. `path` is its drill state, the shell's own
/// (`ShellState`) so its Escape can back out of it and closing the
/// navigation can reset it; a fresh one when not given.
#[component]
pub(crate) fn MobileLanguage(
    current: RwSignal<Language>,
    #[prop(optional)] path: Option<Path>,
) -> impl IntoView {
    mobile_language(current, path.unwrap_or_else(|| RwSignal::new(Vec::new())))
}

/// [`MobileLanguage`] at drill state `path` (empty: the row; `[0]`: the
/// choices, under a Back row), laid out like a
/// [`DrillMenu`](crate::menu::DrillMenu): the row
/// shows the globe and `current`'s name, live; the choices are built with
/// it and only hidden until it is tapped. Its rows (the language row, Back
/// and each choice) are `Size::Lg`, the size of the navigation's other phone
/// rows (`Size::Lg` entries, the Settings row): one left edge, one height.
pub(crate) fn mobile_language(current: RwSignal<Language>, path: Path) -> impl IntoView {
    let at_top = move || path.get().is_empty();
    let stay = Callback::new(|()| {});
    view! {
        <div role="menu" class="flex flex-col gap-0.5">
            <BackRow path=path size=Size::Lg />
            <button
                type="button"
                role="menuitem"
                aria-haspopup="true"
                class=classes(Variant::Nav, Size::Lg, "w-full")
                hidden=move || !at_top()
                on:click=move |_| path.set(vec![0])
            >
                {item_row(
                    Some(IconName::Globe),
                    move || current.get().name(),
                    view! { <Icon name=IconName::ChevronRight size=14 /> },
                    Size::Lg,
                )}
            </button>
            <div role="group" class="flex flex-col gap-0.5" hidden=at_top>
                {language_choices(current)
                    .into_iter()
                    .map(|item| leaf_item(item, stay, Size::Lg))
                    .collect_view()}
            </div>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::language::language_signal;

    fn labels(items: &[MenuItem]) -> Vec<&'static str> {
        items
            .iter()
            .map(|item| match item {
                MenuItem::Choice { label, .. } | MenuItem::Submenu { label, .. } => *label,
                _ => panic!("only choices and submenus"),
            })
            .collect()
    }

    #[test]
    fn the_choices_are_every_language_in_order() {
        Owner::new().with(|| {
            let items = language_choices(language_signal());
            assert_eq!(labels(&items), ["English", "Français", "Tiếng Việt"]);
        });
    }

    #[test]
    fn choosing_a_language_sets_the_language_signal() {
        Owner::new().with(|| {
            let language = language_signal();
            let items = language_choices(language);
            let MenuItem::Choice { on_select, .. } = &items[2] else {
                panic!("a choice");
            };
            on_select.run(());
            assert_eq!(language.get_untracked(), Language::Vi);
        });
    }

    #[test]
    fn exactly_the_current_language_is_selected() {
        Owner::new().with(|| {
            let language = language_signal();
            language.set(Language::Fr);
            let selected: Vec<bool> = language_choices(language)
                .iter()
                .map(|item| match item {
                    MenuItem::Choice { selected, .. } => selected.get_untracked(),
                    _ => panic!("a choice"),
                })
                .collect();
            assert_eq!(selected, [false, true, false]);
        });
    }

    /// The opening tag of the first element in `html` carrying `marker`.
    fn tag_with<'a>(html: &'a str, marker: &str) -> &'a str {
        let at = html
            .find(marker)
            .unwrap_or_else(|| panic!("{marker}: {html}"));
        let start = html[..at].rfind('<').unwrap();
        &html[start..at + html[at..].find('>').unwrap()]
    }

    /// The opening tag of the button whose content holds `text`.
    fn button_with<'a>(html: &'a str, text: &str) -> &'a str {
        let at = html.find(text).unwrap_or_else(|| panic!("{text}: {html}"));
        let start = html[..at].rfind("<button").unwrap();
        &html[start..start + html[start..].find('>').unwrap()]
    }

    /// Picking a language must not rebuild the mobile menu: a rebuild drops
    /// the row just tapped (focus falls to the page) and resets the drill.
    /// So the three choices are built with the row, once, and only hidden
    /// until it is tapped.
    #[test]
    fn the_mobile_choices_are_built_once_and_only_hidden() {
        let html = Owner::new().with(|| {
            let language = language_signal();
            view! { <MobileLanguage current=language /> }.to_html()
        });
        assert_eq!(html.matches(r#"role="menuitemradio""#).count(), 3, "{html}");
        assert!(
            tag_with(&html, r#"role="group""#).contains("hidden"),
            "{html}"
        );
    }

    /// Drilled in, then a pick: the list stays drilled (Back shown, the
    /// choices shown, the new one checked), and the row it came from
    /// already names the new language for when Back is tapped.
    #[test]
    fn a_pick_keeps_the_drill_and_renames_the_row() {
        let html = Owner::new().with(|| {
            let language = language_signal();
            let path = RwSignal::new(vec![0]);
            let row = mobile_language(language, path);
            language.set(Language::Fr);
            row.into_any().to_html()
        });
        assert!(
            !tag_with(&html, r#"role="group""#).contains("hidden"),
            "{html}"
        );
        assert!(
            !button_with(&html, "Retour").contains("hidden"),
            "Back: {html}"
        );
        let row = &html[html.find(r#"aria-haspopup="true""#).unwrap()..];
        assert!(
            row[..row.find("</button>").unwrap()].contains("Français"),
            "{row}"
        );
        let after = &html[html.find(r#"aria-checked="true""#).unwrap()..];
        assert!(
            after[..after.find("</button>").unwrap()].contains("Français"),
            "{html}"
        );
    }

    #[test]
    fn the_mobile_row_shows_the_globe_and_the_current_name_at_the_top() {
        let html = Owner::new().with(|| {
            let language = language_signal();
            language.set(Language::Vi);
            view! { <MobileLanguage current=language /> }.to_html()
        });
        let row = tag_with(&html, r#"aria-haspopup="true""#);
        assert!(!row.contains("hidden"), "{row}");
        let row = &html[html.find(r#"aria-haspopup="true""#).unwrap()..];
        let row = &row[..row.find("</button>").unwrap()];
        assert!(row.contains("Tiếng Việt"), "{row}");
        assert!(row.contains("M2 12h20"), "the globe: {row}");
        assert!(
            button_with(&html, "Quay lại").contains("hidden"),
            "Back: {html}"
        );
    }

    #[test]
    fn the_rail_language_entry_follows_the_language() {
        use crate::button::Disclosed;

        let html = Owner::new().with(|| {
            let controls = Disclosed::new("test-language-1", RwSignal::new(false));
            let entry = view! {
                <RailLanguageEntry
                    controls=controls
                    current=language_signal()
                    shown=Signal::derive(|| true)
                />
            };
            entry.to_html()
        });
        assert!(html.contains("English"), "should show English: {html}");

        let html = Owner::new().with(|| {
            let controls = Disclosed::new("test-language-2", RwSignal::new(false));
            let lang = language_signal();
            lang.set(Language::Fr);
            let entry = view! { <RailLanguageEntry controls=controls current=lang shown=Signal::derive(|| true) /> };
            entry.to_html()
        });
        assert!(html.contains("Français"), "should show Français: {html}");
    }

    /// The phone navigation's row, as every entry of it carries it: what
    /// `classes(Variant::Nav, Size::Lg, ..)` ends with.
    const PHONE_ROW: &str = "h-13 gap-4 px-4 text-base";

    /// The `class` attribute of the opening tag `tag`.
    fn class_of(tag: &str) -> &str {
        let at = tag.find(r#"class=""#).unwrap() + r#"class=""#.len();
        &tag[at..at + tag[at..].find('"').unwrap()]
    }

    /// The mobile language rows (070): the Back row, the language row and
    /// the three choices all carry the large row of the other phone rows,
    /// wherever the drill stands.
    #[test]
    fn the_mobile_language_rows_are_the_large_rows_of_the_phone_navigation() {
        for drilled in [false, true] {
            let html = Owner::new().with(|| {
                let path = RwSignal::new(if drilled { vec![0] } else { Vec::new() });
                language_signal().set(Language::En);
                mobile_language(language_signal(), path)
                    .into_any()
                    .to_html()
            });
            let large = classes(Variant::Nav, Size::Lg, "w-full");
            let rows = [
                button_with(&html, "Back"),
                tag_with(&html, r#"aria-haspopup="true""#),
            ];
            for row in rows {
                assert_eq!(class_of(row), large, "drilled {drilled}: {row}");
            }
            let choices: Vec<&str> = html
                .split("<button")
                .filter(|part| part.contains(r#"role="menuitemradio""#))
                .map(|part| &part[..part.find('>').unwrap()])
                .collect();
            assert_eq!(choices.len(), 3, "{html}");
            for choice in choices {
                assert_eq!(class_of(choice), large, "drilled {drilled}: {choice}");
            }
            assert!(
                !html.contains("h-11") && !html.contains("px-[13px]"),
                "no medium row left: {html}"
            );
            assert_eq!(
                html.matches(r#"<span class="flex flex-1 items-center gap-4">"#)
                    .count(),
                5,
                "Back, the language row and 3 choices: {html}"
            );
            assert!(!html.contains("items-center gap-3"), "{html}");
        }
    }

    /// The pair: the language row and a `NavList` entry of the same phone
    /// navigation share the large row's left edge, gap and height.
    #[test]
    fn the_language_row_and_a_navigation_entry_share_one_row_class() {
        use crate::icon::IconName;
        use crate::nav::{MobileSections, NavEntry, NavList, NavSection};

        let (nav, language) = Owner::new().with(|| {
            let list = NavList::new(
                vec![NavSection {
                    title: None,
                    entries: vec![NavEntry {
                        label: "Buttons",
                        icon: IconName::Button,
                        href: "/buttons",
                    }],
                }],
                "/".to_string(),
            );
            (
                view! { <MobileSections nav=list /> }.to_html(),
                view! { <MobileLanguage current=language_signal() /> }.to_html(),
            )
        });
        let entry = tag_with(&nav, r#"href="/buttons""#);
        let row = tag_with(&language, r#"aria-haspopup="true""#);
        assert!(class_of(entry).contains(PHONE_ROW), "{entry}");
        assert!(class_of(row).contains(PHONE_ROW), "{row}");
    }
}
