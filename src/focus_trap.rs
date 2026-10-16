//! Focus discipline for a modal container: moves focus into it when it
//! opens, keeps Tab cycling among its focusable descendants, restores focus
//! to wherever it was before once it closes, and locks the page's scroll
//! while it is open. At most one container managed this way is ever
//! "current" at a time: opening a second one closes the first immediately
//! (no exit animation, no stacking) with a single focus move straight into
//! the new one — the first does not also restore focus. Unless the first
//! is not dismissable (a required confirmation): then the second is
//! refused — it closes again at once, and the first stays open, untouched.
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
//! no `window`). The one-open rule itself is decided by a plain registry
//! (the internal `registry` module), so it is tested without a DOM. The
//! glue around it (the internal `glue` module: the `Effect` following each
//! container, what it does when an open is admitted, replaced or refused)
//! moves the page through an injected `Page`, so tests run it natively —
//! `reactive_graph`'s real `Effect` on a tokio executor — against a
//! recording page (the test-only `fake` module), `Dialog`s included. The
//! DOM moves themselves — like [`crate::popover`]'s own outside-click and
//! Escape handling — carry no tests of their own.

use crate::button::Disclosure;

/// Moves focus into `controls.id()` when it opens, traps Tab inside it,
/// restores focus on close (or hands off silently if another managed
/// container replaced it first), and locks page scroll while it is open.
/// While it is open and not `dismissable`, any other container's open is
/// refused (that one closes again). Call once per container. No-op during
/// SSR.
pub fn manage<D: Disclosure>(controls: D, dismissable: bool) {
    #[cfg(feature = "browser")]
    hydrate::manage(controls, dismissable);
    // The test-only seam: a test that installed the recording page gets
    // the real glue; every other test renders as SSR does.
    #[cfg(all(test, not(feature = "browser")))]
    if fake::installed() {
        glue::manage::<fake::Recorder, D>(controls, dismissable);
    }
    #[cfg(not(any(test, feature = "browser")))]
    let _ = (controls, dismissable);
}

/// The one-open-at-a-time rule as plain data, so it is decided — and
/// tested — without a DOM. Generic over the closer type `C`: the browser
/// stores a closure, the tests a label.
#[cfg(any(test, feature = "browser"))]
mod registry {
    /// Names one `manage` call. Ids are not unique over time: a component
    /// that remounts reuses its id, and the instance it replaced may be
    /// disposed after the new one became current, so the registry matches
    /// on this token and never on the id.
    pub type Token = u64;

    /// One open container: its id and token, whether it may be closed by
    /// anything but its own actions, and how to close it.
    pub struct Entry<C> {
        /// Read by the tests only: the registry decides on the token.
        #[cfg_attr(not(test), allow(dead_code))]
        pub id: &'static str,
        pub token: Token,
        pub dismissable: bool,
        pub closer: C,
    }

    /// Which container is current, if any.
    pub struct Registry<C> {
        current: Option<Entry<C>>,
    }

    impl<C> Registry<C> {
        pub const fn new() -> Self {
            Self { current: None }
        }

        /// The current container's id.
        #[cfg(test)]
        pub fn current(&self) -> Option<&'static str> {
            self.current.as_ref().map(|e| e.id)
        }

        /// `entry` asks to become current. Refused — `Err` with `entry`
        /// itself — while the current one is not dismissable: only its own
        /// actions close it. Otherwise `Ok` with the closer of the one it
        /// replaces, if any, to call once the registry is released.
        pub fn open(&mut self, entry: Entry<C>) -> Result<Option<C>, Entry<C>> {
            if self.current.as_ref().is_some_and(|e| !e.dismissable) {
                return Err(entry);
            }
            Ok(self.current.replace(entry).map(|old| old.closer))
        }

        /// `token` closed: clears it if it was current, and says whether
        /// it was.
        pub fn close(&mut self, token: Token) -> bool {
            let is_self = self.current.as_ref().is_some_and(|e| e.token == token);
            if is_self {
                self.current = None;
            }
            is_self
        }
    }
}

