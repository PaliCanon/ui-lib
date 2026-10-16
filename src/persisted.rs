//! A value that survives page loads and is shared by every PaliCanon
//! subdomain: a signal mirrored into a cookie (the default
//! [`Store`]; a page with no server and no cookies
//! provides [`Store::Local`], same keys, same
//! behaviour).
//!
//! The cookie by default, not browser storage, because the server renders the first
//! page: it reads the cookie from the request, so the page arrives already
//! in the right state (no flash), and the browser starts from the same value
//! when it hydrates. On `*.palicanon.app` the cookie is set on the parent
//! domain, so `palicanon.app` and `ai.palicanon.app` share it.

use std::str::FromStr;

use leptos::prelude::*;

use crate::store::{Store, parse_stored};
pub use crate::store::{cookie_assignment, cookie_value};

/// Cookie names are prefixed so they never collide with anyone else's.
pub const PREFIX: &str = "pc_";

/// A signal holding the value stored under `pc_<name>`, or `default` when
/// there is none (or it does not parse). Changes are written back to the
/// current [`Store`] in the browser.
pub fn persisted<T>(name: &'static str, default: T) -> RwSignal<T>
where
    T: FromStr + ToString + Clone + Send + Sync + 'static,
{
    let key = format!("{PREFIX}{name}");
    let initial = parse_stored(Store::current().read(&key).as_deref()).unwrap_or(default);
    persisted_with(name, initial)
}

/// A signal starting at `initial` — a value the caller already resolved,
/// not read from the cookie — whose later changes are written to the
/// `pc_<name>` cookie in the browser, exactly as [`persisted`]'s are. For
/// a value whose first state comes from somewhere else than its own cookie:
/// [`crate::language::language_signal`] hydrates from `<html lang>`.
pub fn persisted_with<T>(name: &'static str, initial: T) -> RwSignal<T>
where
    T: ToString + Clone + Send + Sync + 'static,
{
    let signal = RwSignal::new(initial);
    #[cfg(feature = "browser")]
    {
        let key = format!("{PREFIX}{name}");
        let store = Store::current();
        Effect::new(move |previous: Option<()>| {
            let value = signal.get().to_string();
            // The first run only reads: the starting value is no choice —
            // the cookie holds it already, or it came from elsewhere
            // (`Accept-Language`) and is resolved again on the next request.
            if previous.is_some()
                && let Err(error) = store.write(&key, &value)
                && store.warns_on_failed_write()
            {
                crate::store::warn_once(error);
            }
        });
    }
    #[cfg(not(feature = "browser"))]
    let _ = name;
    signal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "ssr")]
    #[test]
    fn persisted_with_starts_from_the_given_value_not_the_cookie() {
        Owner::new().with(|| {
            let request = http_request_with_cookie("pc_count=7");
            provide_context(request);
            assert_eq!(
                persisted("count", 0_u32).get_untracked(),
                7,
                "persisted reads it"
            );
            assert_eq!(
                persisted_with("count", 3_u32).get_untracked(),
                3,
                "persisted_with does not"
            );
        });
    }

    #[cfg(feature = "ssr")]
    fn http_request_with_cookie(cookie: &str) -> http::request::Parts {
        http::Request::builder()
            .header(http::header::COOKIE, cookie)
            .body(())
            .unwrap()
            .into_parts()
            .0
    }

    /// The store is the only reader: under `Local`, the request's cookie is
    /// not consulted (and natively `Local` holds nothing).
    #[cfg(feature = "ssr")]
    #[test]
    fn a_local_store_does_not_read_the_request_cookie() {
        Owner::new().with(|| {
            provide_context(http_request_with_cookie("pc_count=7"));
            provide_context(Store::Local);
            assert_eq!(persisted("count", 0_u32).get_untracked(), 0);
        });
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
