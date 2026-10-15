//! Hydration behind Cloudflare, in a real browser (task 063): the zone
//! injects nodes the server never rendered into `<body>` — a hidden decoy
//! link to `/cdn-cgi/content` as its FIRST child, the Web Analytics beacon
//! after the app. Hydrating from `<body>` meets the decoy where it expects
//! the app and panics; [`palicanon_ui_lib::hydrate`] starts at the
//! [`palicanon_ui_lib::HydrationRoot`] element instead.
//!
//! Run with `wasm-bindgen-test-runner` and a headless Chrome:
//! `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner
//! CHROMEDRIVER=<chromedriver> cargo test --locked --features hydrate
//! --target wasm32-unknown-unknown --test hydration`.
//!
//! The app's markup is the server's: the same view rendered by the `ssr`
//! build, kept in `tests/fixtures/hydration-counter.html`. It cannot be
//! rendered in the browser test itself: without `ssr`, tachys panics on
//! rendering a component to HTML (`You are rendering AnyAttribute to HTML
//! without the ssr feature enabled`), and with `tachys/ssr` on, event
//! handlers are dropped. `cargo test --features ssr` checks the fixture
//! against the render (`UPDATE_FIXTURES=1` rewrites it), so it is never
//! hand-written.
//!
//! `hydrate` leaks its owner (the app lives as long as the page), so this
//! binary calls it once, in `hydrates_past_the_nodes_cloudflare_injects`.

#![cfg(any(feature = "ssr", all(target_arch = "wasm32", feature = "hydrate")))]

use leptos::prelude::*;

/// The server's markup of `<HydrationRoot><Counter/></HydrationRoot>`.
const FIXTURE: &str = include_str!("fixtures/hydration-counter.html");

/// A small interactive view: a button counting its clicks.
#[component]
fn Counter() -> impl IntoView {
    let count = RwSignal::new(0);
    view! {
        <button id="counter" on:click=move |_| count.update(|n| *n += 1)>
            "Clicked "
            {move || count.get()}
        </button>
    }
}

#[cfg(feature = "ssr")]
mod server {
    use super::*;
    use palicanon_ui_lib::HydrationRoot;

    #[test]
    fn the_fixture_is_the_server_markup() {
        let markup = Owner::new().with(|| {
            view! {
                <HydrationRoot>
                    <Counter />
                </HydrationRoot>
            }
            .to_html()
        });
        if std::env::var_os("UPDATE_FIXTURES").is_some() {
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/hydration-counter.html"
            );
            std::fs::write(path, format!("{markup}\n")).expect("the fixture is writable");
            return;
        }
        assert_eq!(
            FIXTURE.trim_end(),
            markup,
            "the fixture is stale: run UPDATE_FIXTURES=1 cargo test --features ssr --test hydration"
        );
    }
}

#[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
mod browser {
    use super::*;
    use palicanon_ui_lib::{APP_ROOT_ID, app_root, hydrate};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::{Document, HtmlElement};

    wasm_bindgen_test_configure!(run_in_browser);

    /// The hidden link Cloudflare puts first in `<body>` (an AI-crawler decoy).
    const DECOY: &str = r#"<a href="/cdn-cgi/content?id=x" style="display:none">…</a>"#;
    /// The Web Analytics beacon Cloudflare appends after the app.
    const BEACON: &str =
        r#"<script src="https://static.cloudflareinsights.com/beacon.min.js"></script>"#;

    /// A new, empty HTML document, apart from the test page's own.
    fn blank_document() -> Document {
        document()
            .implementation()
            .expect("the page has a DOM implementation")
            .create_html_document()
            .expect("a blank HTML document")
    }

    #[wasm_bindgen_test]
    async fn hydrates_past_the_nodes_cloudflare_injects() {
        let markup = FIXTURE.trim_end();
        assert!(markup.starts_with(r#"<div id="pc-app">"#), "{markup}");
        let body = document().body().expect("the test page has a body");
        // Prepended, not replacing the body: the test runner reads its
        // results from its own elements, which stay after the beacon.
        body.insert_adjacent_html("afterbegin", &format!("{DECOY}{markup}{BEACON}"))
            .expect("the page's markup is inserted");

        hydrate(Counter);

        let button: HtmlElement = document()
            .get_element_by_id("counter")
            .expect("the counter is in the page")
            .unchecked_into();
        button.click();
        leptos::task::tick().await;
        assert_eq!(button.text_content().as_deref(), Some("Clicked 1"));

        let first = body.first_element_child().expect("body has children");
        assert_eq!(first.tag_name(), "A");
        assert_eq!(
            first.get_attribute("href").as_deref(),
            Some("/cdn-cgi/content?id=x")
        );

        let html = document().document_element().expect("the page has <html>");
        assert!(html.has_attribute("data-hydrated"), "{}", html.outer_html());
    }

    #[wasm_bindgen_test]
    fn a_missing_root_is_an_error_naming_the_root_and_the_fix() {
        let doc = blank_document();
        let message = app_root(&doc)
            .expect_err("no #pc-app in a blank document")
            .to_string();
        assert!(message.contains("pc-app"), "{message}");
        assert!(message.contains("HydrationRoot"), "{message}");
    }

    #[wasm_bindgen_test]
    fn the_root_is_found_by_its_id() {
        let doc = blank_document();
        let body = doc.body().expect("a blank HTML document has a body");
        body.set_inner_html(r#"<p>before</p><div id="pc-app"><p>app</p></div>"#);
        let root = app_root(&doc).expect("the root is in the document");
        assert_eq!(root.id(), APP_ROOT_ID);
        assert_eq!(root.inner_html(), "<p>app</p>");
    }
}
