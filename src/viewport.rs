//! The part of the page the user actually sees. On a phone, the browser's
//! toolbars cover part of the window: `window.innerHeight` counts the area
//! under them, the visual viewport does not — placements are measured
//! against the visual viewport when the browser has one.

/// The visible width and height in CSS pixels: the visual viewport when
/// the browser has one, the window otherwise.
#[cfg(feature = "browser")]
pub(crate) fn visible_size() -> (f64, f64) {
    let win = leptos::prelude::window();
    if let Some(vv) = win.visual_viewport() {
        return (vv.width(), vv.height());
    }
    let px = |v: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>| {
        v.ok().and_then(|v| v.as_f64()).unwrap_or(f64::MAX)
    };
    (px(win.inner_width()), px(win.inner_height()))
}
