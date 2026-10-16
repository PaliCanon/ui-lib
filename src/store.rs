//! Where a preference is kept: the [`Store`].
//!
//! A preference (theme, language, a rail's state) is persisted under a
//! `pc_<name>` key. **Which medium holds it** is a value, not a build mode:
//!
//! - [`Store::Cookie`], the default: the `pc_*` cookie. On the server it is
//!   the request's `Cookie` header, in the browser `document.cookie`, shared
//!   by every `*.palicanon.app` service. The web sites never provide a
//!   store, so they keep this behaviour unchanged.
//! - [`Store::Local`]: `localStorage` under the same keys, for a page with
//!   no server and no cookies (the app's webview). `csr::mount`
//!   provides it through Leptos context.
//!
//! [`crate::persisted::persisted`], [`crate::persisted::persisted_with`],
//! [`crate::theme::theme_attribute`] and the language all read and write
//! through [`Store::current`], and nothing else touches `document.cookie`
//! (a test greps for it).
//!
//! A `Local` read never fails the page: storage that is absent, that throws
//! (blocked, private mode) or that holds a garbled value reads as "no value",
//! and the default applies. A `Local` write returns a [`Result`]; the signal
//! that made it keeps its value in memory, and one console warning says the
//! choice will not survive the page.

use std::fmt;
use std::str::FromStr;
#[cfg(feature = "browser")]
use std::sync::atomic::{AtomicBool, Ordering};

use leptos::prelude::*;

/// Where preferences are kept. See the [module](self) documentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Store {
    /// The `pc_*` cookie: the request's on the server, `document.cookie` in
    /// the browser. What a page gets when nothing provided a store.
    #[default]
    Cookie,
    /// `localStorage`, under the same `pc_*` keys. Provided by
    /// `csr::mount`.
    Local,
}

/// Why a [`Store::write`] did not store the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// There is no storage to write to: no browser, or the browser withholds
    /// it (blocked storage throws on access).
    Unavailable,
    /// The storage exists but refused this value (quota, a security rule).
    Rejected,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StoreError::Unavailable => "browser storage is unavailable",
            StoreError::Rejected => "browser storage refused the value",
        })
    }
}

impl std::error::Error for StoreError {}

impl Store {
    /// The store of the current Leptos owner tree: the one provided in
    /// context, else [`Store::Cookie`] (the web's behaviour).
    #[must_use]
    pub fn current() -> Store {
        use_context::<Store>().unwrap_or_default()
    }

    /// The value stored under `key` (the full key, `pc_theme`), or `None`:
    /// absent, unreadable, or (for [`Store::Local`]) storage that throws.
    #[must_use]
    pub fn read(self, key: &str) -> Option<String> {
        match self {
            Store::Cookie => cookie::read(key),
            Store::Local => local::read(key),
        }
    }

    /// Whether a failed [`Store::write`] earns the console warning: only
    /// [`Store::Local`]'s does. A failed cookie write keeps the web's
    /// behaviour, the result ignored and no warning.
    #[cfg(any(feature = "browser", test))]
    pub(crate) fn warns_on_failed_write(self) -> bool {
        matches!(self, Store::Local)
    }

    /// Stores `value` under `key` (the full key, `pc_theme`). A cookie write
    /// on the server is a no-op: the browser writes it.
    ///
    /// # Errors
    ///
    /// [`StoreError::Unavailable`] when there is no storage to write to,
    /// [`StoreError::Rejected`] when it refused the value.
    pub fn write(self, key: &str, value: &str) -> Result<(), StoreError> {
        match self {
            Store::Cookie => cookie::write(key, value),
            Store::Local => local::write(key, value),
        }
    }
}

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

/// `raw` parsed as a `T`; `None` when absent or garbled (the default then
/// applies).
pub(crate) fn parse_stored<T: FromStr>(raw: Option<&str>) -> Option<T> {
    raw?.parse().ok()
}

/// A `localStorage.getItem` outcome as a value: absent and throwing are both
/// "no value".
#[cfg(any(feature = "browser", test))]
pub(crate) fn local_value<E>(result: Result<Option<String>, E>) -> Option<String> {
    result.ok().flatten()
}

/// The one console warning a failed [`Store::write`] earns, then silence:
/// the signal keeps its value in memory, no UI.
#[cfg(feature = "browser")]
pub(crate) fn warn_once(error: StoreError) {
    static WARNED: AtomicBool = AtomicBool::new(false);
    if !WARNED.swap(true, Ordering::Relaxed) {
        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!(
            "palicanon-ui-lib: a preference was not saved ({error}); it is kept for this page only"
        )));
    }
}

/// The cookie store: the only place `document.cookie` is named.
mod cookie {
    use super::{StoreError, cookie_value};

    /// The cookie header of the request being rendered, on the server.
    #[cfg(feature = "ssr")]
    fn request_cookies() -> Option<String> {
        use leptos::prelude::use_context;
        let parts = use_context::<http::request::Parts>()?;
        let header = parts.headers.get(http::header::COOKIE)?;
        header.to_str().ok().map(str::to_owned)
    }

