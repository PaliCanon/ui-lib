//! A floating panel tied to a [`Disclosure`]:
//! shown while it is open, hidden otherwise. Once hydrated, it also closes
//! itself on Escape and on a click outside both the panel and its trigger
//! (the element whose `aria-controls` names it) — generic enough for a
//! settings menu, a filter panel, or anything else disclosed near a button.
//!
//! Placement: the caller places it (layout classes), usually just below its
//! trigger. Once open in the browser, a panel that would run past the bottom
//! of the visible viewport is flipped above its trigger instead (when there
//! is more room there). A `fixed` panel is left where the caller put it.

use leptos::prelude::*;

use crate::button::Disclosure;

/// The look of a floating panel: this popover, and a [`crate::menu::Menu`]'s
/// nested submenu panels (which are not a [`Popover`] themselves — one
/// outside click or Escape must close the whole chain, see
/// `menu::level_keydown` — but share its appearance).
pub(crate) const PANEL: &str = "rounded-xl border border-line bg-card shadow-lg p-2";

/// A panel controlled by a [`Disclosure`]: its `id` is the disclosure's own,
/// so the button that opens it can name it with `aria-controls`.
#[component]
pub fn Popover<D: Disclosure>(
    /// What opens and closes it.
    controls: D,
    /// Its name for assistive technology (the panel has no visible title of
    /// its own).
    #[prop(into)]
    aria_label: String,
    /// Layout classes (position, width) added to the panel's look — never
    /// colours.
    #[prop(optional, into)]
    layout: String,
    /// The panel's content.
    children: Children,
) -> impl IntoView {
    close_on_outside_interaction(controls);
    let panel: NodeRef<leptos::html::Div> = NodeRef::new();
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if controls.is_open() {
            flip_if_overflowing(panel);
        }
    });
    let class = format!("{PANEL} {layout}");
    view! {
        <div
            node_ref=panel
            id=controls.id()
            role="dialog"
            aria-label=aria_label
            class=class
            hidden=move || !controls.is_open()
        >
            {children()}
        </div>
    }
}

/// How far a panel stays from the visible viewport's bottom edge.
#[cfg(feature = "hydrate")]
const VIEWPORT_MARGIN_PX: f64 = 8.0;

/// Once the open panel is laid out: if it runs past the bottom of the
/// visible viewport and its trigger has more room above, place it above the
/// trigger (`bottom: 100%`). Reset first, so each opening measures afresh.
#[cfg(feature = "hydrate")]
fn flip_if_overflowing(panel: NodeRef<leptos::html::Div>) {
    request_animation_frame(move || {
        let Some(el) = panel.get_untracked() else {
            return;
        };
        let style = web_sys::HtmlElement::style(&el);
        for prop in ["top", "bottom", "margin-top", "margin-bottom"] {
            let _ = style.remove_property(prop);
        }
        let fixed = window()
            .get_computed_style(&el)
            .ok()
            .flatten()
            .and_then(|s| s.get_property_value("position").ok())
            .is_some_and(|p| p == "fixed");
        let Some(anchor) = el.parent_element() else {
            return;
        };
        let (_, vh) = crate::viewport::visible_size();
        let rect = el.get_bounding_client_rect();
        let room_above = anchor.get_bounding_client_rect().top();
        if !fixed && rect.bottom() > vh - VIEWPORT_MARGIN_PX && room_above > vh - rect.top() {
            let _ = style.set_property("top", "auto");
            let _ = style.set_property("bottom", "100%");
            let _ = style.set_property("margin-top", "0");
            let _ = style.set_property("margin-bottom", "0.5rem");
        }
    });
}

/// The browser only: Escape, or a click outside the panel and its trigger,
/// closes `controls`. There is no window while the server renders.
fn close_on_outside_interaction<D: Disclosure>(controls: D) {
    #[cfg(feature = "hydrate")]
    {
        let _ = window_event_listener(leptos::ev::keydown, move |ev| {
            if controls.is_open() && ev.key() == "Escape" {
                controls.set_open(false);
            }
        });
        let _ = window_event_listener(leptos::ev::click, move |ev| {
            if controls.is_open() && click_is_outside(controls.id(), &ev) {
                controls.set_open(false);
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = controls;
}

/// Whether `ev` landed outside both the element `id` (the panel) and
/// whatever names it with `aria-controls` (its trigger).
#[cfg(feature = "hydrate")]
fn click_is_outside(id: &str, ev: &leptos::ev::MouseEvent) -> bool {
    use wasm_bindgen::JsCast;
    let selector = format!("#{id}, [aria-controls=\"{id}\"]");
    let Some(target) = ev.target() else {
        return true;
    };
    let Some(element) = target.dyn_ref::<web_sys::Element>() else {
        return true;
    };
    !matches!(element.closest(&selector), Ok(Some(_)))
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::button::Disclosed;

    #[test]
    fn hidden_while_closed_shown_while_open_and_named() {
        let closed = Owner::new().with(|| {
            let d = Disclosed::new("pc-settings", RwSignal::new(false));
            view! {
                <Popover controls=d aria_label="Settings">
                    <p>"content"</p>
                </Popover>
            }
            .to_html()
        });
        assert!(closed.contains("hidden"), "{closed}");
        assert!(closed.contains(r#"id="pc-settings""#), "{closed}");
        assert!(closed.contains(r#"role="dialog""#), "{closed}");
        assert!(closed.contains(r#"aria-label="Settings""#), "{closed}");

        let open = Owner::new().with(|| {
            let d = Disclosed::new("pc-settings", RwSignal::new(true));
            view! {
                <Popover controls=d aria_label="Settings">
                    <p>"content"</p>
                </Popover>
            }
            .to_html()
        });
        assert!(!open.contains("hidden"), "{open}");
        assert!(open.contains("<p>content</p>"), "{open}");
    }

    #[test]
    fn layout_classes_join_the_panel_look() {
        let html = Owner::new().with(|| {
            let d = Disclosed::new("pc-settings", RwSignal::new(true));
            view! {
                <Popover controls=d aria_label="Settings" layout="fixed bottom-3.5">
                    <p>"x"</p>
                </Popover>
            }
            .to_html()
        });
        assert!(html.contains("rounded-xl"), "{html}");
        assert!(html.contains("fixed bottom-3.5"), "{html}");
    }
}