/// The glue between one container's open state and the page: the
/// `Effect` that follows each open/close transition, the page-wide
/// registry it asks, and the moves around it. Generic over the [`Page`]
/// that makes those moves, so the same glue drives the browser's DOM and,
/// in tests, a recording page.
#[cfg(any(test, feature = "browser"))]
mod glue {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use leptos::prelude::*;

    use std::sync::atomic::{AtomicU64, Ordering};

    use super::registry::{Entry, Registry, Token};
    use crate::button::Disclosure;

    /// The page-wide moves around the one-open rule.
    pub trait Page: 'static {
        /// What had focus before a container opened, to give it back.
        type Focus: 'static;
        /// What has focus now, if anything.
        fn focused() -> Option<Self::Focus>;
        /// Moves focus into the container `id`.
        fn focus_into(id: &str);
        /// Gives focus back to `previous` (or a fallback when it is gone).
        fn restore_focus(previous: Option<Self::Focus>);
        /// Locks or unlocks the page's scroll.
        fn lock_scroll(locked: bool);
    }

    /// Closes a container from outside its own actions, and flags it to
    /// skip its own restore-focus step. Two callers: the registry replacing
    /// it (the new one makes the single focus move instead), and the
    /// container itself when its open was refused (it never took focus or
    /// locked scroll, so there is nothing to give back).
    type Closer = Rc<dyn Fn()>;

    /// What had focus before a container opened. A local arena value: it
    /// is `Copy`, so the container's cleanup can hold it, and it is still
    /// alive when that cleanup runs (cleanups run before the owner's
    /// values are dropped).
    type Previous<P> = StoredValue<Option<<P as Page>::Focus>, LocalStorage>;

    /// The next token: one per `manage` call.
    static NEXT_TOKEN: AtomicU64 = AtomicU64::new(0);

    thread_local! {
        /// The one container currently open, if any.
        static CURRENT: RefCell<Registry<Closer>> = const { RefCell::new(Registry::new()) };
    }

    /// Forgets the current container: each test starts with an empty page.
    #[cfg(all(test, not(feature = "browser")))]
    pub fn reset() {
        CURRENT.with(|c| *c.borrow_mut() = Registry::new());
    }

    /// Asks the registry to make `id` current. Refused (a non-dismissable
    /// container is open): `id` closes again through its own `closer`,
    /// which skips its restore — it never took focus or locked scroll.
    /// Admitted: closes the container it replaces (flagged to skip its own
    /// restore), remembers what had focus, locks scroll, moves focus in.
    fn opened<P: Page>(
        id: &'static str,
        token: Token,
        dismissable: bool,
        closer: &Closer,
        previous: Previous<P>,
    ) {
        let entry = Entry {
            id,
            token,
            dismissable,
            closer: closer.clone(),
        };
        match CURRENT.with(|c| c.borrow_mut().open(entry)) {
            Err(refused) => return (refused.closer)(),
            Ok(Some(old_closer)) => old_closer(),
            Ok(None) => {}
        }
        previous.set_value(P::focused());
        P::lock_scroll(true);
        P::focus_into(id);
    }

    /// Clears `id` from the registry if it was current, then either skips
    /// the restore — it was closed through its closer: replaced (the new
    /// one already moved focus) or refused (it never took focus) — or
    /// restores focus and, if nothing replaced it, unlocks scroll.
    fn closed<P: Page>(token: Token, skip_restore: &Cell<bool>, previous: Previous<P>) {
        let was_current = CURRENT.with(|c| c.borrow_mut().close(token));
        if skip_restore.replace(false) {
            return;
        }
        if was_current {
            P::lock_scroll(false);
        }
        P::restore_focus(previous.try_update_value(Option::take).flatten());
    }

    /// The container's owner is gone while it may still be current: its
    /// `Effect` will never see it close, so it leaves the registry here
    /// (nothing is left to call into the disposed owner, and a stale
    /// non-dismissable entry would refuse every later open) and, if it was
    /// current, gives back the scroll and the focus it took.
    fn disposed<P: Page>(token: Token, previous: Previous<P>) {
        if CURRENT.with(|c| c.borrow_mut().close(token)) {
            P::lock_scroll(false);
            P::restore_focus(previous.try_update_value(Option::take).flatten());
        }
    }

    /// Wires one container's open/close transitions into [`opened`] and
    /// [`closed`] through an `Effect`.
    pub fn manage<P: Page, D: Disclosure>(controls: D, dismissable: bool) {
        let token = NEXT_TOKEN.fetch_add(1, Ordering::Relaxed);
        let previous: Previous<P> = StoredValue::new_local(None);
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
                opened::<P>(controls.id(), token, dismissable, &closer, previous);
            } else if !is_open && was_open.unwrap_or(false) {
                closed::<P>(token, &skip_restore, previous);
            }
            is_open
        });
        on_cleanup(move || disposed::<P>(token, previous));
    }
}

