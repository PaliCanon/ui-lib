//! The app shell every PaliCanon service shares.
//!
//! - **Desktop** (≥ 640 px): a navigation rail on the left. Collapsed, it
//!   shows the lotus only; the lotus opens it (the name appears) and closes
//!   it. The page beside it moves with it. Open or closed is remembered
//!   ([`persisted`](fn@crate::persisted), cookie `pc_rail`), across pages and subdomains.
//! - **Mobile**: a top bar with the lotus; the lotus opens the navigation
//!   full screen, the lotus or ✕ closes it. Always closed when a page loads.
//! - **Escape** closes whichever is open.
//!
//! Below the lotus, the app's [`NavList`] (none by default): each service
//! adds its own entry when it is created. Choosing an entry in the mobile
//! navigation closes it. Strings are English until the i18n mechanism is
//! defined.
//!
//! **Settings** sits at the bottom of the rail (desktop) and of the
//! full-screen navigation (mobile): a [`Menu`] of `settings`
//! ([`theme::theme_menu`] alone by default; an app overriding it decides
//! whether to keep that entry). On desktop it opens as a [`Menu`] (a
//! [`crate::popover::Popover`] beside the rail, rendered outside the rail's
//! clipped `<nav>` so it is never cut off); on mobile it expands inline,
//! below its own row, as a [`DrillMenu`]. Desktop and mobile each own their
//! disclosure (`ShellState::desktop_settings_open`, `mobile_settings_open`):
//! they are two different elements with two different ids, so one is never
//! mistaken for the other's outside click. Escape — which already closes
//! the rail and the mobile menu — closes both too (each through its own
//! listener: the [`Menu`]'s own popover for the desktop one; the mobile row
//! is simply part of the mobile menu it closes).

use leptos::prelude::*;

use crate::button::{Button, Disclosed, Disclosure, DisclosureButton, Size, Variant};
use crate::icon::{Icon, IconName};
use crate::menu::{DrillMenu, Menu, MenuItem};
use crate::nav::{MobileSections, NavList, RailSections};
use crate::persisted;
use crate::theme::{self, Theme};

/// Rail widths, collapsed and open.
const RAIL_CLOSED: &str = "4.5rem";
const RAIL_OPEN: &str = "15rem";

