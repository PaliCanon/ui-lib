//! The app shell every PaliCanon service shares.
//!
//! - **Desktop** (≥ 640 px): a navigation rail on the left. Collapsed, it
//!   shows the lotus only; the lotus opens it (the name appears) and closes
//!   it. The page sits beside it; the rail spans the page's full height and its
//!   content sticks, capped at the viewport height. An optional footer is the
//!   page column's last block, beside the rail, at the bottom of a short page.
//!   The shell is one column at least a screen tall: the top bar (phone),
//!   then the rail-and-page row — a short page is one screen on both.
//!   Open or closed is remembered
//!   ([`persisted`](fn@crate::persisted), cookie `pc_rail`), across pages and subdomains.
//! - **Mobile**: a top bar with the lotus; the lotus opens the navigation
//!   full screen, the lotus or ✕ closes it. Always closed when a page loads,
//!   and always reopened at its top (never drilled into the languages).
//! - **Escape** closes whichever is open, innermost first: with the
//!   language menu or the Settings dialog open, only that one; drilled
//!   into the mobile languages, it backs out one level.
//!
//! Below the lotus, the app's [`NavList`] (none by default): each service
//! adds its own entry when it is created. Choosing an entry in the mobile
//! navigation closes it.
//!
//! **Language** sits just above Settings, at the bottom of the rail and of
//! the full-screen navigation: the globe and the current language's own
//! name. On desktop it opens a menu of the languages (English · Français ·
//! Tiếng Việt), its own disclosure `pc-language`; on mobile its row drills
//! down to the same choices. Choosing one sets
//! [`crate::language::language_signal`]: the shell's own
//! strings (Settings, Appearance, Open and Close navigation) follow live,
//! and so does `<html lang>` once hydrated.
//!
//! **Settings** sits at the bottom of the rail (desktop) and of the
//! full-screen navigation (mobile): both the gear (desktop) and the row
//! (mobile) open the very same [`crate::dialog::Dialog`], titled
//! "Settings", through one shared disclosure
//! (`ShellState::settings_open`) — only one of the two triggers is ever
//! visible at a time (the rail and the mobile navigation already are), so
//! sharing an id never confuses an outside click the way two overlapping
//! popovers once could. Its content is `settings`: one
//! [`crate::dialog::DialogSection`] "Appearance" holding the theme as a
//! [`crate::theme::ThemePicker`] (picture cards: Light · Dark · System) by
//! default; an app overriding `settings` replaces that content (it adds the
//! Appearance section back itself to keep the theme choice). Escape and the
//! backdrop click, already wired into `Dialog`, close it like anything else
//! it closes.
//!
//! **`env_badge`** ([`crate::env_badge::EnvBadgeInfo`], none by default)
//! shows which build is running: below Settings at the rail bottom
//! (desktop), and on the right of the top bar (mobile) — the very same
//! value, rendered twice.
//!
//! A [`DisclosureButton`]'s label and a [`Dialog`]'s title are fixed once
//! built, so each piece carrying one of the shell's strings is rebuilt in
//! its own reactive closure when the language changes.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Button, Disclosed, Disclosure, DisclosureButton, Size, Variant};
use crate::catalogue::strings;
use crate::dialog::{Dialog, DialogSection};
use crate::env_badge::{EnvBadge, EnvBadgeInfo};
use crate::icon::{Icon, IconName};
use crate::language::{Language, provide_language};
use crate::language_menu::{LanguageMenu, MobileLanguage, RailLanguageEntry};
use crate::menu::Path;
use crate::nav::{MobileSections, NavList, RailSections};
use crate::persisted;
use crate::theme::{self, Theme, ThemePicker};

/// Rail widths, collapsed and open.
const RAIL_CLOSED: &str = "4rem";
const RAIL_OPEN: &str = "13.5rem";

/// Shared state of the shell, provided as context: any component below the
/// shell reads it and follows its changes.
#[derive(Clone, Copy, Debug)]
pub struct ShellState {
    /// The desktop rail is open (remembered in the `pc_rail` cookie).
    pub rail_open: RwSignal<bool>,
    /// The mobile full-screen navigation is open (never remembered).
    pub menu_open: RwSignal<bool>,
    /// The Settings dialog is open (never remembered): shared by the
    /// desktop gear and the mobile row, see the module documentation.
    pub settings_open: RwSignal<bool>,
    /// The desktop language menu is open (never remembered).
    pub language_open: RwSignal<bool>,
    /// The reading theme (remembered in the `pc_theme` cookie).
    pub theme: RwSignal<Theme>,
    /// The interface language: the page's one signal
    /// ([`provide_language`], called again here in case the app's root did
    /// not; remembered in the `pc_lang` cookie).
    pub language: RwSignal<Language>,
    /// The mobile language row's drill state (empty: the row; `[0]`: the
    /// languages), held here so Escape can back out of it and closing the
    /// navigation through the shell (lotus, ✕, an entry, Escape) can reset
    /// it.
    pub(crate) language_path: Path,
}

/// The lotus, the content of the buttons that open and close the navigation.
#[component]
fn Lotus(logo: &'static str) -> impl IntoView {
    view! { <img src=logo alt="" width="28" height="28" class="size-7" /> }
}

/// The name "PaliCanon", beside the lotus; `shown` fades it in and out (the
/// desktop rail shows it only when open).
#[component]
fn Wordmark(#[prop(into)] shown: Signal<bool>) -> impl IntoView {
    view! {
        <span
            class="font-serif text-lg font-semibold whitespace-nowrap text-heading transition-opacity duration-150"
            class:opacity-0=move || !shown.get()
            aria-hidden="true"
        >
            "PaliCanon"
        </span>
    }
}

