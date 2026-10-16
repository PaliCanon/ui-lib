//! The client-side-only mode: a page with no server, built by the browser
//! from an empty document (Leptos `csr`). The counterpart of
//! [`crate::hydration`] for an app such as the PaliCanon app's webview, which
//! has no server to render the first page and keeps no cookies.
//!
//! [`mount`] is that mode's entry: what an app's `main` calls in place of
//! `leptos::mount::mount_to_body`. It keeps the preferences in
//! `localStorage` ([`Store::Local`]) instead of cookies, and puts the
//! stored theme and language on `<html>` before anything renders, as the
//! server's document shell does for the web.
//!
//! An app that mounts with `leptos::mount` directly gets the cookie default
//! ([`Store::Cookie`]) and so loses its preferences with its cookies: this
//! is not guarded at runtime (a guard would also fire where `csr` and
//! `hydrate` unify in one build), the app's own check catches it.
#![cfg(feature = "csr")]

use leptos::mount::mount_to_body;
use leptos::prelude::*;

use crate::language::local_language;
use crate::store::Store;
use crate::theme::theme_attribute_in;

/// Mounts the app `app` into `<body>`, client side only, with its
/// preferences in `localStorage`. Called once per page; the app lives as
/// long as the page (`mount_to_body` leaks its owner).
///
/// Before `app` first renders it sets `<html data-theme>` from the stored
/// `pc_theme` (none for System, or for a missing or garbled value) and
/// `<html lang>` from the stored `pc_lang`, else the first supported
/// `navigator.languages` entry, else English. These are the very values the
/// mounted tree resolves, so the first render already has them. They are
/// set once the wasm runs: until then the document shows its stylesheet
/// defaults. The tree then reads and writes through [`Store::Local`]
/// (provided at the root): a choice lands in `localStorage` and never in
/// `document.cookie`; if the storage is unavailable the choice stays in
/// memory for the page and one console warning is logged.
pub fn mount<F, N>(app: F)
where
    F: FnOnce() -> N + 'static,
    N: IntoView + 'static,
{
    let store = Store::Local;
    set_page_attributes(store);
    mount_to_body(move || {
        provide_context(store);
        app()
    });
}

/// Sets `<html data-theme>` and `<html lang>` from `store`, the way the
/// mounted tree will resolve them.
fn set_page_attributes(store: Store) {
    let Some(html) = document().document_element() else {
        return;
    };
    // Setting an attribute only fails on an invalid name.
    match theme_attribute_in(store) {
        Some(theme) => _ = html.set_attribute("data-theme", theme),
        None => _ = html.remove_attribute("data-theme"),
    }
    _ = html.set_attribute("lang", local_language().code());
}
