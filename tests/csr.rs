//! The client-side-only mode in a real browser (task 067): `csr::mount`
//! builds the page from an empty document and keeps the preferences in
//! `localStorage`, not in cookies (the app's webview keeps none).
//!
//! Run with `wasm-bindgen-test-runner` and a headless Chrome:
//! `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner
//! CHROMEDRIVER=<chromedriver> cargo test --locked --features csr
//! --target wasm32-unknown-unknown --test csr`.
//!
//! `csr::mount` leaks its owner (the app lives as long as the page): each
//! mounting test gives its app a distinct root id and looks only inside it,
//! and every test starts from empty storage. The leaked trees of earlier
//! tests only write when one of their own signals changes. `<html>` and its
//! `data-theme` / `lang` attributes are global to the page, shared by every
//! test, so each test sets (or has its mount set) the values it asserts.

#![cfg(all(target_arch = "wasm32", feature = "csr"))]

use std::cell::RefCell;

use leptos::prelude::*;
use palicanon_ui_lib::theme::{Theme, theme_attribute, theme_signal};
use palicanon_ui_lib::{AppShell, Language, Store, StoreError, csr, language_signal, persisted};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

// Makes `localStorage` unavailable the way a browser does (a getter that
// returns null, or throws) or full (`setItem` throws), and puts it back.
#[wasm_bindgen(inline_js = r"
let saved;
let setItem;
export function storage_unavailable(mode) {
  saved = Object.getOwnPropertyDescriptor(window, 'localStorage');
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    get() {
      if (mode === 'throws') throw new DOMException('blocked', 'SecurityError');
      return null;
    },
  });
}
export function storage_restore() {
  if (saved) Object.defineProperty(window, 'localStorage', saved);
  else delete window.localStorage;
}
export function storage_full() {
  setItem = Storage.prototype.setItem;
  Storage.prototype.setItem = function () {
    throw new DOMException('full', 'QuotaExceededError');
  };
}
export function storage_not_full() { Storage.prototype.setItem = setItem; }
let getItemSaved;
let setItemSaved;
export function storage_methods_throw() {
  if (getItemSaved) return;
  getItemSaved = Storage.prototype.getItem;
  setItemSaved = Storage.prototype.setItem;
  const blocked = function () {
    throw new DOMException('blocked', 'SecurityError');
  };
  Storage.prototype.getItem = blocked;
  Storage.prototype.setItem = blocked;
}
// A no-op when nothing is installed: `reset()` calls it first, because a
// wasm panic does not unwind, so a failed test never runs its guard's Drop.
export function storage_methods_restore() {
  if (!getItemSaved) return;
  Storage.prototype.getItem = getItemSaved;
  Storage.prototype.setItem = setItemSaved;
  getItemSaved = undefined;
  setItemSaved = undefined;
}
")]
extern "C" {
    fn storage_unavailable(mode: &str);
    fn storage_restore();
    fn storage_full();
    fn storage_not_full();
    fn storage_methods_throw();
    fn storage_methods_restore();
}

/// `window.localStorage`.
fn local() -> web_sys::Storage {
    window()
        .local_storage()
        .expect("localStorage is reachable")
        .expect("localStorage exists")
}

/// `document.cookie`.
fn cookies() -> String {
    document()
        .unchecked_into::<web_sys::HtmlDocument>()
        .cookie()
        .expect("document.cookie is readable")
}

/// Puts the page back to a blank state, at the start and the end of every
/// test: puts the throwing stub back if one is installed (a failed test
/// never runs its guard's `Drop`, a wasm panic does not unwind), removes the
/// `pc_*` keys these tests use, and removes `<html>`'s `data-theme` and
/// `lang`, which are global to the page and which a mount sets. Cookies are
/// not touched: no test here writes one.
fn reset() {
    storage_methods_restore();
    if let Some(html) = document().document_element() {
        _ = html.remove_attribute("data-theme");
        _ = html.remove_attribute("lang");
    }
    for key in ["pc_theme", "pc_lang", "pc_rail"] {
        local().remove_item(key).expect("the key is removable");
    }
}

