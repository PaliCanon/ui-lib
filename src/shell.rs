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
//! The rail holds no service yet: each service adds its own entry when it is
//! created. Strings are English until the i18n mechanism is defined.

use leptos::prelude::*;

use crate::button::{Button, Disclosed, Disclosure, DisclosureButton, Variant};
use crate::persisted;

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
    /// The service's page.
    children: Children,
) -> impl IntoView {
    let state = ShellState {
        rail_open: persisted("rail", false),
        menu_open: RwSignal::new(false),
    };
    provide_context(state);
    close_on_escape(state);
    let rail_open = state.rail_open;

    let rail_width = move || {
        if rail_open.get() {
            RAIL_OPEN
        } else {
            RAIL_CLOSED
        }
    };
    view! {
        <DesktopRail logo=logo />
        <MobileNav logo=logo />
        // The service's page, beside the rail on desktop.
        <div class="transition-[padding] duration-200 sm:pl-(--pc-rail)" style:--pc-rail=rail_width>
            {children()}
        </div>
    }
}

/// Escape closes the rail and the mobile navigation. The browser only:
/// there is no window while the server renders.
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

const OPEN_LABEL: &str = "Open navigation";
const CLOSE_LABEL: &str = "Close navigation";

/// The desktop rail: the lotus opens and closes it; the name shows when open.
#[component]
fn DesktopRail(logo: &'static str) -> impl IntoView {
    let ShellState { rail_open, .. } = expect_context::<ShellState>();
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
        </nav>
    }
}

/// The mobile top bar, and the full-screen navigation its lotus opens.
#[component]
fn MobileNav(logo: &'static str) -> impl IntoView {
    let ShellState { menu_open, .. } = expect_context::<ShellState>();
    let menu = Disclosed::new("pc-menu", menu_open);
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
    fn a_garbled_cookie_falls_back_to_closed() {
        let html = render(Some("perhaps"));
        assert!(html.contains(r#"data-open="false""#), "{html}");
    }
}