/// The browser's [`glue::Page`]: the DOM's focus and the body's scroll,
/// plus the Tab trap.
#[cfg(feature = "browser")]
mod hydrate {
    use leptos::prelude::*;
    use wasm_bindgen::JsCast;

    use super::glue::{self, Page};
    use crate::button::Disclosure;

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

    /// The page as the browser has it.
    struct Dom;

    impl Page for Dom {
        type Focus = web_sys::HtmlElement;

        fn focused() -> Option<web_sys::HtmlElement> {
            document()
                .active_element()
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        }

        /// Moves focus to the first focusable descendant of `id`, or to
        /// `id` itself if it has none (e.g. a confirmation with no close
        /// button yet focusable footer buttons further down).
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

        /// Focus back to `previous` if it is still on the page, else to
        /// the `<main>` landmark (e.g. a route change removed the opener).
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
    }

    /// While `controls.is_open()`, Tab on the container's last focusable
    /// descendant (or Shift+Tab on its first) wraps around instead of
    /// leaving the container.
    fn trap_tab<D: Disclosure>(controls: D) {
        let handle = window_event_listener(leptos::ev::keydown, move |ev| {
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
        // The listener ends with the container's owner: it would otherwise
        // read a disposed `controls` on every later Tab.
        on_cleanup(move || handle.remove());
    }

    /// The browser's [`super::manage`]: the Tab trap, then the glue on
    /// the DOM.
    pub fn manage<D: Disclosure>(controls: D, dismissable: bool) {
        trap_tab(controls);
        glue::manage::<Dom, D>(controls, dismissable);
    }
}

/// A recording [`glue::Page`] and the harness that runs the glue natively:
/// a current-thread tokio runtime as `reactive_graph`'s executor (its
/// `effects` feature is on in the lab's test build), a fresh registry and
/// a fresh page per test.
#[cfg(all(test, not(feature = "browser")))]
pub(crate) mod fake {
    use std::cell::{Cell, RefCell};
    use std::future::Future;

    use any_spawner::Executor;
    use leptos::prelude::Owner;

    use super::glue::{self, Page};

    /// What the glue did to the page so far.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Moves {
        /// What has focus now.
        pub focused: Option<String>,
        /// Whether the page's scroll is locked now.
        pub locked: bool,
        /// How many times scroll was locked, and unlocked.
        pub locks: u32,
        pub unlocks: u32,
        /// Each container focus moved into, in order.
        pub focus_into: Vec<String>,
        /// Each restore-focus target, in order (`None`: nothing to give back).
        pub restores: Vec<Option<String>>,
    }

    thread_local! {
        static INSTALLED: Cell<bool> = const { Cell::new(false) };
        static MOVES: RefCell<Moves> = RefCell::new(Moves::default());
    }

    /// The recording page.
    pub struct Recorder;

    impl Page for Recorder {
        type Focus = String;

        fn focused() -> Option<String> {
            moves().focused
        }

        fn focus_into(id: &str) {
            MOVES.with(|m| {
                let mut m = m.borrow_mut();
                m.focused = Some(id.to_owned());
                m.focus_into.push(id.to_owned());
            });
        }

        fn restore_focus(previous: Option<String>) {
            MOVES.with(|m| {
                let mut m = m.borrow_mut();
                m.focused.clone_from(&previous);
                m.restores.push(previous);
            });
        }

        fn lock_scroll(locked: bool) {
            MOVES.with(|m| {
                let mut m = m.borrow_mut();
                m.locked = locked;
                if locked {
                    m.locks += 1;
                } else {
                    m.unlocks += 1;
                }
            });
        }
    }

    /// Whether this test runs the glue on the recording page.
    pub fn installed() -> bool {
        INSTALLED.with(Cell::get)
    }

    /// What the glue did so far.
    pub fn moves() -> Moves {
        MOVES.with(|m| m.borrow().clone())
    }

    /// Puts focus on `id`, as a click on an opener would.
    pub fn focus(id: &str) {
        MOVES.with(|m| m.borrow_mut().focused = Some(id.to_owned()));
    }

    /// Runs `test` with the recording page installed, under a fresh owner,
    /// inside a tokio `LocalSet` (where the glue's Effects are spawned).
    pub fn run<F: Future<Output = ()>>(test: impl FnOnce() -> F) {
        let _ = Executor::init_tokio();
        glue::reset();
        MOVES.with(|m| *m.borrow_mut() = Moves::default());
        INSTALLED.with(|i| i.set(true));
        let owner = Owner::new();
        owner.set();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a current-thread runtime");
        runtime.block_on(tokio::task::LocalSet::new().run_until(test()));
        INSTALLED.with(|i| i.set(false));
    }

    /// Lets every pending Effect run, and the ones they trigger.
    pub async fn settle() {
        for _ in 0..8 {
            Executor::tick().await;
            tokio::task::yield_now().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::registry::{Entry, Registry};

    /// A test's token for `id`: ids are unique within a test, except where
    /// a test builds two entries by hand.
    fn token(id: &'static str) -> u64 {
        u64::from(id.as_bytes()[0])
    }

    fn entry(id: &'static str) -> Entry<&'static str> {
        Entry {
            id,
            token: token(id),
            dismissable: true,
            closer: id,
        }
    }

    fn required(id: &'static str) -> Entry<&'static str> {
        Entry {
            dismissable: false,
            ..entry(id)
        }
    }

    #[test]
    fn the_first_open_becomes_current_and_closes_nothing() {
        let mut registry = Registry::new();
        assert!(matches!(registry.open(entry("a")), Ok(None)));
        assert_eq!(registry.current(), Some("a"));
    }

    #[test]
    fn closing_by_the_token_of_an_older_one_with_the_same_id_leaves_the_current_one() {
        let mut registry = Registry::new();
        let _ = registry.open(Entry {
            token: 1,
            ..entry("a")
        });
        assert!(!registry.close(2));
        assert_eq!(registry.current(), Some("a"));
        assert!(registry.close(1));
    }

    #[test]
    fn a_second_open_replaces_a_dismissable_one() {
        let mut registry = Registry::new();
        let _ = registry.open(entry("a"));
        assert!(matches!(registry.open(entry("b")), Ok(Some("a"))));
        assert_eq!(registry.current(), Some("b"));
    }

    #[test]
    fn closing_the_current_one_clears_it() {
        let mut registry = Registry::new();
        let _ = registry.open(entry("a"));
        assert!(registry.close(token("a")));
        assert_eq!(registry.current(), None);
    }

    #[test]
    fn closing_a_replaced_one_leaves_the_current_one() {
        let mut registry = Registry::new();
        let _ = registry.open(entry("a"));
        let _ = registry.open(entry("b"));
        assert!(!registry.close(token("a")));
        assert_eq!(registry.current(), Some("b"));
    }

    #[test]
    fn a_second_open_is_refused_while_a_non_dismissable_one_is_open() {
        let mut registry = Registry::new();
        let _ = registry.open(required("a"));
        let refused = registry.open(entry("b"));
        assert!(matches!(refused, Err(Entry { id: "b", .. })));
        assert_eq!(registry.current(), Some("a"));
    }

    #[test]
    fn a_non_dismissable_one_still_replaces_a_dismissable_one() {
        let mut registry = Registry::new();
        let _ = registry.open(entry("a"));
        assert!(matches!(registry.open(required("b")), Ok(Some("a"))));
        assert_eq!(registry.current(), Some("b"));
    }

    #[test]
    fn a_non_dismissable_one_is_refused_while_another_non_dismissable_one_is_open() {
        let mut registry = Registry::new();
        let _ = registry.open(required("a"));
        let refused = registry.open(required("b"));
        assert!(matches!(refused, Err(Entry { id: "b", .. })));
        assert_eq!(registry.current(), Some("a"));
    }

    #[test]
    fn closing_a_refused_one_leaves_the_non_dismissable_one_current() {
        let mut registry = Registry::new();
        let _ = registry.open(required("a"));
        let _ = registry.open(entry("b"));
        assert!(!registry.close(token("b")));
        assert_eq!(registry.current(), Some("a"));
    }

    #[test]
    fn once_the_non_dismissable_one_closes_the_next_open_goes_through() {
        let mut registry = Registry::new();
        let _ = registry.open(required("a"));
        let _ = registry.open(entry("b"));
        assert!(registry.close(token("a")));
        assert!(matches!(registry.open(entry("b")), Ok(None)));
        assert_eq!(registry.current(), Some("b"));
    }
}

/// The glue natively, on the recording page: what each case does to focus
/// and scroll, beyond the registry's decision.
#[cfg(all(test, not(feature = "browser")))]
mod glue_tests {
    use leptos::prelude::*;

    use super::fake::{self, Moves, focus, moves, settle};
    use super::manage;
    use crate::button::Disclosed;

    /// A managed container `id`, closed.
    fn container(id: &'static str, dismissable: bool) -> RwSignal<bool> {
        let open = RwSignal::new(false);
        manage(Disclosed::new(id, open), dismissable);
        open
    }

    #[test]
    fn a_refused_open_closes_itself_and_moves_nothing() {
        fake::run(async || {
            let a = container("a", false);
            let b = container("b", true);
            settle().await;
            focus("opener");
            a.set(true);
            settle().await;
            b.set(true);
            settle().await;
            assert!(a.get_untracked(), "A stays open");
            assert!(!b.get_untracked(), "B closed itself");
            let Moves {
                focused,
                locked,
                locks,
                unlocks,
                focus_into,
                restores,
            } = moves();
            assert_eq!(focused.as_deref(), Some("a"), "focus stays in A");
            assert!(locked);
            assert_eq!((locks, unlocks), (1, 0), "B never locked nor unlocked");
            assert_eq!(focus_into, ["a"]);
            assert!(restores.is_empty(), "B gave no focus back: {restores:?}");
        });
    }

    #[test]
    fn a_replaced_one_hands_over_without_restoring_or_unlocking() {
        fake::run(async || {
            let a = container("a", true);
            let b = container("b", true);
            settle().await;
            a.set(true);
            settle().await;
            b.set(true);
            settle().await;
            assert!(!a.get_untracked(), "A replaced");
            assert!(b.get_untracked(), "B open");
            let m = moves();
            assert_eq!(m.focus_into, ["a", "b"], "one focus move each");
            assert!(m.locked);
            assert_eq!(m.unlocks, 0, "A's close did not unlock");
            assert!(m.restores.is_empty(), "A's close did not restore");
        });
    }

    #[test]
    fn once_the_required_one_closes_itself_the_next_open_goes_through() {
        fake::run(async || {
            let a = container("a", false);
            let b = container("b", true);
            settle().await;
            focus("opener");
            a.set(true);
            settle().await;
            b.set(true);
            settle().await;
            a.set(false);
            settle().await;
            let m = moves();
            assert_eq!(
                m.restores,
                [Some("opener".to_owned())],
                "A gives focus back"
            );
            assert_eq!((m.locked, m.unlocks), (false, 1), "A unlocks scroll");
            b.set(true);
            settle().await;
            assert!(b.get_untracked(), "B opens now");
            let m = moves();
            assert_eq!(m.focus_into, ["a", "b"]);
            assert!(m.locked);
        });
    }

    /// A managed container `id` built under its own owner, returned with
    /// that owner so a test can dispose it, as an unmounted view does.
    fn owned_container(id: &'static str, dismissable: bool) -> (Owner, RwSignal<bool>) {
        let open = RwSignal::new(false);
        let owner = Owner::new();
        owner.with(|| manage(Disclosed::new(id, open), dismissable));
        (owner, open)
    }

    #[test]
    fn a_disposed_required_one_no_longer_refuses_the_next_open() {
        fake::run(async || {
            let (a_owner, a) = owned_container("a", false);
            let b = container("b", true);
            settle().await;
            a.set(true);
            settle().await;
            a_owner.cleanup();
            settle().await;
            b.set(true);
            settle().await;
            assert!(b.get_untracked(), "B opens: A left the registry");
            assert_eq!(moves().focus_into, ["a", "b"]);
        });
    }

    #[test]
    fn a_disposed_open_one_unlocks_the_scroll_and_gives_focus_back() {
        fake::run(async || {
            let (owner, a) = owned_container("a", true);
            settle().await;
            focus("opener");
            a.set(true);
            settle().await;
            assert!(moves().locked);
            owner.cleanup();
            settle().await;
            let m = moves();
            assert_eq!((m.locked, m.unlocks), (false, 1), "scroll unlocked once");
            assert_eq!(m.restores, [Some("opener".to_owned())]);
        });
    }

    #[test]
    fn a_disposed_closed_one_moves_nothing() {
        fake::run(async || {
            let (owner, _a) = owned_container("a", true);
            settle().await;
            owner.cleanup();
            settle().await;
            assert_eq!(moves(), Moves::default());
        });
    }

    #[test]
    fn a_disposed_one_is_not_closed_by_the_next_open() {
        fake::run(async || {
            let (a_owner, a) = owned_container("a", true);
            let b = container("b", true);
            settle().await;
            a.set(true);
            settle().await;
            a_owner.cleanup();
            settle().await;
            let before = moves();
            b.set(true);
            settle().await;
            assert!(b.get_untracked());
            let m = moves();
            assert_eq!(m.unlocks, before.unlocks, "no second unlock for A");
            assert_eq!(m.restores, before.restores, "no second restore for A");
        });
    }

    #[test]
    fn an_older_disposed_one_with_the_same_id_leaves_the_live_one_current() {
        fake::run(async || {
            // A component remounts under the same id: the old instance is
            // disposed after the new one became current.
            let (old_owner, _old) = owned_container("same", true);
            let (_new_owner, new) = owned_container("same", true);
            settle().await;
            focus("opener");
            new.set(true);
            settle().await;
            old_owner.cleanup();
            settle().await;
            let m = moves();
            assert!(m.locked, "the live one keeps the scroll locked");
            assert_eq!(m.unlocks, 0, "the old one unlocked nothing");
            assert!(m.restores.is_empty(), "the old one restored nothing");
        });
    }

    #[test]
    fn the_old_one_disposed_before_the_new_one_opens_still_clears_the_registry() {
        fake::run(async || {
            let (old_owner, old) = owned_container("same", false);
            let (_new_owner, new) = owned_container("same", true);
            settle().await;
            old.set(true);
            settle().await;
            old_owner.cleanup();
            settle().await;
            new.set(true);
            settle().await;
            assert!(new.get_untracked(), "the new one opens: no stale entry");
        });
    }
}
