//! The app shell every PaliCanon service shares.
//!
//! - **Desktop** (≥ 640 px): a navigation rail on the left. Collapsed, it
//!   shows the lotus only; the lotus opens it (the name appears) and closes
//!   it. The page beside it moves with it. Open or closed is remembered
//!   ([`persisted`], cookie `pc_rail`), across pages and subdomains.
//! - **Mobile**: a top bar with the lotus; the lotus opens the navigation
//!   full screen, the lotus or ✕ closes it. Always closed when a page loads.
//! - **Escape** closes whichever is open.
//!
//! The rail holds no service yet: each service adds its own entry when it is
//! created. Strings are English until the i18n mechanism is defined.

use leptos::prelude::*;

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

/// The lotus mark and the name, as the button content.
#[component]
fn Mark(logo: &'static str, #[prop(into)] show_name: Signal<bool>) -> impl IntoView {
    view! {
        <img src=logo alt="" width="32" height="32" class="size-8 shrink-0" />
        <span
            class="font-serif text-lg font-semibold whitespace-nowrap text-heading transition-opacity duration-150"
            class:opacity-0=move || !show_name.get()
            aria-hidden=move || (!show_name.get()).to_string()
        >
            "PaliCanon"
        </span>
    }
}

/// The shell around a service's page. `logo` is the lotus image's URL
/// (the service serves it; `/lotus.png` by default).
#[component]
pub fn AppShell(
    #[prop(default = "/lotus.png")] logo: &'static str,
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

/// "Close navigation" or "Open navigation", following `open`.
fn toggle_label(open: RwSignal<bool>) -> impl Fn() -> &'static str + Copy {
    move || {
        if open.get() {
            "Close navigation"
        } else {
            "Open navigation"
        }
    }
}

/// The desktop rail: the lotus opens and closes it.
#[component]
fn DesktopRail(logo: &'static str) -> impl IntoView {
    let ShellState { rail_open, .. } = expect_context::<ShellState>();
    let label = toggle_label(rail_open);
    view! {
        <nav
            id="pc-rail"
            aria-label="PaliCanon"
            class="fixed inset-y-0 left-0 z-30 hidden flex-col overflow-hidden border-r border-line bg-page transition-[width] duration-200 sm:flex"
            style:width=move || if rail_open.get() { RAIL_OPEN } else { RAIL_CLOSED }
            data-open=move || rail_open.get().to_string()
        >
            <button
                type="button"
                class="m-3 flex h-12 cursor-pointer items-center gap-3 rounded-xl px-2 text-left transition hover:bg-chip"
                aria-controls="pc-rail"
                aria-expanded=move || rail_open.get().to_string()
                aria-label=label
                title=label
                on:click=move |_| rail_open.update(|open| *open = !*open)
            >
                <Mark logo=logo show_name=rail_open />
            </button>
        </nav>
    }
}

/// The mobile top bar, and the full-screen navigation its lotus opens.
#[component]
fn MobileNav(logo: &'static str) -> impl IntoView {
    let ShellState { menu_open, .. } = expect_context::<ShellState>();
    view! {
        <header class="sticky top-0 z-30 flex h-14 items-center border-b border-line bg-page px-3 sm:hidden">
            <button
                type="button"
                class="flex h-11 cursor-pointer items-center gap-3 rounded-xl px-2 transition hover:bg-chip"
                aria-controls="pc-menu"
                aria-expanded=move || menu_open.get().to_string()
                aria-label=toggle_label(menu_open)
                on:click=move |_| menu_open.update(|open| *open = !*open)
            >
                <Mark logo=logo show_name=Signal::derive(|| true) />
            </button>
        </header>
        <div
            id="pc-menu"
            role="dialog"
            aria-modal="true"
            aria-label="PaliCanon"
            class="fixed inset-0 z-40 flex flex-col bg-page sm:hidden"
            hidden=move || !menu_open.get()
        >
            <div class="flex h-14 items-center justify-between border-b border-line px-3">
                <button
                    type="button"
                    class="flex h-11 cursor-pointer items-center gap-3 rounded-xl px-2 transition hover:bg-chip"
                    aria-label="Close navigation"
                    on:click=move |_| menu_open.set(false)
                >
                    <Mark logo=logo show_name=Signal::derive(|| true) />
                </button>
                <button
                    type="button"
                    class="grid size-11 cursor-pointer place-items-center rounded-full text-muted transition hover:bg-chip hover:text-heading"
                    aria-label="Close navigation"
                    on:click=move |_| menu_open.set(false)
                >
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                        <path d="m4 4 8 8M12 4l-8 8" />
                    </svg>
                </button>
            </div>
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
