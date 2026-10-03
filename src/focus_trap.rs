//! Focus discipline for a modal container: moves focus into it when it
//! opens, keeps Tab cycling among its focusable descendants, restores focus
//! to wherever it was before once it closes, and locks the page's scroll
//! while it is open. At most one container managed this way is ever
//! "current" at a time: opening a second one closes the first immediately
//! (no exit animation, no stacking) with a single focus move straight into
//! the new one — the first does not also restore focus.
//!
//! Generic over [`Disclosure`] so it knows nothing about
//! [`crate::dialog::Dialog`] itself ("used by Dialog, not the reverse");
//! nothing else needs it yet. The registry that tracks which container is
//! current is a module-level `thread_local`, not Leptos context: a page can
//! hold several independent `Dialog`s as siblings with no shared owner to
//! provide context through, and the DOM — like the registry — is a single,
//! page-wide resource regardless of the component tree's shape (WASM is
//! single-threaded, so `thread_local` is exactly one registry per page).
//!
//! Browser-only: every real behaviour here is a no-op during SSR (there is
//! no `window`), so — like [`crate::popover`]'s own outside-click and
//! Escape handling — it carries no tests of its own.

use crate::button::Disclosure;

/// Moves focus into `controls.id()` when it opens, traps Tab inside it,
/// restores focus on close (or hands off silently if another managed
/// container replaced it first), and locks page scroll while it is open.
/// Call once per container. No-op during SSR.
pub fn manage<D: Disclosure>(controls: D) {
    #[cfg(feature = "hydrate")]
    hydrate::manage(controls);
    #[cfg(not(feature = "hydrate"))]
    let _ = controls;
}

#[cfg(feature = "hydrate")]
mod hydrate {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use leptos::prelude::*;
    use wasm_bindgen::JsCast;

    use crate::button::Disclosure;

    /// Closes the currently-current container and flags it to skip its own
    /// restore-focus step: the new one opening is about to make the single
    /// focus move instead.
    type Closer = Rc<dyn Fn()>;

    thread_local! {
        /// The id and closer of the one container currently open, if any.
        static CURRENT: RefCell<Option<(&'static str, Closer)>> = const { RefCell::new(None) };
    }

    /// Elements a dialog's Tab trap and initial focus consider reachable.
    const FOCUSABLE: &str = "a[href], button:not([disabled]), input:not([disabled]), \
        select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";

    fn focusables(container: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
        let Ok(nodes) = container.query_selector_all(FOCUSABLE) else {
            return Vec::new();
        };
        (0..nodes.length())
            .filter_map(|i| nodes.item(i))
            .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
            .collect()
    }

    /// Moves focus to the first focusable descendant of `id`, or to `id`
    /// itself if it has none (e.g. a confirmation with no close button yet
    /// focusable footer buttons further down).
    fn focus_into(id: &str) {
        let Some(container) = document().get_element_by_id(id) else {
            return;
        };
        let target = focusables(&container)
            .into_iter()
            .next()
            .or_else(|| container.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(el) = target {
            let _ = el.focus();
        }
    }

    /// Focus back to `previous` if it is still on the page, else to the
    /// `<main>` landmark (e.g. a route change removed the opener).
    fn restore_focus(previous: Option<web_sys::HtmlElement>) {
        if let Some(el) = previous.filter(|el| el.is_connected()) {
            let _ = el.focus();
            return;
        }
        let main = document()
            .query_selector("main")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(main) = main {
            let _ = main.focus();
        }
    }

    fn lock_scroll(locked: bool) {
        let Some(body) = document().body() else {
            return;
        };
        let style = web_sys::HtmlElement::style(&body);
        if locked {
            let _ = style.set_property("overflow", "hidden");
        } else {
            let _ = style.remove_property("overflow");
        }
    }

    /// While `controls.is_open()`, Tab on the container's last focusable
    /// descendant (or Shift+Tab on its first) wraps around instead of
    /// leaving the container.
    fn trap_tab<D: Disclosure>(controls: D) {
        let _ = window_event_listener(leptos::ev::keydown, move |ev| {
            if !controls.is_open() || ev.key() != "Tab" {
                return;
            }
            let Some(container) = document().get_element_by_id(controls.id()) else {
                return;
            };
            let items = focusables(&container);
            let (Some(first), Some(last)) = (items.first(), items.last()) else {
                return;
            };
            let active = document().active_element();
            let at_first = active.as_ref().is_some_and(|a| first.is_same_node(Some(a)));
            let at_last = active.as_ref().is_some_and(|a| last.is_same_node(Some(a)));
            if ev.shift_key() && at_first {
                let _ = last.focus();
                ev.prevent_default();
            } else if !ev.shift_key() && (at_last || active.is_none()) {
                let _ = first.focus();
                ev.prevent_default();
            }
        });
    }

    /// Another container was current: close it and flag it to skip its own
    /// restore, becomes current itself, remembers what had focus, locks
    /// scroll, and moves focus in.
    fn opened(
        id: &'static str,
        closer: &Closer,
        previous: &Rc<RefCell<Option<web_sys::HtmlElement>>>,
    ) {
        if let Some((_, old_closer)) = CURRENT.with(|c| c.borrow_mut().take()) {
            old_closer();
        }
        CURRENT.with(|c| *c.borrow_mut() = Some((id, closer.clone())));
        *previous.borrow_mut() = document()
            .active_element()
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
        lock_scroll(true);
        focus_into(id);
    }

    /// Clears `id` from the registry if it was current, then either skips
    /// the restore (it was replaced — the new one already moved focus) or
    /// restores focus and, if nothing replaced it, unlocks scroll.
    fn closed(
        id: &'static str,
        skip_restore: &Rc<Cell<bool>>,
        previous: &Rc<RefCell<Option<web_sys::HtmlElement>>>,
    ) {
        let was_current = CURRENT.with(|c| {
            let mut current = c.borrow_mut();
            let is_self = current.as_ref().map(|(cid, _)| *cid) == Some(id);
            if is_self {
                *current = None;
            }
            is_self
        });
        if skip_restore.replace(false) {
            return;
        }
        if was_current {
            lock_scroll(false);
        }
        restore_focus(previous.borrow_mut().take());
    }

    /// Wires one container's open/close transitions into [`opened`] and
    /// [`closed`], plus its own [`trap_tab`] listener.
    pub fn manage<D: Disclosure>(controls: D) {
        trap_tab(controls);
        let previous: Rc<RefCell<Option<web_sys::HtmlElement>>> = Rc::new(RefCell::new(None));
        let skip_restore = Rc::new(Cell::new(false));
        let closer: Closer = {
            let skip_restore = skip_restore.clone();
            Rc::new(move || {
                skip_restore.set(true);
                controls.set_open(false);
            })
        };
        Effect::new(move |was_open: Option<bool>| {
            let is_open = controls.is_open();
            if is_open && !was_open.unwrap_or(false) {
                opened(controls.id(), &closer, &previous);
            } else if !is_open && was_open.unwrap_or(false) {
                closed(controls.id(), &skip_restore, &previous);
            }
            is_open
        });
    }
}