/// The shell around a service's page. `logo` is the lotus image's URL
/// (the service serves it; `/lotus.png` by default). One column at least a
/// screen tall holds the top bar (phone), then the rail-and-page row: the
/// rail spans the page's full height; beside it the page column ends with
/// the optional `footer`, at the bottom of a short page — one screen on a
/// phone and on a desktop alike.
#[component]
pub fn AppShell(
    /// URL of the lotus image, served by the service.
    #[prop(default = "/lotus.png")]
    logo: &'static str,
    /// The app's navigation, below the lotus; none by default.
    #[prop(optional)]
    nav: Option<NavList>,
    /// The Settings dialog's content: one Appearance section with the
    /// theme as a [`ThemePicker`] when not given; an app overriding it that
    /// still wants the theme choice adds that section back itself.
    #[prop(optional, into)]
    settings: Option<ViewFn>,
    /// Which build is running, shown at the rail bottom (desktop) and in
    /// the top bar (mobile); none by default.
    #[prop(optional)]
    env_badge: Option<EnvBadgeInfo>,
    /// A footer at the bottom of the page column, beside the rail; none by
    /// default.
    #[prop(optional, into)]
    footer: Option<ViewFn>,
    /// The service's page.
    children: Children,
) -> impl IntoView {
    let state = shell_state();
    provide_context(state);
    close_on_escape(state);
    apply_theme(state.theme);
    apply_language(state.language);
    let settings = settings.unwrap_or_else(default_settings);
    view! {
        <SettingsDialog settings=settings />
        // One column at least a screen tall: the top bar (phone), then the
        // row — the rail first, the full height of the page; beside it the
        // page column, its footer at the bottom. A short page is one screen,
        // top bar included.
        <div class="flex min-h-screen flex-col">
            <MobileNav logo=logo nav=nav.clone() env_badge=env_badge />
            <div class="flex flex-1 flex-col sm:flex-row">
                <DesktopRail logo=logo nav=nav env_badge=env_badge />
                <div class="flex min-w-0 flex-1 flex-col">
                    {children()}
                    {footer.map(|footer| view! { <div class="mt-auto">{footer.run()}</div> })}
                </div>
            </div>
        </div>
    }
}

/// A fresh [`ShellState`]: the rail as remembered, everything else closed,
/// the theme and the language as this render resolved them.
fn shell_state() -> ShellState {
    ShellState {
        rail_open: persisted("rail", false),
        menu_open: RwSignal::new(false),
        settings_open: RwSignal::new(false),
        language_open: RwSignal::new(false),
        theme: theme::theme_signal(),
        language: provide_language(),
        language_path: RwSignal::new(Vec::new()),
    }
}

/// The rail's width, open or collapsed.
fn rail_width(open: bool) -> &'static str {
    if open { RAIL_OPEN } else { RAIL_CLOSED }
}

/// The default of `AppShell`'s `settings`: one Appearance section, the
/// theme as a [`ThemePicker`]. `ThemePicker` reads [`theme::theme_signal`]
/// itself (the very same instance `AppShell` already provided as
/// `ShellState::theme`, through context), so nothing needs threading here.
/// The section re-renders itself when the language changes (its title is
/// a plain string); the dialog around it stays mounted.
fn default_settings() -> ViewFn {
    (move || {
        let t = strings();
        (move || {
            view! {
                <DialogSection title=td_string!(t.locale(), appearance)>
                    <ThemePicker />
                </DialogSection>
            }
        })
        .into_any()
    })
    .into()
}

/// The Settings dialog, opened by both the desktop gear and the mobile row
/// through the same shared `ShellState::settings_open`. Built once, with
/// the shell: its title follows the language in place. Rebuilding it on a
/// switch would install one more Escape listener and focus manager each
/// time (window listeners outlive the view they came from).
#[component]
fn SettingsDialog(settings: ViewFn) -> impl IntoView {
    let state = expect_context::<ShellState>();
    let controls = Disclosed::new("pc-settings", state.settings_open);
    let t = strings();
    let title = Signal::derive(move || td_string!(t.locale(), settings).to_owned());
    view! {
        <Dialog controls=controls title=title>
            {settings.run()}
        </Dialog>
    }
}

/// Escape closes the rail and the mobile navigation, or only the language
/// menu or the Settings dialog when one is open (`escape`). The browser
/// only: there is no window while the server renders.
fn close_on_escape(state: ShellState) {
    #[cfg(feature = "hydrate")]
    let _ = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            escape(state);
        }
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = state;
}

/// What Escape does to the shell, innermost first: with the desktop
/// language menu or the Settings dialog open, it closes that one only —
/// the rail stays as it is (and `pc_rail` is not rewritten), and so does
/// the mobile navigation Settings may have been opened from; drilled into
/// the mobile languages, it backs out one level, the navigation still
/// open (the row has no focus once drilled, so no keydown of its own
/// could); otherwise it closes the rail and the mobile navigation. The
/// shell's listener runs before the menu's and the dialog's own (it is
/// registered first, in `AppShell`'s body), so it still sees them open;
/// theirs then find nothing left to close.
#[cfg(any(feature = "hydrate", test))]
fn escape(state: ShellState) {
    for open in [state.language_open, state.settings_open] {
        if open.get_untracked() {
            open.set(false);
            return;
        }
    }
    if state.menu_open.get_untracked() && !state.language_path.get_untracked().is_empty() {
        state.language_path.update(|path| {
            path.pop();
        });
        return;
    }
    mobile_navigation(state).set_open(false);
    state.rail_open.set(false);
}

/// Mirrors `theme` onto the `<html data-theme>` attribute as it changes, so
/// a choice applies without a reload. The browser only: the server already
/// set the right attribute for the first render ([`crate::theme::theme_attribute`]).
fn apply_theme(theme: RwSignal<Theme>) {
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(root) = document().document_element() {
            match theme.get().attribute() {
                Some(value) => {
                    let _ = root.set_attribute("data-theme", value);
                }
                None => {
                    let _ = root.remove_attribute("data-theme");
                }
            }
        }
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = theme;
}

/// Mirrors `language` onto the `<html lang>` attribute as it changes, so
/// a choice applies without a reload. The browser only: the server's shell
/// already wrote the request's language there for the first render.
fn apply_language(language: RwSignal<Language>) {
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(root) = document().document_element() {
            let _ = root.set_attribute("lang", language.get().code());
        }
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = language;
}

/// The lotus button that opens and closes `controls` (the rail, or the
/// mobile navigation), named "Open navigation" / "Close navigation" in the
/// current language: rebuilt when it changes.
#[component]
fn LotusToggle<D: Disclosure>(controls: D, logo: &'static str) -> impl IntoView {
    let t = strings();
    move || {
        let open = td_string!(t.locale(), open_navigation);
        let close = td_string!(t.locale(), close_navigation);
        view! {
            <DisclosureButton
                controls=controls
                variant=Variant::Icon
                label_closed=open
                label_open=close
            >
                <Lotus logo=logo />
            </DisclosureButton>
        }
    }
}

