//! A value that survives page loads and is shared by every PaliCanon
//! subdomain: a signal mirrored into a cookie.
//!
//! The cookie, not browser storage, because the server renders the first
//! page: it reads the cookie from the request, so the page arrives already
//! in the right state (no flash), and the browser starts from the same value
//! when it hydrates. On `*.palicanon.app` the cookie is set on the parent
//! domain, so `palicanon.app` and `ai.palicanon.app` share it.

use std::str::FromStr;

use leptos::prelude::*;

/// Cookie names are prefixed so they never collide with anyone else's.
pub const PREFIX: &str = "pc_";

/// A year: interface preferences are kept, not session-long.
const MAX_AGE_SECONDS: u32 = 365 * 24 * 3600;

/// The value of cookie `name` in a `Cookie:` header (or `document.cookie`),
/// if present.
#[must_use]
pub fn cookie_value<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header.split(';').find_map(|pair| {
        let (k, v) = pair.trim().split_once('=')?;
        (k == name).then_some(v)
    })
}

/// The `document.cookie` assignment that stores `value` under `name`:
/// a year, the whole site, sent on same-site navigation, and on the parent
/// domain when `host` is a PaliCanon one (so every service sees it).
#[must_use]
pub fn cookie_assignment(name: &str, value: &str, host: &str) -> String {
    let domain = if host == "palicanon.app" || host.ends_with(".palicanon.app") {
        "; Domain=palicanon.app"
    } else {
        ""
    };
    format!("{name}={value}; Path=/; Max-Age={MAX_AGE_SECONDS}; SameSite=Lax{domain}")
}

/// The cookie header of the request being rendered, on the server. Also
/// used by [`crate::theme::theme_attribute`] to read `pc_theme` for the
/// first render.
#[cfg(feature = "ssr")]
pub(crate) fn request_cookies() -> Option<String> {
    let parts = use_context::<http::request::Parts>()?;
    let header = parts.headers.get(http::header::COOKIE)?;
    header.to_str().ok().map(str::to_owned)
}

/// `document.cookie`, in the browser.
#[cfg(feature = "hydrate")]
pub(crate) fn request_cookies() -> Option<String> {
    use wasm_bindgen::JsCast;
    let document = web_sys::window()?.document()?;
    document
        .dyn_into::<web_sys::HtmlDocument>()
        .ok()?
        .cookie()
        .ok()
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub(crate) fn request_cookies() -> Option<String> {
    None
}

#[cfg(feature = "hydrate")]
fn write_cookie(assignment: &str) {
    use wasm_bindgen::JsCast;
    if let Some(document) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
    {
        let _ = document.set_cookie(assignment);
    }
}

#[cfg(feature = "hydrate")]
fn current_host() -> String {
    web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .unwrap_or_default()
}

/// A signal holding the value stored under `pc_<name>`, or `default` when
/// there is none (or it does not parse). Changes are written back to the
/// cookie in the browser.
pub fn persisted<T>(name: &'static str, default: T) -> RwSignal<T>
where
    T: FromStr + ToString + Clone + Send + Sync + 'static,
{
    let key = format!("{PREFIX}{name}");
    let initial = request_cookies()
        .as_deref()
        .and_then(|cookies| cookie_value(cookies, &key))
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(default);
    let signal = RwSignal::new(initial);
    #[cfg(feature = "hydrate")]
    Effect::new(move |previous: Option<()>| {
        let value = signal.get().to_string();
        // The first run only reads: the cookie already holds this value.
        if previous.is_some() {
            write_cookie(&cookie_assignment(&key, &value, &current_host()));
        }
    });
    signal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_cookie_among_others() {
        let header = "a=1; pc_rail=true;pc_theme=sepia";
        assert_eq!(cookie_value(header, "pc_rail"), Some("true"));
        assert_eq!(cookie_value(header, "pc_theme"), Some("sepia"));
        assert_eq!(cookie_value(header, "a"), Some("1"));
        assert_eq!(cookie_value(header, "pc_lang"), None);
        assert_eq!(cookie_value("", "pc_rail"), None);
        assert_eq!(
            cookie_value("pc_rail", "pc_rail"),
            None,
            "no '=' is no value"
        );
    }

    #[test]
    fn a_prefix_is_not_a_match() {
        assert_eq!(
            cookie_value("pc_rail_old=x; pc_rail=false", "pc_rail"),
            Some("false")
        );
    }

    #[test]
    fn the_cookie_is_shared_by_palicanon_subdomains_only() {
        let shared = cookie_assignment("pc_rail", "true", "ai.palicanon.app");
        assert!(shared.ends_with("; Domain=palicanon.app"), "{shared}");
        assert!(
            cookie_assignment("pc_rail", "true", "palicanon.app").contains("Domain=palicanon.app")
        );
        let local = cookie_assignment("pc_rail", "true", "127.0.0.1");
        assert!(!local.contains("Domain"), "{local}");
        assert!(!cookie_assignment("x", "1", "evilpalicanon.app").contains("Domain"));
        assert!(local.starts_with("pc_rail=true; Path=/; Max-Age="));
        assert!(local.contains("SameSite=Lax"));
    }

    #[test]
    fn a_missing_or_unparsable_value_falls_back_to_the_default() {
        // No request in context (a plain test): the default.
        let owner = Owner::new();
        owner.with(|| {
            assert!(persisted("rail", true).get_untracked());
            assert_eq!(persisted("count", 3_u32).get_untracked(), 3);
        });
    }
}
