//! The browser-only glue of the carousel, thin on purpose: it reads four
//! numbers off the live element and hands them to the pure functions of
//! `scroll`, which hold every decision and are tested natively. Without
//! the `hydrate` feature every function here does nothing.

use leptos::prelude::*;

use super::scroll::Direction;

/// The scroller's element type.
pub(super) type Scroller = NodeRef<leptos::html::Div>;

/// Once the scroller is mounted after hydration: reads the two end flags
/// from it, turns `hydrated` on (which shows the controls), and keeps the
/// flags current on every scroll (a passive listener on the scroller) and
/// every window resize. The listeners end with the carousel.
pub(super) fn track(
    scroller: Scroller,
    hydrated: RwSignal<bool>,
    at_start: RwSignal<bool>,
    at_end: RwSignal<bool>,
) {
    #[cfg(feature = "browser")]
    Effect::new(move |_| {
        let Some(el) = scroller.get() else {
            return;
        };
        refresh(&el, at_start, at_end);
        listen(el, at_start, at_end);
        hydrated.set(true);
    });
    #[cfg(not(feature = "browser"))]
    let _ = (scroller, hydrated, at_start, at_end);
}

/// A button press: scrolls the scroller to the previous or next slide's
/// left edge (stopped at the end of the scroll range), read from the DOM
/// now, and announces the slide reached ("n of `total`", in the current
/// language) in `status`; `presses` counts the presses, so a repeated press
/// to the same slide is announced again. Nothing when there is no slide
/// that way. `total` is the slide count of the render, the one N.
pub(super) fn press(
    scroller: Scroller,
    direction: Direction,
    total: usize,
    status: RwSignal<String>,
    presses: StoredValue<usize>,
) {
    #[cfg(feature = "browser")]
    {
        use super::scroll::{clamp_index, scroll_target, target_index};
        use super::status_text;
        use crate::catalogue::Locale;
        use crate::language::language_signal;

        let Some(el) = scroller.get_untracked() else {
            return;
        };
        let lefts = slide_lefts(&el);
        let scroll_left = f64::from(el.scroll_left());
        let client = f64::from(el.client_width());
        let content = f64::from(el.scroll_width());
        let Some(index) = target_index(&lefts, scroll_left, client, content, direction) else {
            return;
        };
        // Clamp the index to the rendered slides before using it.
        let shown = clamp_index(index, total);
        // The element's own `scroll-behavior` decides whether this glides.
        el.scroll_to_with_x_and_y(scroll_target(lefts[shown], client, content), 0.0);
        presses.update_value(|count| *count += 1);
        let locale = Locale::from(language_signal().get_untracked());
        status.set(status_text(locale, shown, total, presses.get_value()));
    }
    #[cfg(not(feature = "browser"))]
    let _ = (scroller, direction, total, status, presses);
}

/// Sets the two flags from the element's current position.
#[cfg(feature = "browser")]
fn refresh(el: &web_sys::Element, at_start: RwSignal<bool>, at_end: RwSignal<bool>) {
    use super::scroll;
    let left = f64::from(el.scroll_left());
    at_start.set(scroll::at_start(left));
    at_end.set(scroll::at_end(
        left,
        f64::from(el.client_width()),
        f64::from(el.scroll_width()),
    ));
}

/// Refreshes the flags on the element's scroll (passive) and on the
/// window's resize; both listeners are removed with the carousel's owner.
#[cfg(feature = "browser")]
fn listen(el: web_sys::HtmlDivElement, at_start: RwSignal<bool>, at_end: RwSignal<bool>) {
    use wasm_bindgen::{JsCast, closure::Closure};
    let on_resize = el.clone();
    let resize = window_event_listener(leptos::ev::resize, move |_| {
        refresh(&on_resize, at_start, at_end);
    });
    let on_scroll = el.clone();
    let scroll =
        Closure::<dyn Fn()>::new(move || refresh(&on_scroll, at_start, at_end)).into_js_value();
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(true);
    let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
        "scroll",
        scroll.unchecked_ref(),
        &options,
    );
    // The JS function and the element are not `Send`: kept in local storage,
    // read at cleanup (cleanups run before the owner's values are disposed).
    let listener = StoredValue::new_local((el, scroll));
    on_cleanup(move || {
        resize.remove();
        listener.try_with_value(|(el, scroll)| {
            let _ = el.remove_event_listener_with_callback("scroll", scroll.unchecked_ref());
        });
    });
}

/// The slides' left edges, in the scroller's own coordinates (the scroller's
/// content, not the page): each slide's rectangle, minus the scroller's
/// plus its scroll position. Empty when the query fails. Selects only the
/// carousel's own slides (direct children with `data-slide`), not nested
/// carousels' slides.
#[cfg(feature = "browser")]
fn slide_lefts(el: &web_sys::Element) -> Vec<f64> {
    use wasm_bindgen::JsCast;
    let origin = el.get_bounding_client_rect().left() - f64::from(el.scroll_left());
    let Ok(nodes) = el.query_selector_all(":scope > [data-slide]") else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|i| nodes.item(i))
        .filter_map(|node| node.dyn_into::<web_sys::Element>().ok())
        .map(|slide| slide.get_bounding_client_rect().left() - origin)
        .collect()
}