/// The desktop rail: the lotus opens and closes it; the name shows when
/// open. The Language and Settings entries at its bottom open
/// [`LanguageMenu`] and [`SettingsDialog`], siblings of the rail (not
/// nested in it): they float over the page, not clipped by the rail's content
/// block's `overflow-hidden`. The nav stretches with the page row; its content
/// is a sticky flex column capped at the viewport height.
#[component]
fn DesktopRail(
    logo: &'static str,
    nav: Option<NavList>,
    env_badge: Option<EnvBadgeInfo>,
) -> impl IntoView {
    let ShellState {
        rail_open,
        language_open,
        language,
        ..
    } = expect_context::<ShellState>();
    let language_menu = Disclosed::new("pc-language", language_open);
    view! {
        <nav
            id="pc-rail"
            aria-label="PaliCanon"
            class="z-30 hidden shrink-0 border-r border-line bg-rail transition-[width] duration-200 sm:block"
            style:width=move || rail_width(rail_open.get())
            data-open=move || rail_open.get().to_string()
        >
            <div class="sticky top-0 flex h-full max-h-screen flex-col overflow-hidden">
                <div class="flex items-center gap-3 p-3.5">
                    <LotusToggle controls=Disclosed::new("pc-rail", rail_open) logo=logo />
                    <Wordmark shown=rail_open />
                </div>
                <div class="flex flex-col gap-1 overflow-y-auto px-3.5 pb-3.5">
                    {nav.map(|nav| view! { <RailSections nav=nav open=rail_open /> })}
                </div>
                <RailBottom language_menu=language_menu shown=rail_open env_badge=env_badge />
            </div>
        </nav>
        <LanguageMenu
            controls=language_menu
            current=language
            rail_width=Signal::derive(move || rail_width(rail_open.get()))
        />
    }
}

/// The rail's bottom block, pinned there by `mt-auto` (the rail's sticky
/// content block is a flex column): the Language entry, then Settings, then the env badge when
/// given. The badge is hidden while the rail is collapsed (it would not fit,
/// and it must not hold space under the entries), shown when the rail opens.
#[component]
fn RailBottom(
    language_menu: Disclosed,
    #[prop(into)] shown: Signal<bool>,
    env_badge: Option<EnvBadgeInfo>,
) -> impl IntoView {
    let ShellState {
        settings_open,
        language,
        ..
    } = expect_context::<ShellState>();
    let settings = Disclosed::new("pc-settings", settings_open);
    view! {
        <div class="mt-auto flex flex-col gap-1 p-3.5">
            <RailLanguageEntry controls=language_menu current=language shown=shown />
            <RailSettingsEntry settings=settings shown=shown />
            {env_badge
                .map(|info| {
                    view! {
                        <div class="mt-2" class:hidden=move || !shown.get()>
                            <EnvBadge env=info.env sha=info.sha />
                        </div>
                    }
                })}
        </div>
    }
}

/// The Settings entry: a gear, and the label, fading like the nav entries
/// above it while the rail is collapsed; rebuilt when the language changes.
#[component]
fn RailSettingsEntry(settings: Disclosed, #[prop(into)] shown: Signal<bool>) -> impl IntoView {
    let t = strings();
    move || {
        let label = td_string!(t.locale(), settings);
        view! {
            <DisclosureButton
                controls=settings
                variant=Variant::Nav
                label_closed=label
                label_open=label
            >
                <Icon name=IconName::Gear />
                <span class="transition-opacity duration-150" class:opacity-0=move || !shown.get()>
                    {label}
                </span>
            </DisclosureButton>
        }
    }
}

/// The mobile top bar, and the full-screen navigation its lotus opens. The
/// env badge, when given, sits on the right, truncating its env (the
/// branch) rather than pushing the lotus and name off screen.
#[component]
fn MobileNav(
    logo: &'static str,
    nav: Option<NavList>,
    env_badge: Option<EnvBadgeInfo>,
) -> impl IntoView {
    let state = expect_context::<ShellState>();
    let menu_open = state.menu_open;
    let menu = mobile_navigation(state);
    view! {
        <header class="sticky top-0 z-30 flex h-14 items-center gap-2 border-b border-line bg-rail px-2 sm:hidden">
            <LotusToggle controls=menu logo=logo />
            <Wordmark shown=Signal::derive(|| true) />
            {env_badge
                .map(|info| {
                    view! {
                        <div class="ml-auto max-w-[8rem]">
                            <EnvBadge env=info.env sha=info.sha />
                        </div>
                    }
                })}
        </header>
        <div
            id="pc-menu"
            role="dialog"
            aria-modal="true"
            aria-label="PaliCanon"
            class="fixed inset-0 z-40 flex flex-col bg-rail sm:hidden"
            hidden=move || !menu_open.get()
        >
            <MobileMenuHeader logo=logo menu=menu />
            // Clicks bubble up from the entries: choosing one closes the menu.
            // Language and Settings sit outside this container: choosing a
            // language or opening Settings must not close the mobile menu.
            <div
                class="flex flex-col gap-1 overflow-y-auto p-3"
                on:click=move |_| menu.set_open(false)
            >
                {nav.map(|nav| view! { <MobileSections nav=nav /> })}
            </div>
            <MobileBottom />
        </div>
    }
}

/// What opens and closes the full-screen navigation: the lotus, the ✕,
/// an entry, and Escape all go through it.
fn mobile_navigation(state: ShellState) -> MobileNavigation {
    MobileNavigation {
        open: state.menu_open,
        language_path: state.language_path,
    }
}

/// The full-screen navigation's disclosure (`pc-menu`): closing it also
/// brings the language row back to its top level, so the navigation
/// always reopens on the Language row, never drilled into the languages.
#[derive(Clone, Copy, Debug)]
struct MobileNavigation {
    open: RwSignal<bool>,
    language_path: Path,
}

impl Disclosure for MobileNavigation {
    fn id(&self) -> &'static str {
        "pc-menu"
    }
    fn is_open(&self) -> bool {
        self.open.get()
    }
    fn set_open(&self, open: bool) {
        if !open {
            self.language_path.set(Vec::new());
        }
        self.open.set(open);
    }
}

/// The bottom of the full-screen navigation, pinned there: the Language
/// row (drilling down to the languages), then Settings.
#[component]
fn MobileBottom() -> impl IntoView {
    let ShellState {
        settings_open,
        language,
        language_path,
        ..
    } = expect_context::<ShellState>();
    let settings = Disclosed::new("pc-settings", settings_open);
    view! {
        <div class="mt-auto flex flex-col gap-1 border-t border-line p-3">
            <MobileLanguage current=language path=language_path />
            <MobileSettings settings=settings />
        </div>
    }
}

/// The Settings row of the full-screen navigation: opens the very same
/// [`SettingsDialog`] the desktop gear does; rebuilt when the language
/// changes.
#[component]
fn MobileSettings(settings: Disclosed) -> impl IntoView {
    let t = strings();
    move || {
        let label = td_string!(t.locale(), settings);
        view! {
            <DisclosureButton
                controls=settings
                variant=Variant::Nav
                size=Size::Lg
                label_closed=label
                label_open=label
            >
                <Icon name=IconName::Gear />
                <span>{label}</span>
            </DisclosureButton>
        }
    }
}