/// Shared state of the shell, provided as context: any component below the
/// shell reads it and follows its changes.
#[derive(Clone, Copy, Debug)]
pub struct ShellState {
    /// The desktop rail is open (remembered in the `pc_rail` cookie).
    pub rail_open: RwSignal<bool>,
    /// The mobile full-screen navigation is open (never remembered).
    pub menu_open: RwSignal<bool>,
    /// The desktop Settings popover is open (never remembered). Distinct
    /// from `mobile_settings_open`: see the module documentation.
    pub desktop_settings_open: RwSignal<bool>,
    /// The mobile Settings row is expanded (never remembered).
    pub mobile_settings_open: RwSignal<bool>,
    /// The reading theme (remembered in the `pc_theme` cookie).
    pub theme: RwSignal<Theme>,
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
/// (the service serves it; `/lotus.png` by default).
#[component]
pub fn AppShell(
    /// URL of the lotus image, served by the service.
    #[prop(default = "/lotus.png")]
    logo: &'static str,
    /// The app's navigation, below the lotus; none by default.
    #[prop(optional)]
    nav: Option<NavList>,
    /// Settings, at the bottom of the rail and of the mobile menu:
    /// [`theme::theme_menu`] alone when not given; an app overriding it
    /// that still wants the theme choice adds that itself. An empty `Vec`
    /// hides the Settings entry.
    #[prop(optional)]
    settings: Option<Vec<MenuItem>>,
    /// The service's page.
    children: Children,
) -> impl IntoView {
    let state = ShellState {
        rail_open: persisted("rail", false),
        menu_open: RwSignal::new(false),
        desktop_settings_open: RwSignal::new(false),
        mobile_settings_open: RwSignal::new(false),
        theme: theme::theme_signal(),
    };
    provide_context(state);
    close_on_escape(state);
    apply_theme(state.theme);
    let rail_open = state.rail_open;
    let settings = settings.unwrap_or_else(|| vec![theme::theme_menu()]);

    let rail_width = move || {
        if rail_open.get() {
            RAIL_OPEN
        } else {
            RAIL_CLOSED
        }
    };
    view! {
        <DesktopRail logo=logo nav=nav.clone() settings=settings.clone() />
        <MobileNav logo=logo nav=nav settings=settings />
        // The service's page, beside the rail on desktop.
        <div class="transition-[padding] duration-200 sm:pl-(--pc-rail)" style:--pc-rail=rail_width>
            {children()}
        </div>
    }
}

/// Escape closes the rail, the mobile navigation, and the mobile Settings
/// row (the desktop one closes itself: its [`Menu`] is a
/// [`crate::popover::Popover`], which
/// already listens for Escape). The browser only: there is no window while
/// the server renders.
fn close_on_escape(state: ShellState) {
    #[cfg(feature = "hydrate")]
    let _ = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            state.menu_open.set(false);
            state.rail_open.set(false);
            state.mobile_settings_open.set(false);
        }
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = state;
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

const OPEN_LABEL: &str = "Open navigation";
const CLOSE_LABEL: &str = "Close navigation";

/// The desktop rail: the lotus opens and closes it; the name shows when
/// open. The Settings popover is a sibling, not a child: the rail clips its
/// own overflow (`overflow-hidden`, for the width transition), which would
/// clip the popover too. Nothing of Settings renders when `settings` is
/// empty.
#[component]
fn DesktopRail(logo: &'static str, nav: Option<NavList>, settings: Vec<MenuItem>) -> impl IntoView {
    let ShellState { rail_open, desktop_settings_open, .. } = expect_context::<ShellState>();
    let disclosure = Disclosed::new("pc-settings", desktop_settings_open);
    let has_settings = !settings.is_empty();
    view! {
        <nav
            id="pc-rail"
            aria-label="PaliCanon"
            class="fixed inset-y-0 left-0 z-30 hidden flex-col overflow-hidden border-r border-line bg-page transition-[width] duration-200 sm:flex"
            style:width=move || if rail_open.get() { RAIL_OPEN } else { RAIL_CLOSED }
            data-open=move || rail_open.get().to_string()
        >
            <div class="flex items-center gap-3 p-3.5">
                <DisclosureButton
                    controls=Disclosed::new("pc-rail", rail_open)
                    variant=Variant::Icon
                    label_closed=OPEN_LABEL
                    label_open=CLOSE_LABEL
                >
                    <Lotus logo=logo />
                </DisclosureButton>
                <Wordmark shown=rail_open />
            </div>
            <div class="flex flex-col gap-1 overflow-y-auto px-3.5 pb-3.5">
                {nav.map(|nav| view! { <RailSections nav=nav open=rail_open /> })}
            </div>
            {has_settings.then(|| view! { <RailSettingsEntry settings=disclosure shown=rail_open /> })}
        </nav>
        {has_settings
            .then(|| view! { <SettingsMenu controls=disclosure items=settings rail_open=rail_open /> })}
    }
}

/// The Settings entry, pinned to the bottom of the rail by `mt-auto` (the
/// rail is a flex column): a gear, and the label, fading like the nav
/// entries above it while the rail is collapsed.
#[component]
fn RailSettingsEntry(settings: Disclosed, #[prop(into)] shown: Signal<bool>) -> impl IntoView {
    view! {
        <div class="mt-auto border-t border-line px-3.5 pt-3.5 pb-3.5">
            <DisclosureButton controls=settings variant=Variant::Nav label_closed="Settings" label_open="Settings">
                <Icon name=IconName::Gear />
                <span class="transition-opacity duration-150" class:opacity-0=move || !shown.get()>
                    "Settings"
                </span>
            </DisclosureButton>
        </div>
    }
}

/// The [`Menu`] Settings opens on desktop: fixed beside the rail, outside
/// its clipped `<nav>`, its left edge following the rail's width so it
/// stays flush whether the rail is open or collapsed.
#[component]
fn SettingsMenu(
    controls: Disclosed,
    items: Vec<MenuItem>,
    #[prop(into)] rail_open: Signal<bool>,
) -> impl IntoView {
    let left = move || {
        let width = if rail_open.get() {
            RAIL_OPEN
        } else {
            RAIL_CLOSED
        };
        format!("calc({width} + 0.5rem)")
    };
    view! {
        <div class="fixed bottom-3.5 z-30 hidden sm:block" style:left=left>
            <Menu controls=controls aria_label="Settings" items=items />
        </div>
    }
}

/// The mobile top bar, and the full-screen navigation its lotus opens.
#[component]
fn MobileNav(logo: &'static str, nav: Option<NavList>, settings: Vec<MenuItem>) -> impl IntoView {
    let ShellState { menu_open, mobile_settings_open, .. } = expect_context::<ShellState>();
    let menu = Disclosed::new("pc-menu", menu_open);
    let settings_disclosure = Disclosed::new("pc-settings-menu", mobile_settings_open);
    let has_settings = !settings.is_empty();
    view! {
        <header class="sticky top-0 z-30 flex h-14 items-center gap-2 border-b border-line bg-page px-2 sm:hidden">
            <DisclosureButton controls=menu variant=Variant::Icon label_closed=OPEN_LABEL label_open=CLOSE_LABEL>
                <Lotus logo=logo />
            </DisclosureButton>
            <Wordmark shown=Signal::derive(|| true) />
        </header>
        <div
            id="pc-menu"
            role="dialog"
            aria-modal="true"
            aria-label="PaliCanon"
            class="fixed inset-0 z-40 flex flex-col bg-page sm:hidden"
            hidden=move || !menu_open.get()
        >
            <MobileMenuHeader logo=logo menu=menu />
            // Clicks bubble up from the entries: choosing one closes the menu.
            // Settings sits outside this container: choosing a theme must
            // not close the menu the same way.
            <div class="flex flex-col gap-1 overflow-y-auto p-3" on:click=move |_| menu.set_open(false)>
                {nav.map(|nav| view! { <MobileSections nav=nav /> })}
            </div>
            {has_settings
                .then(|| view! { <MobileSettings settings=settings_disclosure items=settings /> })}
        </div>
    }
}

/// The Settings row of the full-screen navigation, pinned to the bottom;
/// a [`DrillMenu`] of `items` expands inline below it, never closing the
/// mobile menu.
#[component]
fn MobileSettings(settings: Disclosed, items: Vec<MenuItem>) -> impl IntoView {
    view! {
        <div class="mt-auto border-t border-line p-3">
            <DisclosureButton controls=settings variant=Variant::Nav size=Size::Lg label_closed="Settings" label_open="Settings">
                <Icon name=IconName::Gear />
                <span>"Settings"</span>
            </DisclosureButton>
            <div id="pc-settings-menu" class="mt-1 pl-1" hidden=move || !settings.is_open()>
                <DrillMenu items=items />
            </div>
        </div>
    }
}

/// The top of the full-screen navigation: the lotus (closes it), the name,
/// and a close cross.
#[component]
fn MobileMenuHeader(logo: &'static str, menu: Disclosed) -> impl IntoView {
    view! {
        <div class="flex h-14 items-center gap-2 border-b border-line px-2">
            <DisclosureButton controls=menu variant=Variant::Icon label_closed=OPEN_LABEL label_open=CLOSE_LABEL>
                <Lotus logo=logo />
            </DisclosureButton>
            <Wordmark shown=Signal::derive(|| true) />
            <Button variant=Variant::Icon label=CLOSE_LABEL layout="ml-auto" on_click=move |_| menu.set_open(false)>
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                    <path d="m4 4 8 8M12 4l-8 8" />
                </svg>
            </Button>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn render(rail_cookie: Option<&str>) -> String {
        let owner = Owner::new();
        owner.with(|| {
            if let Some(value) = rail_cookie {
                let request = http::Request::builder()
                    .header(http::header::COOKIE, format!("pc_rail={value}"))
                    .body(())
                    .unwrap();
                provide_context(request.into_parts().0);
            }
            view! { <AppShell><p>"page"</p></AppShell> }.to_html()
        })
    }

    #[test]
    fn closed_by_default_with_the_lotus_only() {
        let html = render(None);
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains(r#"aria-label="Open navigation""#), "{html}");
        assert!(html.contains(r#"data-open="false""#), "{html}");
        assert!(html.contains("4.5rem"), "{html}");
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
        assert!(html.contains("15rem"), "{html}");
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
            view! { <AppShell nav=nav><p>"page"</p></AppShell> }.to_html()
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
    fn desktop_and_mobile_settings_are_distinct_disclosures() {
        // The bug this regression-tests: both used to share one signal
        // under two different ids, so the desktop popover's outside-click
        // listener (always mounted, only hidden on mobile by CSS) saw the
        // mobile button as "outside" and closed what it had just opened.
        let html = render(None);
        assert!(html.contains(r#"aria-controls="pc-settings""#), "{html}");
        assert!(html.contains(r#"aria-controls="pc-settings-menu""#), "{html}");
        assert!(html.contains(r#"id="pc-settings""#), "{html}");
        assert!(html.contains(r#"id="pc-settings-menu""#), "{html}");
    }

    #[test]
    fn settings_default_to_the_theme_menu() {
        let html = render(None);
        assert!(html.contains("Theme"), "{html}");
        assert!(html.contains("System"), "{html}");
        assert!(html.contains("Light"), "{html}");
        assert!(html.contains("Dark"), "{html}");
    }

    #[test]
    fn an_empty_settings_list_hides_the_entry() {
        let html = Owner::new()
            .with(|| view! { <AppShell settings=Vec::new()><p>"page"</p></AppShell> }.to_html());
        assert!(!html.contains("Settings"), "{html}");
        assert!(!html.contains(r#"id="pc-settings""#), "{html}");
        assert!(!html.contains(r#"id="pc-settings-menu""#), "{html}");
    }

    #[test]
    fn an_app_can_override_settings_with_its_own_items() {
        let html = Owner::new().with(|| {
            let items = vec![MenuItem::Action {
                label: "Say hello",
                icon: None,
                on_select: Callback::new(|()| {}),
            }];
            view! { <AppShell settings=items><p>"page"</p></AppShell> }.to_html()
        });
        assert!(html.contains("Say hello"), "{html}");
        assert!(!html.contains("Theme"), "not kept unless the app adds it back: {html}");
    }
}