/// What the mounted test app saw while it rendered.
#[derive(Default)]
struct Seen {
    data_theme: Option<String>,
    lang: Option<String>,
    store: Option<Store>,
    language: Option<Language>,
    theme: Option<RwSignal<Theme>>,
    rail: Option<RwSignal<bool>>,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

/// The app `csr::mount` mounts: it records `<html>` and the store as they
/// are when it first renders.
#[component]
fn TestApp() -> impl IntoView {
    let html = document().document_element().expect("the page has <html>");
    SEEN.with_borrow_mut(|seen| {
        seen.data_theme = html.get_attribute("data-theme");
        seen.lang = html.get_attribute("lang");
        seen.store = Some(Store::current());
        seen.language = Some(language_signal().get_untracked());
        seen.theme = Some(theme_signal());
        seen.rail = Some(persisted("rail", true));
    });
    view! { <p id="csr-app">"mounted"</p> }
}

/// The local store holds a value under its key and leaves cookies alone.
#[wasm_bindgen_test]
fn the_local_store_round_trips_and_writes_no_cookie() {
    reset();
    let before = cookies();
    assert_eq!(Store::Local.read("pc_rail"), None, "absent");
    Store::Local.write("pc_rail", "false").expect("stored");
    assert_eq!(Store::Local.read("pc_rail").as_deref(), Some("false"));
    assert_eq!(
        local().get_item("pc_rail").unwrap().as_deref(),
        Some("false")
    );
    assert_eq!(cookies(), before, "document.cookie is unchanged");
    reset();
}

/// Unavailable storage reads as no value and refuses writes.
#[wasm_bindgen_test]
fn unavailable_storage_reads_as_none_and_a_write_is_an_error() {
    reset();
    for mode in ["null", "throws"] {
        storage_unavailable(mode);
        assert_eq!(Store::Local.read("pc_theme"), None, "{mode}");
        assert_eq!(
            Store::Local.write("pc_theme", "dark"),
            Err(StoreError::Unavailable),
            "{mode}"
        );
        storage_restore();
    }
    storage_full();
    assert_eq!(
        Store::Local.write("pc_theme", "dark"),
        Err(StoreError::Rejected),
        "a full storage refuses the value"
    );
    storage_not_full();
    assert_eq!(
        local().get_item("pc_theme").unwrap(),
        None,
        "nothing written"
    );
}

/// The theme attribute is read from `localStorage` under the local store;
/// a garbled value is no theme.
#[wasm_bindgen_test]
fn the_theme_attribute_reads_localstorage_and_ignores_garbage() {
    reset();
    let attribute = || {
        Owner::new().with(|| {
            provide_context(Store::Local);
            theme_attribute()
        })
    };
    assert_eq!(attribute(), None, "absent");
    local().set_item("pc_theme", "dark").unwrap();
    assert_eq!(attribute(), Some("dark"));
    local().set_item("pc_theme", "sepia").unwrap();
    assert_eq!(attribute(), None, "garbled");
    storage_unavailable("throws");
    assert_eq!(attribute(), None, "throwing");
    storage_restore();
    reset();
}

/// `csr::mount`: the page's `data-theme` and `lang` are set from the stored
/// values before the first render, the tree reads the local store, and a
/// later choice lands in `localStorage` and not in `document.cookie`; with
/// the storage gone the choice stays in memory.
#[wasm_bindgen_test]
async fn mount_sets_the_page_then_stores_choices_locally() {
    reset();
    local().set_item("pc_theme", "dark").unwrap();
    local().set_item("pc_lang", "vi").unwrap();
    local().set_item("pc_rail", "false").unwrap();
    let cookies_before = cookies();

    csr::mount(TestApp);

    let (data_theme, lang, store, language, theme, rail) = SEEN.with_borrow(|seen| {
        (
            seen.data_theme.clone(),
            seen.lang.clone(),
            seen.store,
            seen.language,
            seen.theme.expect("the app rendered"),
            seen.rail.expect("the app rendered"),
        )
    });
    assert_eq!(data_theme.as_deref(), Some("dark"), "set before the render");
    assert_eq!(lang.as_deref(), Some("vi"), "set before the render");
    assert_eq!(store, Some(Store::Local), "provided at the root");
    assert_eq!(language, Some(Language::Vi));
    assert_eq!(theme.get_untracked(), Theme::Dark);
    assert!(!rail.get_untracked(), "read from localStorage");

    // The persisting effects run once on their own (they only read); a
    // choice is what changes a signal after that.
    leptos::task::tick().await;
    theme.set(Theme::Light);
    leptos::task::tick().await;
    assert_eq!(
        local().get_item("pc_theme").unwrap().as_deref(),
        Some("light")
    );
    assert_eq!(cookies(), cookies_before, "document.cookie is unchanged");

    storage_unavailable("null");
    rail.set(true);
    leptos::task::tick().await;
    storage_restore();
    assert!(rail.get_untracked(), "the choice stays in memory");
    assert_eq!(
        local().get_item("pc_rail").unwrap().as_deref(),
        Some("false")
    );
    reset();
}

/// What the mounted shell app exposes: the page's two signals.
struct ShellSeen {
    language: RwSignal<Language>,
    theme: RwSignal<Theme>,
}

thread_local! {
    static SHELL: RefCell<Option<ShellSeen>> = const { RefCell::new(None) };
}

/// A full [`AppShell`] inside `<div id=root>`, the signals it shares with
/// the page recorded in `SHELL`.
#[component]
fn ShellApp(root: &'static str) -> impl IntoView {
    let language = language_signal();
    let theme = theme_signal();
    SHELL.with_borrow_mut(|seen| *seen = Some(ShellSeen { language, theme }));
    view! {
        <div id=root>
            <AppShell>
                <p>"page"</p>
            </AppShell>
        </div>
    }
}

/// Mounts [`ShellApp`] under `root` through `csr::mount`.
fn mount_shell(root: &'static str) -> (RwSignal<Language>, RwSignal<Theme>) {
    csr::mount(move || view! { <ShellApp root=root /> });
    SHELL.with_borrow(|seen| {
        let seen = seen.as_ref().expect("the shell app rendered");
        (seen.language, seen.theme)
    })
}

/// The first element matching `selector` inside `#root`.
fn within(root: &str, selector: &str) -> web_sys::HtmlElement {
    document()
        .query_selector(&format!("#{root} {selector}"))
        .expect("a valid selector")
        .unwrap_or_else(|| panic!("no {selector} inside #{root}"))
        .unchecked_into()
}

/// The `<html>` element's attribute `name`.
fn html_attribute(name: &str) -> Option<String> {
    document()
        .document_element()
        .expect("the page has <html>")
        .get_attribute(name)
}

/// The first supported language of `navigator.languages`, else English:
/// what a page with no stored choice detects.
fn detected_language() -> Language {
    window()
        .navigator()
        .languages()
        .iter()
        .filter_map(|tag| tag.as_string())
        .find_map(|tag| Language::from_tag(&tag))
        .unwrap_or_default()
}

/// Clicks the desktop rail's lotus inside `#root`: it opens (`aria-expanded`)
/// and the state lands in `localStorage`, not in `document.cookie`.
async fn click_the_rail_lotus(root: &str, cookies_before: &str) {
    let lotus = within(root, r#"nav[data-open] button[aria-controls="pc-rail"]"#);
    assert_eq!(
        lotus.get_attribute("aria-expanded").as_deref(),
        Some("false")
    );
    lotus.click();
    leptos::task::tick().await;
    assert_eq!(
        lotus.get_attribute("aria-expanded").as_deref(),
        Some("true")
    );
    assert_eq!(
        local().get_item("pc_rail").unwrap().as_deref(),
        Some("true"),
        "the rail's state is stored locally"
    );
    assert_eq!(cookies(), cookies_before, "document.cookie is unchanged");
}

/// With no stored `pc_lang`, the page detects its language from the browser
/// and never writes it; a language choice then lands in `localStorage`, not
/// in `document.cookie`, and `<html lang>` follows. A click on the rail's
/// lotus (the shell's code that went through the `hydrate` to `browser`
/// rename) reacts and is stored the same way.
#[wasm_bindgen_test]
async fn a_detected_language_is_not_stored_but_a_choice_is() {
    reset();
    let cookies_before = cookies();

    let (language, _theme) = mount_shell("shell-choice");
    leptos::task::tick().await;
    assert_eq!(language.get_untracked(), detected_language());
    assert_eq!(
        html_attribute("lang").as_deref(),
        Some(detected_language().code()),
        "<html lang> is the detected language"
    );
    assert_eq!(
        local().get_item("pc_lang").unwrap(),
        None,
        "detection is never written"
    );

    // Vi is never the detected one in a headless Chrome set to en-US.
    let choice = if detected_language() == Language::Vi {
        Language::Fr
    } else {
        Language::Vi
    };
    language.set(choice);
    leptos::task::tick().await;
    assert_eq!(
        local().get_item("pc_lang").unwrap().as_deref(),
        Some(choice.code()),
        "the choice lands in localStorage"
    );
    assert_eq!(html_attribute("lang").as_deref(), Some(choice.code()));
    assert_eq!(cookies(), cookies_before, "document.cookie is unchanged");

    click_the_rail_lotus("shell-choice", &cookies_before).await;
    reset();
}

/// While alive, `Storage.prototype.getItem` and `setItem` throw (blocked
/// storage). Dropping it puts them back, so a test that returns early (or
/// ends) cannot leave the later tests' storage throwing.
struct ThrowingStorage;

impl ThrowingStorage {
    fn install() -> Self {
        storage_methods_throw();
        Self
    }
}

impl Drop for ThrowingStorage {
    fn drop(&mut self) {
        storage_methods_restore();
    }
}

/// A language other than the detected one, to make a choice with.
fn another_language() -> Language {
    if detected_language() == Language::Vi {
        Language::Fr
    } else {
        Language::Vi
    }
}

/// The page over throwing storage starts from the defaults.
fn assert_blocked_defaults(language: RwSignal<Language>, theme: RwSignal<Theme>) {
    assert_eq!(language.get_untracked(), detected_language(), "the default");
    assert_eq!(theme.get_untracked(), Theme::System, "the default");
    assert_eq!(html_attribute("data-theme"), None, "System sets none");
    assert_eq!(
        html_attribute("lang").as_deref(),
        Some(detected_language().code())
    );
}

/// A theme choice and a rail click over throwing storage stay in memory.
async fn choices_stay_in_memory(theme: RwSignal<Theme>) {
    let lotus = within(
        "shell-blocked",
        r#"nav[data-open] button[aria-controls="pc-rail"]"#,
    );
    assert_eq!(
        lotus.get_attribute("aria-expanded").as_deref(),
        Some("false")
    );
    theme.set(Theme::Dark);
    lotus.click();
    leptos::task::tick().await;
    assert_eq!(theme.get_untracked(), Theme::Dark, "kept in memory");
    assert_eq!(html_attribute("data-theme").as_deref(), Some("dark"));
    assert_eq!(
        lotus.get_attribute("aria-expanded").as_deref(),
        Some("true")
    );
}

/// `localStorage` there but throwing on every read and write (blocked
/// storage): the page still mounts with its defaults, and a choice stays in
/// its signal for the page. Once the storage works again, the next choice
/// is stored.
#[wasm_bindgen_test]
async fn mounting_over_throwing_storage_renders_defaults_and_keeps_choices() {
    reset();
    let blocked = ThrowingStorage::install();
    let (language, theme) = mount_shell("shell-blocked");
    leptos::task::tick().await;
    assert_blocked_defaults(language, theme);
    choices_stay_in_memory(theme).await;
    drop(blocked);

    assert_eq!(local().get_item("pc_theme").unwrap(), None, "not stored");
    assert_eq!(local().get_item("pc_rail").unwrap(), None, "not stored");
    let choice = another_language();
    language.set(choice);
    leptos::task::tick().await;
    assert_eq!(
        local().get_item("pc_lang").unwrap().as_deref(),
        Some(choice.code()),
        "a choice made once the storage works is stored"
    );
    theme.set(Theme::Light);
    leptos::task::tick().await;
    assert_eq!(
        local().get_item("pc_theme").unwrap().as_deref(),
        Some("light"),
        "and so is a theme choice"
    );
    reset();
}

/// A stored `pc_lang` beats what the browser prefers: the language order is
/// stored, then `navigator.languages`, then English (v3).
#[wasm_bindgen_test]
async fn a_stored_language_beats_the_detected_one() {
    reset();
    let stored = another_language();
    assert_ne!(stored, detected_language(), "the test needs a difference");
    local().set_item("pc_lang", stored.code()).unwrap();

    let (language, _theme) = mount_shell("shell-stored-lang");
    leptos::task::tick().await;
    assert_eq!(language.get_untracked(), stored);
    assert_eq!(html_attribute("lang").as_deref(), Some(stored.code()));
    assert_eq!(
        local().get_item("pc_lang").unwrap().as_deref(),
        Some(stored.code()),
        "the stored value is left as it was"
    );
    reset();
}

/// A garbled `pc_theme` gives `<html data-theme>` exactly what
/// `theme_attribute()` gives it under the local store (none: System),
/// whatever an earlier page state left on `<html>`.
#[wasm_bindgen_test]
async fn a_garbled_stored_theme_maps_as_theme_attribute_maps_it() {
    reset();
    local().set_item("pc_theme", "sepia").unwrap();
    let expected = Owner::new().with(|| {
        provide_context(Store::Local);
        theme_attribute()
    });
    assert_eq!(expected, None, "garbled is no theme");
    document()
        .document_element()
        .expect("the page has <html>")
        .set_attribute("data-theme", "dark")
        .unwrap();

    let (_language, theme) = mount_shell("shell-garbled-theme");
    // Straight after mount, before any tick: `AppShell`'s own effect would
    // otherwise fix the attribute afterwards and hide what `mount` set.
    assert_eq!(html_attribute("data-theme").as_deref(), expected);
    assert_eq!(theme.get_untracked(), Theme::System);
    leptos::task::tick().await;
    assert_eq!(
        html_attribute("data-theme").as_deref(),
        expected,
        "and after"
    );
    reset();
}

/// `reset()` leaves a clean page: no keys, no stub, no `<html>` attributes.
#[wasm_bindgen_test]
fn reset_clears_the_page_attributes_the_keys_and_the_stub() {
    let html = document().document_element().expect("the page has <html>");
    html.set_attribute("data-theme", "dark").unwrap();
    html.set_attribute("lang", "vi").unwrap();
    local().set_item("pc_theme", "dark").unwrap();
    storage_methods_throw();

    reset();
    assert_eq!(html_attribute("data-theme"), None);
    assert_eq!(html_attribute("lang"), None);
    assert_eq!(local().get_item("pc_theme").unwrap(), None, "keys");
    local().set_item("pc_rail", "x").unwrap();
    assert_eq!(local().get_item("pc_rail").unwrap().as_deref(), Some("x"));
    reset();
}