/// The top of the full-screen navigation: the lotus (closes it), the name,
/// and a close cross, named "Close navigation" in the current language.
#[component]
fn MobileMenuHeader(logo: &'static str, menu: MobileNavigation) -> impl IntoView {
    let t = strings();
    view! {
        <div class="flex h-14 items-center gap-2 border-b border-line px-2">
            <LotusToggle controls=menu logo=logo />
            <Wordmark shown=Signal::derive(|| true) />
            {move || {
                view! {
                    <Button
                        variant=Variant::Icon
                        label=td_string!(t.locale(), close_navigation)
                        layout="ml-auto"
                        on_click=move |_| menu.set_open(false)
                    >
                        <svg
                            width="16"
                            height="16"
                            viewBox="0 0 16 16"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            aria-hidden="true"
                        >
                            <path d="m4 4 8 8M12 4l-8 8" />
                        </svg>
                    </Button>
                }
            }}
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::language::{Language, language_signal, provide_language};

    fn render(rail_cookie: Option<&str>) -> String {
        render_cookies(
            rail_cookie
                .map(|value| format!("pc_rail={value}"))
                .as_deref(),
        )
    }

    /// The shell rendered for a request carrying `cookies` (a whole
    /// `Cookie` header), if any.
    fn render_cookies(cookies: Option<&str>) -> String {
        let owner = Owner::new();
        owner.with(|| {
            if let Some(cookies) = cookies {
                let request = http::Request::builder()
                    .header(http::header::COOKIE, cookies)
                    .body(())
                    .unwrap();
                provide_context(request.into_parts().0);
            }
            view! {
                <AppShell>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        })
    }

    /// The desktop rail's markup alone: its `<nav>`, open tag to close.
    fn rail(html: &str) -> &str {
        let start = html.find(r#"<nav id="pc-rail""#).expect("rail");
        let end = start + html[start..].find("</nav>").expect("rail end");
        &html[start..end]
    }

    /// The mobile full-screen navigation's markup, to the end of the page.
    fn mobile(html: &str) -> &str {
        &html[html.find(r#"id="pc-menu""#).expect("mobile menu")..]
    }

    /// The globe's SVG, as an `Icon` renders it.
    fn globe() -> String {
        Owner::new().with(|| view! { <Icon name=IconName::Globe /> }.to_html())
    }

    #[test]
    fn the_rails_bottom_block_has_no_rule_above_it() {
        let html = render(None);
        let rail = rail(&html);
        let from = &rail[rail.find("mt-auto").expect("bottom block")..];
        let tag = &from[..from.find('>').unwrap()];
        assert!(!tag.contains("border-t"), "{tag}");
    }

    #[test]
    fn the_language_entry_sits_above_settings_in_the_rail() {
        let html = render(None);
        let rail = rail(&html);
        let language = rail
            .find(r#"aria-controls="pc-language""#)
            .expect("a language entry in the rail");
        let settings = rail
            .find(r#"aria-controls="pc-settings""#)
            .expect("settings");
        assert!(language < settings, "{rail}");
    }

    #[test]
    fn the_language_entry_shows_the_globe_and_the_current_name() {
        for (code, name) in [("en", "English"), ("fr", "Français"), ("vi", "Tiếng Việt")] {
            let html = render_cookies(Some(&format!("pc_lang={code}")));
            let rail = rail(&html);
            let entry = &rail[rail.find(r#"aria-controls="pc-language""#).unwrap()..];
            let entry = &entry[..entry.find("</button>").unwrap()];
            assert!(entry.contains(&globe()), "{code}: {entry}");
            assert!(entry.contains(name), "{code}: {entry}");
        }
    }

    #[test]
    fn the_language_entry_is_named_language_in_the_current_language() {
        let html = render_cookies(Some("pc_lang=vi"));
        let rail = rail(&html);
        let entry = &rail[rail.find(r#"aria-controls="pc-language""#).unwrap()..];
        let tag = &entry[..entry.find('>').unwrap()];
        assert!(tag.contains(r#"aria-label="Ngôn ngữ""#), "{tag}");
    }

    #[test]
    fn the_language_menu_lists_the_three_in_order_with_the_current_one_checked() {
        let html = render_cookies(Some("pc_lang=fr"));
        let start = html.find(r#"id="pc-language""#).expect("menu panel");
        let end = start
            + html[start..]
                .find(r#"<div class="flex min-w-0 flex-1 flex-col""#)
                .expect("the page column after it");
        // The mobile navigation, with its own three choices, comes before
        // it; the page column, after it.
        let panel = &html[start..end];
        let rows: Vec<&str> = panel.split(r#"role="menuitemradio""#).skip(1).collect();
        assert_eq!(rows.len(), 3, "{panel}");
        for (row, name) in rows.iter().zip(["English", "Français", "Tiếng Việt"]) {
            let row = &row[..row.find("</button>").unwrap()];
            assert!(row.contains(name), "{name}: {row}");
            let checked = row.contains(r#"aria-checked="true""#);
            assert_eq!(checked, name == "Français", "{name}: {row}");
        }
    }

    #[test]
    fn the_language_menu_is_outside_the_clipped_rail() {
        let html = render(None);
        assert!(!rail(&html).contains(r#"id="pc-language""#), "{html}");
        assert_eq!(html.matches(r#"id="pc-language""#).count(), 1, "{html}");
    }

    #[test]
    fn the_mobile_language_row_sits_above_settings() {
        let html = render(None);
        let mobile = mobile(&html);
        let language = mobile.find(&globe()).expect("a language row");
        let settings = mobile
            .find(r#"aria-controls="pc-settings""#)
            .expect("settings");
        assert!(language < settings, "{mobile}");
    }

    /// Picking a language on mobile keeps the full-screen navigation open:
    /// the row is in the bottom block, not in the entries' container,
    /// whose clicks close the navigation.
    #[test]
    fn the_mobile_language_row_is_outside_the_closing_container() {
        let html = render(None);
        let mobile = mobile(&html);
        let entries = mobile.find("overflow-y-auto p-3").expect("entries");
        let bottom = mobile.find("mt-auto").expect("bottom block");
        let row = mobile.find(&globe()).expect("language row");
        assert!(entries < bottom && bottom < row, "{mobile}");
    }

    /// Drilled into the languages, the navigation closed, then reopened:
    /// it opens on the Language row again, Back and the languages hidden
    /// (T20).
    #[test]
    fn closing_the_navigation_resets_the_language_drill() {
        let html = Owner::new().with(|| {
            let shell = view! {
                <AppShell>
                    <p>"page"</p>
                </AppShell>
            };
            let state = expect_context::<ShellState>();
            let navigation = mobile_navigation(state);
            navigation.set_open(true);
            state.language_path.set(vec![0]);
            navigation.set_open(false);
            navigation.set_open(true);
            shell.to_html()
        });
        let mobile = mobile(&html);
        let opening_tag = |marker: &str| {
            let at = mobile
                .find(marker)
                .unwrap_or_else(|| panic!("{marker}: {mobile}"));
            let start = mobile[..at].rfind('<').unwrap();
            &mobile[start..at + mobile[at..].find('>').unwrap()]
        };
        assert!(
            !opening_tag(r#"aria-haspopup="true""#).contains("hidden"),
            "the Language row shows: {mobile}"
        );
        assert!(
            opening_tag(r#"role="group""#).contains("hidden"),
            "the languages are hidden: {mobile}"
        );
        let back = mobile.find(">Back<").expect("the Back row");
        let back = &mobile[mobile[..back].rfind("<button").unwrap()..back];
        assert!(back.contains("hidden"), "Back is hidden: {back}");
    }

    #[test]
    fn switching_the_language_renames_the_mobile_row() {
        let html = Owner::new().with(|| {
            let shell = view! {
                <AppShell>
                    <p>"page"</p>
                </AppShell>
            };
            language_signal().set(Language::Fr);
            shell.to_html()
        });
        let mobile = mobile(&html);
        let row = &mobile[mobile.find(&globe()).unwrap()..];
        let row = &row[..row.find("</button>").unwrap()];
        assert!(row.contains("Français"), "{row}");
    }

    #[test]
    fn desktop_and_mobile_show_the_same_language() {
        let html = render_cookies(Some("pc_lang=vi"));
        assert!(rail(&html).contains("Tiếng Việt"), "{html}");
        let mobile = mobile(&html);
        let row = &mobile[mobile.find(&globe()).unwrap()..];
        assert!(
            row[..row.find("</button>").unwrap()].contains("Tiếng Việt"),
            "{row}"
        );
    }

    #[test]
    fn the_shell_speaks_the_current_language() {
        let html = render_cookies(Some("pc_lang=vi"));
        assert!(html.contains("Cài đặt"), "Settings: {html}");
        assert!(html.contains("Giao diện"), "Appearance: {html}");
        assert!(
            html.contains(r#"aria-label="Mở thanh điều hướng""#),
            "{html}"
        );
        assert!(
            html.contains(r#"aria-label="Đóng thanh điều hướng""#),
            "the ✕: {html}"
        );
        for english in [
            "Settings",
            "Appearance",
            "Open navigation",
            "Close navigation",
        ] {
            assert!(!html.contains(english), "{english}: {html}");
        }
    }

    #[test]
    fn switching_the_language_re_renders_the_language_entry_and_settings() {
        let html = Owner::new().with(|| {
            let language = language_signal();
            let shell = view! {
                <AppShell>
                    <p>"page"</p>
                </AppShell>
            };
            language.set(Language::Fr);
            shell.to_html()
        });
        let rail = rail(&html);
        assert!(rail.contains("Français"), "{rail}");
        assert!(!rail.contains("English"), "{rail}");
        assert!(rail.contains("Paramètres"), "{rail}");
        assert!(!html.contains("Settings"), "{html}");
        assert!(!html.contains("Open navigation"), "{html}");
    }

    /// T5, as far as SSR goes: switched twice, the shell still holds one
    /// desktop language menu, its check on the last choice. (Its window
    /// listeners are browser-only: task 024.)
    #[test]
    fn switching_twice_keeps_one_language_menu() {
        let html = Owner::new().with(|| {
            let shell = view! {
                <AppShell>
                    <p>"page"</p>
                </AppShell>
            };
            language_signal().set(Language::Fr);
            language_signal().set(Language::Vi);
            shell.to_html()
        });
        assert_eq!(html.matches(r#"id="pc-language""#).count(), 1, "{html}");
        let panel = &html[html.find(r#"id="pc-language""#).unwrap()..];
        let checked = &panel[panel.find(r#"aria-checked="true""#).unwrap()..];
        assert!(checked[..checked.find("</button>").unwrap()].contains("Tiếng Việt"));
    }

    /// T11: switched twice, every piece re-rendered on a switch speaks the
    /// last language — an app's nav entry, the rail's language entry, the
    /// Settings entry and the theme picker.
    #[test]
    fn switching_twice_re_renders_in_the_last_language() {
        use crate::{NavEntry, NavSection};
        let html = Owner::new().with(|| {
            let language = language_signal();
            let sections = Signal::derive(move || {
                let label = match language.get() {
                    Language::En => "Buttons",
                    Language::Fr => "Boutons",
                    Language::Vi => "Nút",
                };
                let entry = NavEntry {
                    label,
                    icon: IconName::Button,
                    href: "/buttons",
                };
                vec![NavSection {
                    title: None,
                    entries: vec![entry],
                }]
            });
            let nav = NavList::new(sections, "/".to_string());
            let shell = view! {
                <AppShell nav=nav>
                    <p>"page"</p>
                </AppShell>
            };
            language.set(Language::Fr);
            language.set(Language::Vi);
            shell.to_html()
        });
        let rail = rail(&html);
        for vi in ["Nút", "Tiếng Việt", "Cài đặt"] {
            assert!(rail.contains(vi), "{vi}: {rail}");
        }
        assert!(html.contains("Sáng"), "the theme picker's Light: {html}");
        // ("Français" stays: the language menus list every endonym.)
        for other in ["Boutons", "Buttons", "Paramètres", "Clair", ">Light<"] {
            assert!(!html.contains(other), "{other}: {html}");
        }
    }

    #[test]
    fn a_collapsed_rail_fades_the_language_label_like_settings() {
        for (rail_open, faded) in [("false", true), ("true", false)] {
            let html = render(Some(rail_open));
            let rail = rail(&html);
            for label in ["English", "Settings"] {
                let before = &rail[..rail.find(&format!(">{label}")).unwrap()];
                let span = &before[before.rfind("<span").unwrap()..];
                assert_eq!(span.contains("opacity-0"), faded, "{label}: {span}");
            }
        }
    }

    #[test]
    fn the_language_and_settings_disclosures_are_distinct() {
        let html = render(None);
        assert_eq!(
            html.matches(r#"aria-controls="pc-language""#).count(),
            1,
            "{html}"
        );
        assert_eq!(
            html.matches(r#"aria-controls="pc-settings""#).count(),
            2,
            "{html}"
        );
        assert_eq!(html.matches(r#"id="pc-language""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"id="pc-settings""#).count(), 1, "{html}");
    }

    /// The shell's state, built as `AppShell` builds it, with the rail
    /// open and the desktop language menu `language_open`.
    fn open_shell(language_open: bool) -> ShellState {
        let _shell = view! {
            <AppShell>
                <p>"page"</p>
            </AppShell>
        };
        let state = expect_context::<ShellState>();
        state.rail_open.set(true);
        state.language_open.set(language_open);
        state
    }

    #[test]
    fn escape_on_the_open_language_menu_closes_only_the_menu() {
        Owner::new().with(|| {
            let state = open_shell(true);
            escape(state);
            assert!(!state.language_open.get_untracked(), "the menu closes");
            assert!(
                state.rail_open.get_untracked(),
                "the rail stays open, and pc_rail is not rewritten"
            );
        });
    }

    #[test]
    fn escape_on_the_open_settings_dialog_closes_only_the_dialog() {
        Owner::new().with(|| {
            let state = open_shell(false);
            state.settings_open.set(true);
            escape(state);
            assert!(
                state.rail_open.get_untracked(),
                "the rail stays open, and pc_rail is not rewritten"
            );
            assert!(!state.settings_open.get_untracked(), "the dialog closes");
        });
    }

    /// Settings opened from the mobile navigation: Escape lands back in
    /// that navigation, it does not close it too.
    #[test]
    fn escape_on_settings_opened_from_the_mobile_navigation_keeps_the_navigation() {
        Owner::new().with(|| {
            let state = open_shell(false);
            state.menu_open.set(true);
            state.settings_open.set(true);
            escape(state);
            assert!(state.menu_open.get_untracked(), "the navigation stays");
            assert!(!state.settings_open.get_untracked(), "the dialog closes");
        });
    }

    /// Drilled into the mobile languages, Escape backs out one level, as
    /// a submenu does on desktop: the navigation stays open, and the rail
    /// as it was (`pc_rail` is not rewritten) (T19).
    #[test]
    fn escape_drilled_into_the_mobile_languages_backs_out_one_level() {
        Owner::new().with(|| {
            let state = open_shell(false);
            state.menu_open.set(true);
            state.language_path.set(vec![0]);
            escape(state);
            assert!(
                state.language_path.get_untracked().is_empty(),
                "back at the Language row"
            );
            assert!(state.menu_open.get_untracked(), "the navigation stays");
            assert!(
                state.rail_open.get_untracked(),
                "the rail stays open, and pc_rail is not rewritten"
            );
        });
    }

    /// Settings opened from the drilled navigation: Escape closes the
    /// dialog first (innermost first), the drill stays where it was.
    #[test]
    fn escape_on_settings_over_the_drilled_languages_closes_only_the_dialog() {
        Owner::new().with(|| {
            let state = open_shell(false);
            state.menu_open.set(true);
            state.language_path.set(vec![0]);
            state.settings_open.set(true);
            escape(state);
            assert!(!state.settings_open.get_untracked(), "the dialog closes");
            assert_eq!(state.language_path.get_untracked(), [0], "still drilled");
            assert!(state.menu_open.get_untracked(), "the navigation stays");
        });
    }

    #[test]
    fn escape_with_no_menu_open_closes_the_rail_and_the_mobile_navigation() {
        Owner::new().with(|| {
            let state = open_shell(false);
            state.menu_open.set(true);
            escape(state);
            assert!(!state.rail_open.get_untracked());
            assert!(!state.menu_open.get_untracked());
        });
    }

    /// The page's one language signal, provided at the app root: a
    /// component beside the shell that asks first, in its own owner, and
    /// the shell, in a sibling owner, read that same signal (T4).
    #[test]
    fn the_root_provides_one_language_signal_for_the_whole_page() {
        let root = Owner::new();
        root.with(|| {
            let page = provide_language();
            let beside = root.child().with(language_signal);
            let shell = root.child().with(|| {
                let _shell = view! {
                    <AppShell>
                        <p>"page"</p>
                    </AppShell>
                };
                expect_context::<ShellState>().language
            });
            assert_eq!(beside, page, "a sibling of the shell");
            assert_eq!(shell, page, "the shell's own");
        });
    }

    /// An app that forgets the root call still gets one signal below the
    /// shell: `AppShell` provides it, and every owner under it shares it.
    #[test]
    fn without_a_root_call_the_shell_provides_the_signal() {
        let root = Owner::new();
        root.with(|| {
            let _shell = view! {
                <AppShell>
                    <p>"page"</p>
                </AppShell>
            };
            let shell = expect_context::<ShellState>().language;
            let below = root.child().with(|| root.child().with(language_signal));
            assert_eq!(below, shell);
        });
    }

    #[test]
    fn closed_by_default_with_the_lotus_only() {
        let html = render(None);
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains(r#"aria-label="Open navigation""#), "{html}");
        assert!(html.contains(r#"data-open="false""#), "{html}");
        assert!(html.contains("4rem"), "{html}");
        assert!(html.contains("/lotus.png"), "{html}");
        assert!(
            html.contains("<p>page</p>"),
            "the service's page is rendered: {html}"
        );
    }

    #[test]
    fn the_mobile_menu_is_hidden_when_a_page_loads() {
        let html = render(Some("true"));
        let menu = &html[html.find(r#"id="pc-menu""#).expect("menu")..];
        let tag_end = menu.find('>').unwrap();
        assert!(menu[..tag_end].contains("hidden"), "{}", &menu[..tag_end]);
    }

    #[test]
    fn an_open_rail_is_remembered_from_the_cookie() {
        let html = render(Some("true"));
        assert!(html.contains(r#"data-open="true""#), "{html}");
        assert!(html.contains("13.5rem"), "{html}");
        assert!(html.contains(r#"aria-label="Close navigation""#), "{html}");
    }

    #[test]
    fn the_app_navigation_shows_in_the_rail_and_the_mobile_menu() {
        use crate::{IconName, NavEntry, NavSection};
        let html = Owner::new().with(|| {
            let nav = NavList::new(
                vec![NavSection {
                    title: Some("Components"),
                    entries: vec![NavEntry {
                        label: "Buttons",
                        icon: IconName::Button,
                        href: "/buttons",
                    }],
                }],
                "/".to_string(),
            );
            view! {
                <AppShell nav=nav>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        assert_eq!(
            html.matches(r#"href="/buttons""#).count(),
            2,
            "rail and menu: {html}"
        );
        assert!(
            !html.contains("aria-current"),
            "no entry is current on /: {html}"
        );
    }

    #[test]
    fn without_navigation_only_the_lotus_shows() {
        assert!(!render(None).contains("<a"));
    }

    #[test]
    fn a_garbled_cookie_falls_back_to_closed() {
        let html = render(Some("perhaps"));
        assert!(html.contains(r#"data-open="false""#), "{html}");
    }

    #[test]
    fn the_desktop_gear_and_mobile_row_share_one_settings_dialog() {
        // Both triggers name the same dialog: only one of the two is ever
        // visible at a time (the rail and the mobile nav already are), so
        // sharing an id never confuses an outside click the way two
        // overlapping popovers once could.
        let html = render(None);
        assert_eq!(
            html.matches(r#"aria-controls="pc-settings""#).count(),
            2,
            "the gear and the mobile row: {html}"
        );
        assert_eq!(
            html.matches(r#"id="pc-settings""#).count(),
            1,
            "a single dialog panel: {html}"
        );
        assert!(html.contains(r#"role="dialog""#), "{html}");
        assert!(
            html.contains(r#"aria-labelledby="pc-settings-title""#),
            "{html}"
        );
    }

    #[test]
    fn settings_default_to_appearance_with_theme_picture_cards() {
        let html = render(None);
        assert!(html.contains("Appearance"), "{html}");
        assert!(html.contains(r#"role="radiogroup""#), "{html}");
        assert!(html.contains(r#"aria-label="Theme""#), "{html}");
        assert_eq!(html.matches(r#"role="radio""#).count(), 3, "{html}");
        assert!(html.contains("Light"), "{html}");
        assert!(html.contains("Dark"), "{html}");
        assert!(html.contains("System"), "{html}");
        // The picture cards, not the former Segmented chips: each card is a
        // page preview forcing its own palette, and only the checked one is
        // a tab stop.
        assert!(html.contains(r#"data-theme="light""#), "{html}");
        assert!(html.contains(r#"data-theme="dark""#), "{html}");
        assert_eq!(html.matches(r#"tabindex="0""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"tabindex="-1""#).count(), 2, "{html}");
    }

    /// The README's item rows say what the code above renders: the
    /// default Settings hold a `ThemePicker`, never the former `Segmented`,
    /// and the Theme row names the `ThemePicker` it is.
    #[test]
    fn the_readme_rows_name_the_theme_picker_of_the_default_settings() {
        let readme = include_str!("../README.md");
        let row = |item: &str| {
            readme
                .lines()
                .find(|line| line.starts_with(&format!("| `{item}`")))
                .unwrap_or_else(|| panic!("a README row for {item}"))
        };
        for item in ["AppShell", "Theme"] {
            let line = row(item);
            assert!(line.contains("`ThemePicker`"), "{item}: {line}");
            assert!(!line.contains("Segmented"), "{item}: {line}");
        }
    }

    /// T1: on a phone the top bar sits in the column that carries
    /// `min-h-screen`, so a short page is one screen, top bar included —
    /// then the rail-and-page row, then the page, then its footer.
    /// The README rows and the crate doc say the footer's final layout
    /// and the shell's `footer` slot (review Δ4).
    #[test]
    fn the_readme_and_crate_doc_say_the_final_footer_layout() {
        let readme = include_str!("../README.md");
        let row = |item: &str| {
            readme
                .lines()
                .find(|line| line.starts_with(&format!("| `{item}`")))
                .unwrap_or_else(|| panic!("a README row for {item}"))
        };
        let footer = row("Footer");
        for said in ["1fr/auto/1fr", "right edge", "no credits"] {
            assert!(footer.contains(said), "{said}: {footer}");
        }
        assert!(!footer.contains("one row from `sm` up"), "{footer}");
        let shell = row("AppShell");
        for said in [
            "`footer`",
            "full height",
            "hidden while the rail is collapsed",
        ] {
            assert!(shell.contains(said), "{said}: {shell}");
        }
        let lib = include_str!("lib.rs");
        assert!(!lib.contains("laid out one row on desktop"), "lib.rs");
        assert!(lib.contains("1fr/auto/1fr"), "lib.rs");
    }

    #[test]
    fn the_footer_sits_in_the_page_column_beside_the_full_height_rail() {
        let html = Owner::new().with(|| {
            view! { <AppShell footer=|| view! { <footer id="f">"foot"</footer> }>"page"</AppShell> }
                .to_html()
        });
        let column = html
            .find(r#"<div class="flex min-h-screen flex-col""#)
            .expect("the screen-tall column");
        let header = html.find("<header").expect("the mobile top bar");
        let row = html
            .find(r#"<div class="flex flex-1 flex-col sm:flex-row""#)
            .expect("the rail-and-page row");
        let rail_at = html.find(r#"<nav id="pc-rail""#).expect("the rail");
        let page_column = html
            .find(r#"<div class="flex min-w-0 flex-1 flex-col""#)
            .expect("the page column");
        // The children, inside the page column (a theme card's class may
        // say `page` earlier, in the Settings dialog).
        let page = page_column + html[page_column..].find("page").expect("page");
        let footer = html.find("<footer").expect("footer");
        assert!(column < header, "the column holds the top bar: {html}");
        assert!(header < row, "the top bar, then the row: {html}");
        assert!(row < rail_at, "the row holds the rail: {html}");
        assert!(rail_at < page_column, "the rail, then the page: {html}");
        assert!(page < footer, "the page, then its footer: {html}");
        let mut plain = html.clone();
        while let Some(start) = plain.find("<!--") {
            let end = start + plain[start..].find("-->").expect("comment end") + 3;
            plain.replace_range(start..end, "");
        }
        assert!(plain.contains(r#"<div class="mt-auto"><footer"#), "{plain}");
        let rail = rail(&html);
        let open_tag = &rail[..rail.find('>').expect("nav open tag")];
        for banned in ["sticky", "h-screen", "fixed"] {
            assert!(!open_tag.contains(banned), "{banned}: {open_tag}");
        }
        assert!(rail.contains("sticky top-0"), "{rail}");
        assert!(rail.contains("max-h-screen"), "{rail}");
        assert!(!html.contains("min-h-screen sm:flex"), "{html}");
        assert!(html.contains("flex min-w-0 flex-1 flex-col"), "{html}");
        assert!(!html.contains("sm:pl-(--pc-rail)"), "{html}");
    }

    #[test]
    fn no_footer_renders_no_footer_element() {
        let html = Owner::new().with(|| view! { <AppShell>"page"</AppShell> }.to_html());
        assert!(!html.contains("<footer"), "{html}");
    }

    #[test]
    fn no_env_badge_by_default() {
        let html = render(None);
        assert!(!html.contains("inline-flex max-w-full"), "{html}");
    }

    #[test]
    fn an_env_badge_shows_at_the_rail_bottom_and_in_the_mobile_top_bar() {
        use crate::EnvBadgeInfo;
        let html = Owner::new().with(|| {
            view! {
                <AppShell env_badge=EnvBadgeInfo {
                    env: "main",
                    sha: "a1b2c3f",
                }>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        assert_eq!(
            html.matches(r#"title="main · a1b2c3f""#).count(),
            2,
            "the rail (desktop) and the top bar (mobile): {html}"
        );
    }

    #[test]
    fn a_long_branch_truncates_in_the_mobile_top_bar() {
        use crate::EnvBadgeInfo;
        let long = "feat/a-very-long-branch-name-that-would-not-fit";
        let html = Owner::new().with(|| {
            view! {
                <AppShell env_badge=EnvBadgeInfo {
                    env: long,
                    sha: "a1b2c3f",
                }>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        let header = &html[html.find("<header").unwrap()..html.find("</header>").unwrap()];
        assert!(header.contains("max-w-[8rem]"), "{header}");
        assert!(header.contains(long), "{header}");
    }

    /// The open tag of the rail badge's wrapper (the tag whose class starts
    /// with `mt-2`), for a rail cookie value.
    fn badge_wrapper(rail_cookie: &str) -> String {
        use crate::EnvBadgeInfo;
        let html = Owner::new().with(|| {
            let request = http::Request::builder()
                .header(http::header::COOKIE, format!("pc_rail={rail_cookie}"))
                .body(())
                .unwrap();
            provide_context(request.into_parts().0);
            view! {
                <AppShell env_badge=EnvBadgeInfo {
                    env: "main",
                    sha: "abc1234",
                }>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        let rail = rail(&html);
        let at = rail.find(r#"class="mt-2"#).expect("badge wrapper");
        let start = rail[..at].rfind('<').unwrap();
        rail[start..start + rail[start..].find('>').unwrap()].to_string()
    }

    #[test]
    fn a_collapsed_rails_env_badge_is_out_of_the_flow() {
        let tag = badge_wrapper("false");
        assert!(tag.contains("hidden"), "{tag}");
        assert!(!tag.contains("opacity-0"), "{tag}");
        assert!(
            !tag.contains("transition-opacity"),
            "no fade on a hidden badge: {tag}"
        );
    }

    #[test]
    fn the_rail_badge_shows_once_the_rail_is_open() {
        let tag = badge_wrapper("true");
        assert!(!tag.contains("hidden"), "{tag}");
        assert!(!tag.contains("opacity-0"), "{tag}");
    }

    #[test]
    fn a_services_own_env_name_shows_as_is() {
        use crate::EnvBadgeInfo;
        let html = Owner::new().with(|| {
            view! {
                <AppShell env_badge=EnvBadgeInfo {
                    env: "dev",
                    sha: "a1b2c3f",
                }>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        assert_eq!(
            html.matches(r#"title="dev · a1b2c3f""#).count(),
            2,
            "{html}"
        );
    }

    #[test]
    fn the_rail_bottom_reads_language_then_settings_then_the_env_badge() {
        use crate::EnvBadgeInfo;
        let html = Owner::new().with(|| {
            view! {
                <AppShell env_badge=EnvBadgeInfo {
                    env: "main",
                    sha: "a1b2c3f",
                }>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        let rail = &html[html.find(r#"id="pc-rail""#).unwrap()..html.find("</nav>").unwrap()];
        let at = |needle: &str| {
            rail.find(needle)
                .unwrap_or_else(|| panic!("{needle}: {rail}"))
        };
        let language = at(r#"aria-controls="pc-language""#);
        let settings = at(r#"aria-controls="pc-settings""#);
        let badge = at(r#"title="main · a1b2c3f""#);
        assert!(language < settings && settings < badge, "{rail}");
    }

    /// Mobile, with an env badge: the badge sits in the top bar, and the
    /// full-screen navigation still holds the Language row above Settings
    /// — neither displaces the other (T21).
    #[test]
    fn on_mobile_the_badge_and_the_language_row_show_together() {
        use crate::EnvBadgeInfo;
        let html = Owner::new().with(|| {
            view! {
                <AppShell env_badge=EnvBadgeInfo {
                    env: "main",
                    sha: "a1b2c3f",
                }>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        let header = &html[html.find("<header").unwrap()..html.find("</header>").unwrap()];
        assert!(header.contains(r#"title="main · a1b2c3f""#), "{header}");
        let mobile = mobile(&html);
        let language = mobile
            .find(&globe())
            .unwrap_or_else(|| panic!("the Language row: {mobile}"));
        let settings = mobile
            .find(r#"aria-controls="pc-settings""#)
            .expect("Settings");
        assert!(language < settings, "{mobile}");
    }

    #[test]
    fn an_app_can_override_settings_with_its_own_view() {
        let html = Owner::new().with(|| {
            view! {
                <AppShell settings=|| view! { <p>"Say hello"</p> }.into_any()>
                    <p>"page"</p>
                </AppShell>
            }
            .to_html()
        });
        assert!(html.contains("Say hello"), "{html}");
        assert!(
            !html.contains("Appearance"),
            "not kept unless the app adds it back: {html}"
        );
    }

    /// The Settings dialog is wired once, with the shell — never rebuilt
    /// when the language changes: a rebuild would install a new Escape
    /// listener and a new focus manager each time (the window listeners
    /// are not removed with the old view). Built, switched twice, then
    /// opened: exactly one focus manager moves focus into it. A dialog
    /// rebuilt inside a closure is not even wired before a render (no
    /// move), and one rebuilt per switch would move focus once per copy.
    #[test]
    fn switching_the_language_keeps_the_one_settings_dialog() {
        use crate::focus_trap::fake::{self, settle};
        fake::run(async || {
            let _shell = view! {
                <AppShell>
                    <p>"page"</p>
                </AppShell>
            };
            let state = expect_context::<ShellState>();
            settle().await;
            language_signal().set(Language::Fr);
            settle().await;
            language_signal().set(Language::Vi);
            settle().await;
            state.settings_open.set(true);
            settle().await;
            assert_eq!(fake::moves().focus_into, ["pc-settings"]);
        });
    }
}