    /// `document.cookie`, in the browser.
    #[cfg(feature = "browser")]
    fn request_cookies() -> Option<String> {
        use wasm_bindgen::JsCast;
        let document = web_sys::window()?.document()?;
        document
            .dyn_into::<web_sys::HtmlDocument>()
            .ok()?
            .cookie()
            .ok()
    }

    #[cfg(not(any(feature = "ssr", feature = "browser")))]
    fn request_cookies() -> Option<String> {
        None
    }

    pub(super) fn read(key: &str) -> Option<String> {
        request_cookies()
            .as_deref()
            .and_then(|cookies| cookie_value(cookies, key))
            .map(str::to_owned)
    }

    #[cfg(feature = "browser")]
    pub(super) fn write(key: &str, value: &str) -> Result<(), StoreError> {
        use wasm_bindgen::JsCast;
        let host = web_sys::window()
            .and_then(|w| w.location().hostname().ok())
            .unwrap_or_default();
        let assignment = super::cookie_assignment(key, value, &host);
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
            .ok_or(StoreError::Unavailable)?
            .set_cookie(&assignment)
            .map_err(|_| StoreError::Rejected)
    }

    /// The server writes no cookie: the browser does, on the first change.
    #[cfg(not(feature = "browser"))]
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn write(_key: &str, _value: &str) -> Result<(), StoreError> {
        Ok(())
    }
}

/// The `localStorage` store.
mod local {
    use super::StoreError;

    /// `window.localStorage`; `None` when there is no window, when it is
    /// null, or when reaching it throws (blocked storage).
    #[cfg(feature = "browser")]
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    #[cfg(feature = "browser")]
    pub(super) fn read(key: &str) -> Option<String> {
        super::local_value(storage()?.get_item(key))
    }

    #[cfg(feature = "browser")]
    pub(super) fn write(key: &str, value: &str) -> Result<(), StoreError> {
        storage()
            .ok_or(StoreError::Unavailable)?
            .set_item(key, value)
            .map_err(|_| StoreError::Rejected)
    }

    #[cfg(not(feature = "browser"))]
    pub(super) fn read(_key: &str) -> Option<String> {
        None
    }

    #[cfg(not(feature = "browser"))]
    pub(super) fn write(_key: &str, _value: &str) -> Result<(), StoreError> {
        Err(StoreError::Unavailable)
    }
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
    fn a_stored_value_parses_or_is_none() {
        assert_eq!(parse_stored::<bool>(Some("true")), Some(true));
        assert_eq!(parse_stored::<u32>(Some("7")), Some(7));
        assert_eq!(parse_stored::<bool>(None), None, "absent");
        for garbled in ["", "maybe", " true", "TRUE"] {
            assert_eq!(parse_stored::<bool>(Some(garbled)), None, "{garbled:?}");
        }
    }

    #[test]
    fn a_local_read_that_throws_or_finds_nothing_is_no_value() {
        assert_eq!(
            local_value::<()>(Ok(Some("x".into()))),
            Some("x".to_owned())
        );
        assert_eq!(local_value::<()>(Ok(None)), None, "absent");
        assert_eq!(local_value(Err("SecurityError")), None, "throwing");
    }

    #[test]
    fn only_a_failed_local_write_earns_the_warning() {
        assert!(Store::Local.warns_on_failed_write());
        assert!(
            !Store::Cookie.warns_on_failed_write(),
            "the web keeps its silent cookie write"
        );
    }

    #[test]
    fn cookie_is_the_default_store() {
        assert_eq!(Store::default(), Store::Cookie);
    }

    #[test]
    fn with_no_store_in_context_the_current_one_is_cookie() {
        Owner::new().with(|| assert_eq!(Store::current(), Store::Cookie, "none provided"));
    }

    #[test]
    fn the_store_in_context_is_the_current_one() {
        Owner::new().with(|| {
            provide_context(Store::Local);
            assert_eq!(Store::current(), Store::Local);
            let below = Owner::current().unwrap().child().with(Store::current);
            assert_eq!(below, Store::Local, "context flows down");
        });
    }

    #[cfg(not(feature = "browser"))]
    #[test]
    fn without_a_browser_local_has_no_storage() {
        assert_eq!(Store::Local.read("pc_rail"), None);
        assert_eq!(
            Store::Local.write("pc_rail", "true"),
            Err(StoreError::Unavailable)
        );
        assert_eq!(Store::Cookie.write("pc_rail", "true"), Ok(()), "no-op");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn the_cookie_store_reads_the_request_and_the_local_store_does_not() {
        let parts = http::Request::builder()
            .header(http::header::COOKIE, "a=1; pc_rail=false")
            .body(())
            .unwrap()
            .into_parts()
            .0;
        Owner::new().with(|| {
            provide_context(parts);
            assert_eq!(Store::Cookie.read("pc_rail").as_deref(), Some("false"));
            assert_eq!(Store::Cookie.read("pc_nope"), None);
            assert_eq!(Store::Local.read("pc_rail"), None);
        });
    }

    #[test]
    fn the_error_says_what_failed() {
        assert!(StoreError::Unavailable.to_string().contains("unavailable"));
        assert!(StoreError::Rejected.to_string().contains("refused"));
    }
}
