//! Hydration that starts at the app's own root element, so that nodes added
//! to `<body>` by someone else than the server are never visited.
//!
//! Behind Cloudflare, the served HTML is not exactly what the server
//! rendered: the zone puts a hidden decoy link to `/cdn-cgi/content` (an
//! AI-crawler trap) as the FIRST child of `<body>`, and appends the Web
//! Analytics beacon and challenge scripts after the app. Leptos's
//! `hydrate_body` walks `<body>`'s children in render order, meets the
//! decoy where it expects the app's first node, and panics (task 063).
//!
//! The fix wraps the app in one element the server writes and the browser
//! looks up by id:
//!
//! - the server's `shell()` renders `<body><HydrationRoot><App/></HydrationRoot></body>`
//!   ([`HydrationRoot`], `<div id="pc-app">`);
//! - the browser's entry calls `hydrate(App)` instead of
//!   `leptos::mount::hydrate_body(App)`: it hydrates from that element, so
//!   whatever sits before or after it in `<body>` is left alone — nothing is
//!   removed or moved, and no list of foreign nodes is kept;
//! - once hydrated, `<html>` carries `data-hydrated`, which a site's
//!   rollout check reads in the dumped DOM.
//!
//! The two changes go together: a site whose shell has no root fails at
//! once, on localhost too ([`MissingRoot`]), with no fallback to `<body>`.
//! What this cannot fix: a node changed INSIDE the root (Email Obfuscation
//! rewriting an address in the app's markup) still breaks hydration.

use std::fmt;

use leptos::prelude::*;

/// The id of the app's root element: the one name the server writes
/// ([`HydrationRoot`]) and the browser looks up (`hydrate`).
pub const APP_ROOT_ID: &str = "pc-app";

/// The app's root element, `<div id="pc-app">` around `children`: what
/// `hydrate` starts from. The server's `shell()` puts it as the one
/// child of `<body>` the app renders: `<body><HydrationRoot><App/></HydrationRoot></body>`.
/// It has no class and no style (a block in `<body>`'s flow), so the layout
/// is unchanged. It compiles in every build, since a site's `shell()` is
/// built for the browser too; only the server renders it.
#[component]
pub fn HydrationRoot(
    /// The app.
    children: Children,
) -> impl IntoView {
    view! { <div id=APP_ROOT_ID>{children()}</div> }
}

/// The page has no element `#pc-app`: the server's shell does not wrap the
/// app in [`HydrationRoot`]. `app_root` returns it; `hydrate` panics
/// with its message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingRoot;

impl fmt::Display for MissingRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "no element #{APP_ROOT_ID} in the page: wrap <App/> in HydrationRoot in the server's shell() \
             (<body><HydrationRoot><App/></HydrationRoot></body>)"
        )
    }
}

impl std::error::Error for MissingRoot {}

/// The app's root element in `document`: the HTML element whose id is
/// [`APP_ROOT_ID`], or [`MissingRoot`] when there is none.
///
/// # Errors
///
/// [`MissingRoot`] when `document` has no HTML element `#pc-app`.
#[cfg(feature = "hydrate")]
pub fn app_root(document: &web_sys::Document) -> Result<web_sys::HtmlElement, MissingRoot> {
    use wasm_bindgen::JsCast;

    document
        .get_element_by_id(APP_ROOT_ID)
        .and_then(|root| root.dyn_into::<web_sys::HtmlElement>().ok())
        .ok_or(MissingRoot)
}

/// Hydrates the app `f` from its root element ([`HydrationRoot`]) instead
/// of `<body>`, then marks `<html>` with `data-hydrated`. What a site's
/// browser entry calls in place of `leptos::mount::hydrate_body(App)`.
/// The app lives as long as the page (its owner is leaked, as
/// `hydrate_body` does).
///
/// # Panics
///
/// When the page has no root: the server's shell does not wrap the app in
/// [`HydrationRoot`] (the message is [`MissingRoot`]'s). There is no
/// fallback to `<body>`.
#[cfg(feature = "hydrate")]
pub fn hydrate<F, N>(f: F)
where
    F: FnOnce() -> N + 'static,
    N: IntoView,
{
    let document = document();
    let root = app_root(&document).unwrap_or_else(|missing| panic!("{missing}"));
    leptos::mount::hydrate_from(root, f).forget();
    if let Some(html) = document.document_element() {
        // Setting an attribute on <html> only fails on an invalid name.
        _ = html.set_attribute("data-hydrated", "");
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn the_root_id_is_pc_app() {
        assert_eq!(APP_ROOT_ID, "pc-app");
    }

    #[test]
    fn the_root_wraps_its_children_in_a_bare_div_named_by_the_root_id() {
        let html = Owner::new().with(|| {
            view! {
                <HydrationRoot>
                    <p>"app"</p>
                </HydrationRoot>
            }
            .to_html()
        });
        assert_eq!(html, r#"<div id="pc-app"><p>app</p></div>"#);
    }

    #[test]
    fn a_missing_root_names_the_root_and_the_fix() {
        let message = MissingRoot.to_string();
        assert!(message.contains("#pc-app"), "{message}");
        assert!(message.contains("HydrationRoot"), "{message}");
        assert!(message.contains("shell"), "{message}");
    }
}
