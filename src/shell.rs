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
//! full-screen navigation (mobile): both the gear (desktop) and the row
//! (mobile) open the very same [`crate::dialog::Dialog`], titled
//! "Settings", through one shared disclosure
//! (`ShellState::settings_open`) — only one of the two triggers is ever
//! visible at a time (the rail and the mobile navigation already are), so
//! sharing an id never confuses an outside click the way two overlapping
//! popovers once could. Its content is `settings`: one
//! [`crate::dialog::DialogSection`] "Appearance" holding the theme as a
//! [`crate::button::Segmented`] (Light · Dark · System) by default; an app
//! overriding `settings` replaces that content (it adds the Appearance
//! section back itself to keep the theme choice). Escape and the backdrop
//! click, already wired into `Dialog`, close it like anything else it
//! closes.

use leptos::prelude::*;

use crate::button::{
    Button, Disclosed, Disclosure, DisclosureButton, Segmented, SegmentedOption, Size, Variant,
};
use crate::dialog::{Dialog, DialogSection};
use crate::icon::{Icon, IconName};
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
    /// The Settings dialog is open (never remembered): shared by the
    /// desktop gear and the mobile row, see the module documentation.
    pub settings_open: RwSignal<bool>,
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
    /// The Settings dialog's content: one Appearance section with the
    /// theme as a [`Segmented`] when not given; an app overriding it that
    /// still wants the theme choice adds that section back itself.
    #[prop(optional, into)]
    settings: Option<ViewFn>,
    /// The service's page.
    children: Children,
) -> impl IntoView {
    let state = ShellState {
        rail_open: persisted("rail", false),
        menu_open: RwSignal::new(false),
        settings_open: RwSignal::new(false),
        theme: theme::theme_signal(),
    };
    provide_context(state);
    close_on_escape(state);
    apply_theme(state.theme);
    let rail_open = state.rail_open;
    let settings = settings.unwrap_or_else(|| default_settings(state.theme));

    let rail_width = move || {
        if rail_open.get() {
            RAIL_OPEN
        } else {
            RAIL_CLOSED
        }
    };
    view! {
        <DesktopRail logo=logo nav=nav.clone() />
        <MobileNav logo=logo nav=nav />
        <SettingsDialog settings=settings />
        // The service's page, beside the rail on desktop.
        <div class="transition-[padding] duration-200 sm:pl-(--pc-rail)" style:--pc-rail=rail_width>
            {children()}
        </div>
    }
}

/// The default of `AppShell`'s `settings`: one Appearance section, the
/// theme as a [`Segmented`] of Light, Dark and System, bound to `theme`.
fn default_settings(theme: RwSignal<Theme>) -> ViewFn {
    (move || {
        let options = vec![
            SegmentedOption {
                label: "Light",
                value: Theme::Light,
            },
            SegmentedOption {
                label: "Dark",
                value: Theme::Dark,
            },
            SegmentedOption {
                label: "System",
                value: Theme::System,
            },
        ];
        view! {
            <DialogSection title="Appearance">
                <Segmented
                    aria_label="Theme"
                    options=options
                    value=Signal::derive(move || theme.get())
                    on_change=Callback::new(move |value| theme.set(value))
                />
            </DialogSection>
        }
        .into_any()
    })
    .into()
}

/// The Settings dialog, opened by both the desktop gear and the mobile row
/// through the same shared `ShellState::settings_open`.
#[component]
fn SettingsDialog(settings: ViewFn) -> impl IntoView {
    let state = expect_context::<ShellState>();
    let controls = Disclosed::new("pc-settings", state.settings_open);
    view! { <Dialog controls=controls title="Settings">{settings.run()}</Dialog> }
}

/// Escape closes the rail and the mobile navigation (the Settings dialog
/// closes itself: it is a [`crate::dialog::Dialog`], which already listens
/// for Escape). The browser only: there is no window while the server
/// renders.
fn close_on_escape(state: ShellState) {
    #[cfg(feature = "hydrate")]
    let _ = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            state.menu_open.set(false);
            state.rail_open.set(false);
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
/// open. The Settings entry at its bottom opens [`SettingsDialog`], a
/// sibling of the rail (not nested in it): the dialog is fixed over the
/// whole page, not clipped by the rail's own `overflow-hidden`.
#[component]
fn DesktopRail(logo: &'static str, nav: Option<NavList>) -> impl IntoView {
    let ShellState {
        rail_open,
        settings_open,
        ..
    } = expect_context::<ShellState>();
    let settings = Disclosed::new("pc-settings", settings_open);
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
            <RailSettingsEntry settings=settings shown=rail_open />
        </nav>
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

/// The mobile top bar, and the full-screen navigation its lotus opens.
#[component]
fn MobileNav(logo: &'static str, nav: Option<NavList>) -> impl IntoView {
    let ShellState {
        menu_open,
        settings_open,
        ..
    } = expect_context::<ShellState>();
    let menu = Disclosed::new("pc-menu", menu_open);
    let settings = Disclosed::new("pc-settings", settings_open);
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
            // Settings sits outside this container: opening it must not
            // close the mobile menu the same way.
            <div class="flex flex-col gap-1 overflow-y-auto p-3" on:click=move |_| menu.set_open(false)>
                {nav.map(|nav| view! { <MobileSections nav=nav /> })}
            </div>
            <MobileSettings settings=settings />
        </div>
    }
}

/// The Settings row of the full-screen navigation, pinned to the bottom:
/// opens the very same [`SettingsDialog`] the desktop gear does.
#[component]
fn MobileSettings(settings: Disclosed) -> impl IntoView {
    view! {
        <div class="mt-auto border-t border-line p-3">
            <DisclosureButton controls=settings variant=Variant::Nav size=Size::Lg label_closed="Settings" label_open="Settings">
                <Icon name=IconName::Gear />
                <span>"Settings"</span>
            </DisclosureButton>
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
    fn settings_default_to_appearance_with_a_segmented_theme() {
        let html = render(None);
        assert!(html.contains("Appearance"), "{html}");
        assert!(html.contains(r#"role="radiogroup""#), "{html}");
        assert!(html.contains(r#"aria-label="Theme""#), "{html}");
        assert!(html.contains("Light"), "{html}");
        assert!(html.contains("Dark"), "{html}");
        assert!(html.contains("System"), "{html}");
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
}
