//! `RevealCard`: a card with a front, a reveal shown in place and pinned
//! actions, knowing nothing of grading or learning.
//!
//! The caller gives the three slots (the front, the reveal, the actions),
//! a `key` that names the presentation and the label of the "Show" button;
//! the card owns when the reveal is shown, which keys reveal it and where
//! the focus goes.
//!
//! # The key contract
//!
//! The card remembers **the key that was revealed**; `revealed` is derived:
//! "the remembered key equals the current `key`". No effect resets it, so a
//! new key reads unrevealed in the very same tick (a grade key pressed
//! right after the change cannot land on an unrevealed card). The
//! consequence is the contract: **a key is unique per presentation**; the
//! caller puts a presentation counter in it (term + facet + counter).
//! Equal keys keep their reveal: a batched A, B, A signals as no change and
//! keeps card A revealed.
//!
//! # Height and keys of the caller
//!
//! The caller gives the card a **definite height** (`h-[calc(100dvh-…)]`
//! or a fixed box): the wrapper is `h-full`, and a percentage height under a
//! parent that only has a `min-height` is auto, so the body would grow
//! instead of scrolling under the pinned actions.
//!
//! The caller binds its own keys (the grades) on an **ancestor** of the
//! card, never on the wrapper (the card exposes no handle to it): the
//! wrapper's `keydown` bubbles, and the caller reads `revealed` (the
//! signal given to `actions`) in its handler.
//!
//! The caller's keys are **never Space or Enter** (those are the card's,
//! and on a focused grade button they are its native click): letters and
//! arrows. A target the card could not classify (the browser refused a
//! selector, `TargetKind::Unmatched`) is treated as interactive, so the
//! card never steals a key it cannot read.
//!
//! # Keys and focus
//!
//! The wrapper owns Space and Enter while the card is unrevealed, unless
//! the event comes from an editable or interactive target (the native
//! click of a focused button wins) or is a repeat; after the reveal they do
//! nothing and native Space scrolling returns. The un-hide moves the focus
//! into the reveal (`tabindex="-1"`, a visible ring; no `aria-live`: the
//! focus move is the announcement), and a key change moves it back to the
//! wrapper (`tabindex="-1"`, no ring), neither on the first mount.
//!
//! # Tested where
//!
//! The rules are pure functions (`key_effect`, `focus_move`) and the
//! state is real signals, tested natively. The thin glue that needs a
//! browser (`target_kind`, the focus effect) is compiled for `hydrate` only,
//! with a no-op stub otherwise, checked by clippy for `wasm32` and by the
//! gallery; it has no native test.

use leptos::ev::KeyboardEvent;
use leptos::{html, prelude::*};

use crate::button::{Button, Variant};

/// The wrapper: a flex column filling the caller's height, a focus target
/// (not a tab stop) with no ring, capped at 40rem from `sm` up.
const WRAPPER: &str = "flex h-full w-full flex-col outline-none sm:mx-auto sm:max-w-[40rem]";
/// The body: its own scroll region, without a tab stop of its own.
const BODY: &str = "min-h-0 flex-1 overflow-y-auto";
/// The reveal region: focused by script, with a visible ring.
const REGION: &str =
    "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold";
/// The actions: in the flow at the bottom, above the safe-area inset.
const ACTIONS: &str = "shrink-0 pb-[env(safe-area-inset-bottom)]";

/// A card: `front`, then a `reveal` shown in place once revealed, with the
/// `actions` pinned at the bottom of the card. See the module
/// documentation for the contract of `key` and the keys the card owns.
///
/// The caller gives the card a **definite height** (`h-[calc(100dvh-…)]`
/// or a fixed box; a percentage height under a `min-height`-only parent is
/// auto and the body would grow instead of scrolling), and binds its own
/// keys on an **ancestor** of the card (the wrapper's `keydown` bubbles),
/// reading `revealed` in its handler.
///
/// The caller's keys are never Space or Enter (the card's; they would
/// activate a focused button): letters and arrows. A target the classifier
/// could not match (`TargetKind::Unmatched`) is treated as interactive.
#[component]
pub fn RevealCard(
    /// What the card shows first; stays after the reveal.
    #[prop(into)]
    front: ViewFn,
    /// What the reveal shows; rendered `hidden` until revealed.
    #[prop(into)]
    reveal: ViewFn,
    /// The actions at the bottom (the caller's grade buttons). Called once
    /// with a read-only `revealed`; the caller greys or hides its buttons
    /// from it. The view is always rendered.
    actions: impl Fn(Signal<bool>) -> AnyView + 'static,
    /// Names the presentation: unique per presentation, a new key resets
    /// the card to unrevealed in the same tick.
    #[prop(into)]
    key: Signal<String>,
    /// The label of the button that reveals, the caller's string.
    #[prop(into)]
    reveal_label: String,
    /// Called once per reveal: a side effect only, never the state channel.
    #[prop(optional, into)]
    on_reveal: Option<Callback<()>>,
) -> impl IntoView {
    let state = Reveal::new(key, on_reveal);
    let actions = actions(state.revealed.into());
    card_markup(state, front, reveal, actions, reveal_label)
}

/// The card's state: the key that was revealed, and `revealed` derived
/// from it (stored key equals current key), never reset by an effect.
#[derive(Clone, Copy)]
struct Reveal {
    stored: RwSignal<Option<String>>,
    key: Signal<String>,
    revealed: Memo<bool>,
    notify: Option<Callback<()>>,
}

impl Reveal {
    fn new(key: Signal<String>, on_reveal: Option<Callback<()>>) -> Self {
        let stored = RwSignal::new(None::<String>);
        let revealed = Memo::new(move |_| stored.get().as_deref() == Some(key.get().as_str()));
        Self {
            stored,
            key,
            revealed,
            notify: on_reveal,
        }
    }

    /// Reveals the current key; `on_reveal` runs only if it was not revealed.
    fn reveal(&self) {
        if self.revealed.get_untracked() {
            return;
        }
        self.stored.set(Some(self.key.get_untracked()));
        if let Some(notify) = self.notify {
            notify.run(());
        }
    }
}

/// The markup, given the state and the slots.
fn card_markup(
    state: Reveal,
    front: ViewFn,
    reveal: ViewFn,
    actions: AnyView,
    label: String,
) -> impl IntoView {
    let wrapper = NodeRef::<html::Div>::new();
    let region = NodeRef::<html::Div>::new();
    wire_focus(state, wrapper, region);
    let revealed = state.revealed;
    view! {
        <div node_ref=wrapper tabindex="-1" class=WRAPPER on:keydown=move |ev| keydown(&ev, state)>
            <div class=BODY>
                {front.run()} {move || (!revealed.get()).then(|| show_button(state, label.clone()))}
                <div node_ref=region hidden=move || !revealed.get() tabindex="-1" class=REGION>
                    {reveal.run()}
                </div>
            </div>
            <div class=ACTIONS>{actions}</div>
        </div>
    }
}

/// The visible button that reveals, for touch (and the keyboard's Tab).
fn show_button(state: Reveal, label: String) -> impl IntoView {
    view! {
        <Button variant=Variant::Primary layout="m-4" on_click=move |_| state.reveal()>
            {label}
        </Button>
    }
}

fn keydown(ev: &KeyboardEvent, state: Reveal) {
    let effect = key_effect(
        &ev.key(),
        ev.repeat(),
        target_kind(ev),
        state.revealed.get_untracked(),
    );
    if effect == KeyEffect::Reveal {
        ev.prevent_default();
        state.reveal();
    }
}

// --- The key rule --------------------------------------------------------

/// What a keydown on the card does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyEffect {
    /// Reveals the card and prevents the default.
    Reveal,
    /// Leaves the event alone.
    Ignore,
}

/// What the event's target is, as far as the card cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "browser"), allow(dead_code))]
enum TargetKind {
    /// Anything that does not use Space or Enter itself.
    Plain,
    /// An `<input>`.
    Input,
    /// A `<textarea>`.
    Textarea,
    /// A `<select>`.
    Select,
    /// A `contenteditable` element.
    Editable,
    /// A `<button>`: its native click wins.
    Button,
    /// An `<a>`: its native activation wins.
    Link,
    /// The browser could not match a selector against the target: treated
    /// as interactive, so the card never steals a key it cannot classify.
    Unmatched,
}

/// Space and Enter reveal (and the default is prevented) only while
/// unrevealed, from a plain target, and not as a repeat; anything else is
/// left alone, so after the reveal native Space scrolling returns.
fn key_effect(key: &str, repeat: bool, target: TargetKind, revealed: bool) -> KeyEffect {
    let reveals = (key == " " || key == "Enter") && !repeat && !revealed;
    if reveals && target == TargetKind::Plain {
        KeyEffect::Reveal
    } else {
        KeyEffect::Ignore
    }
}

/// Classifies the keydown's target (hydrate only: it needs the DOM; not
/// tested natively, the gallery shows it).
#[cfg(feature = "browser")]
fn target_kind(ev: &KeyboardEvent) -> TargetKind {
    use wasm_bindgen::JsCast;
    const KINDS: [(&str, TargetKind); 6] = [
        ("input", TargetKind::Input),
        ("textarea", TargetKind::Textarea),
        ("select", TargetKind::Select),
        (
            "[contenteditable]:not([contenteditable='false'])",
            TargetKind::Editable,
        ),
        ("button", TargetKind::Button),
        ("a", TargetKind::Link),
    ];
    let Some(element) = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    else {
        return TargetKind::Plain;
    };
    // Review round 1 ruling: a selector the browser rejects (`Err`) must
    // not let the card steal Space or Enter from a focused button or
    // input, so it classifies the target as `Unmatched` (interactive),
    // never `Plain`. The selectors are constants, so this is unreachable
    // in practice.
    for (selector, kind) in KINDS {
        match element.matches(selector) {
            Ok(true) => return kind,
            Ok(false) => {}
            Err(_) => return TargetKind::Unmatched,
        }
    }
    TargetKind::Plain
}

#[cfg(not(feature = "browser"))]
fn target_kind(_ev: &KeyboardEvent) -> TargetKind {
    TargetKind::Plain
}

// --- The focus rule ------------------------------------------------------

/// What the focus rule looks at: the key, and whether it is revealed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "browser"), allow(dead_code))]
struct CardState {
    key: String,
    revealed: bool,
}

/// Where the focus goes after a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "browser"), allow(dead_code))]
enum FocusMove {
    /// Nowhere.
    None,
    /// Into the reveal region.
    Reveal,
    /// Back to the card's wrapper.
    Wrapper,
}

/// The first run moves nothing (the hydration pass is unrevealed and the
/// card must not steal the focus on mount); an un-hide focuses the reveal;
/// a key change focuses the wrapper.
#[cfg_attr(not(feature = "browser"), allow(dead_code))]
fn focus_move(previous: Option<&CardState>, now: &CardState) -> FocusMove {
    let Some(previous) = previous else {
        return FocusMove::None;
    };
    if now.revealed && !previous.revealed {
        FocusMove::Reveal
    } else if now.key != previous.key {
        FocusMove::Wrapper
    } else {
        FocusMove::None
    }
}

/// Runs the focus rule after every change of the key or of `revealed`
/// (hydrate only: `focus()` needs the DOM, and an effect runs after the
/// un-hide has rendered; not tested natively, the gallery shows it).
#[cfg(feature = "browser")]
fn wire_focus(state: Reveal, wrapper: NodeRef<html::Div>, region: NodeRef<html::Div>) {
    Effect::new(move |previous: Option<CardState>| {
        let now = CardState {
            key: state.key.get(),
            revealed: state.revealed.get(),
        };
        match focus_move(previous.as_ref(), &now) {
            FocusMove::None => {}
            FocusMove::Reveal => focus(region),
            FocusMove::Wrapper => focus(wrapper),
        }
        now
    });
}

#[cfg(feature = "browser")]
fn focus(node: NodeRef<html::Div>) {
    if let Some(element) = node.get_untracked() {
        let _ = element.focus();
    }
}

#[cfg(not(feature = "browser"))]
fn wire_focus(_state: Reveal, _wrapper: NodeRef<html::Div>, _region: NodeRef<html::Div>) {}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    const SPACE: &str = " ";

    /// A card with recognisable slots; `actions` writes what it was given.
    fn rendered(key: &'static str) -> String {
        Owner::new().with(|| {
            let actions = |revealed: Signal<bool>| {
                view! { <span>{move || format!("got:{}", revealed.get())}</span> }.into_any()
            };
            view! {
                <RevealCard
                    front=|| "FRONT"
                    reveal=|| "REVEAL"
                    actions=actions
                    key=key
                    reveal_label="Show the answer"
                    on_reveal=|()| ()
                />
            }
            .to_html()
        })
    }

    /// The opening tag right before `needle`.
    fn tag_before<'a>(html: &'a str, needle: &str) -> &'a str {
        let at = html
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} in {html}"));
        let start = html[..at].rfind('<').expect("a tag before");
        &html[start..at]
    }

    /// Whether the opening tag carries the boolean attribute `name`.
    fn has_attribute(tag: &str, name: &str) -> bool {
        tag.trim_end_matches('>')
            .split_whitespace()
            .any(|word| word == name)
    }

    #[test]
    fn the_wrapper_is_a_focus_target_without_a_ring_and_caps_its_width() {
        let html = rendered("a");
        let wrapper = &html[..html.find('>').unwrap()];
        assert!(wrapper.starts_with("<div"), "{html}");
        assert!(wrapper.contains("tabindex=\"-1\""), "{wrapper}");
        assert!(wrapper.contains("outline-none"), "{wrapper}");
        assert!(wrapper.contains("flex-col"), "{wrapper}");
        assert!(wrapper.contains("sm:max-w-[40rem]"), "{wrapper}");
        assert!(wrapper.contains("sm:mx-auto"), "{wrapper}");
    }

    #[test]
    fn the_body_scrolls_by_itself_and_is_not_a_tab_stop() {
        let html = rendered("a");
        let body = tag_before(&html, "FRONT");
        assert!(body.contains("overflow-y-auto"), "{body}");
        assert!(body.contains("min-h-0"), "{body}");
        assert!(!body.contains("tabindex"), "{body}");
    }

    #[test]
    fn the_front_is_there_and_the_reveal_is_hidden_and_focusable() {
        let html = rendered("a");
        assert!(html.contains("FRONT"), "{html}");
        let region = tag_before(&html, "REVEAL");
        assert!(has_attribute(region, "hidden"), "{region}");
        assert!(region.contains("tabindex=\"-1\""), "{region}");
        assert!(region.contains("focus-visible:outline-gold"), "{region}");
        assert_eq!(html.matches("aria-live").count(), 0, "{html}");
    }

    #[test]
    fn the_show_button_carries_the_callers_label_in_the_primary_look() {
        let html = rendered("a");
        let button = tag_before(&html, "Show the answer");
        assert!(button.starts_with("<button"), "{button}");
        assert!(button.contains("bg-primary text-on-primary"), "{button}");
        assert!(html.contains(">Show the answer</button>"), "{html}");
    }

    #[test]
    fn the_actions_are_rendered_once_and_told_it_is_not_revealed() {
        let html = rendered("a");
        assert_eq!(html.matches("got:").count(), 1, "{html}");
        assert!(html.contains("got:false"), "{html}");
        let wrapper = tag_before(&html, "<span>got:");
        assert!(
            wrapper.contains("pb-[env(safe-area-inset-bottom)]"),
            "{wrapper}"
        );
    }

    #[test]
    fn the_actions_come_after_the_body_in_the_flow() {
        let html = rendered("a");
        assert!(
            html.find("REVEAL").unwrap() < html.find("got:").unwrap(),
            "{html}"
        );
    }

    // --- the state, through real signals ---------------------------------

    fn state(key: &'static str) -> (RwSignal<String>, Reveal) {
        let key = RwSignal::new(key.to_owned());
        (key, Reveal::new(key.into(), None))
    }

    #[test]
    fn revealing_turns_revealed_on() {
        Owner::new().with(|| {
            let (_, reveal) = state("A");
            assert!(!reveal.revealed.get_untracked());
            reveal.reveal();
            assert!(reveal.revealed.get_untracked());
        });
    }

    #[test]
    fn a_different_key_reads_unrevealed_in_the_same_tick() {
        Owner::new().with(|| {
            let (key, reveal) = state("A");
            reveal.reveal();
            key.set("B".to_owned());
            assert!(!reveal.revealed.get_untracked());
        });
    }

    #[test]
    fn the_same_key_keeps_its_reveal() {
        Owner::new().with(|| {
            let (key, reveal) = state("A");
            reveal.reveal();
            key.set("A".to_owned());
            assert!(reveal.revealed.get_untracked());
        });
    }

    #[test]
    fn a_batched_a_b_a_keeps_the_reveal_of_a() {
        // The documented contract: A to B to A with no read in between
        // signals as no change, so the card stays revealed; a caller who
        // wants a fresh card puts a presentation counter in the key.
        Owner::new().with(|| {
            let (key, reveal) = state("A");
            reveal.reveal();
            key.set("B".to_owned());
            key.set("A".to_owned());
            assert!(reveal.revealed.get_untracked());
        });
    }

    #[test]
    fn an_unbatched_a_b_a_keeps_a_revealed_and_on_reveal_is_not_called_again() {
        // The contract (a key is unique per presentation), with a read
        // between the sets: the card reads unrevealed on B, revealed again
        // on A with no new `reveal()`, so `on_reveal` stays at one call.
        Owner::new().with(|| {
            let count = RwSignal::new(0_u32);
            let key = RwSignal::new("A".to_owned());
            let on_reveal = Callback::new(move |()| count.update(|c| *c += 1));
            let reveal = Reveal::new(key.into(), Some(on_reveal));
            reveal.reveal();
            key.set("B".to_owned());
            assert!(!reveal.revealed.get_untracked());
            key.set("A".to_owned());
            assert!(reveal.revealed.get_untracked());
            assert_eq!(count.get_untracked(), 1);
        });
    }

    #[test]
    fn revealing_without_an_on_reveal_works_and_does_not_panic() {
        Owner::new().with(|| {
            let key = RwSignal::new("A".to_owned());
            let reveal = Reveal::new(key.into(), None);
            reveal.reveal();
            reveal.reveal();
            assert!(reveal.revealed.get_untracked());
        });
    }

    #[test]
    fn a_new_presentation_key_is_not_revealed() {
        Owner::new().with(|| {
            let (key, reveal) = state("A#1");
            reveal.reveal();
            key.set("A#2".to_owned());
            assert!(!reveal.revealed.get_untracked());
        });
    }

    #[test]
    fn on_reveal_runs_once_per_reveal() {
        Owner::new().with(|| {
            let count = RwSignal::new(0_u32);
            let key = RwSignal::new("A".to_owned());
            let on_reveal = Callback::new(move |()| count.update(|c| *c += 1));
            let reveal = Reveal::new(key.into(), Some(on_reveal));
            reveal.reveal();
            reveal.reveal();
            assert_eq!(count.get_untracked(), 1);
            key.set("B".to_owned());
            reveal.reveal();
            assert_eq!(count.get_untracked(), 2);
        });
    }

    #[test]
    fn once_revealed_the_region_shows_the_button_goes_and_the_front_stays() {
        let html = Owner::new().with(|| {
            let (_, reveal) = state("A");
            reveal.reveal();
            card_markup(
                reveal,
                ViewFn::from(|| "FRONT"),
                ViewFn::from(|| "REVEAL"),
                ().into_any(),
                "Show the answer".to_owned(),
            )
            .to_html()
        });
        assert!(html.contains("FRONT"), "{html}");
        assert_eq!(html.matches("Show the answer").count(), 0, "{html}");
        let region = tag_before(&html, "REVEAL");
        assert!(!has_attribute(region, "hidden"), "{region}");
    }

    // --- the key rule ----------------------------------------------------

    use TargetKind::{Button, Editable, Input, Link, Plain, Select, Textarea, Unmatched};

    #[test]
    fn space_and_enter_reveal_an_unrevealed_card_from_a_plain_target() {
        for key in [SPACE, "Enter"] {
            assert_eq!(
                key_effect(key, false, Plain, false),
                KeyEffect::Reveal,
                "{key:?}"
            );
        }
    }

    #[test]
    fn nothing_reveals_from_an_interactive_target() {
        for target in [Input, Textarea, Select, Editable, Button, Link, Unmatched] {
            for key in [SPACE, "Enter"] {
                assert_eq!(
                    key_effect(key, false, target, false),
                    KeyEffect::Ignore,
                    "{key:?} on {target:?}"
                );
            }
        }
    }

    #[test]
    fn nothing_reveals_after_the_reveal_so_native_scrolling_returns() {
        for key in [SPACE, "Enter"] {
            assert_eq!(key_effect(key, false, Plain, true), KeyEffect::Ignore);
        }
    }

    #[test]
    fn a_repeat_does_not_reveal() {
        for key in [SPACE, "Enter"] {
            assert_eq!(key_effect(key, true, Plain, false), KeyEffect::Ignore);
        }
    }

    #[test]
    fn a_repeat_after_the_reveal_is_left_alone_too() {
        for key in [SPACE, "Enter"] {
            assert_eq!(key_effect(key, true, Plain, true), KeyEffect::Ignore);
        }
    }

    #[test]
    fn other_keys_are_not_the_cards() {
        for key in ["a", "1", "Escape", "ArrowDown", "PageDown", "Tab", "Shift"] {
            assert_eq!(
                key_effect(key, false, Plain, false),
                KeyEffect::Ignore,
                "{key}"
            );
        }
    }

    // --- the focus rule --------------------------------------------------

    fn card(key: &str, revealed: bool) -> CardState {
        CardState {
            key: key.to_owned(),
            revealed,
        }
    }

    #[test]
    fn the_first_run_moves_nothing_even_when_already_revealed() {
        assert_eq!(focus_move(None, &card("A", false)), FocusMove::None);
        assert_eq!(focus_move(None, &card("A", true)), FocusMove::None);
    }

    #[test]
    fn the_unhide_focuses_the_reveal() {
        let before = card("A", false);
        assert_eq!(
            focus_move(Some(&before), &card("A", true)),
            FocusMove::Reveal
        );
    }

    #[test]
    fn a_key_change_focuses_the_wrapper() {
        let before = card("A", true);
        assert_eq!(
            focus_move(Some(&before), &card("B", false)),
            FocusMove::Wrapper
        );
        let before = card("A", false);
        assert_eq!(
            focus_move(Some(&before), &card("B", false)),
            FocusMove::Wrapper
        );
    }

    #[test]
    fn the_reveal_wins_when_a_key_change_also_flips_revealed() {
        // Cannot happen under the contract (a new key is never the stored
        // one, so a key change never reads revealed); the order is written
        // down all the same: the reveal branch is tested first.
        let before = card("A", false);
        assert_eq!(
            focus_move(Some(&before), &card("B", true)),
            FocusMove::Reveal
        );
    }

    #[test]
    fn nothing_changed_moves_nothing() {
        let before = card("A", true);
        assert_eq!(focus_move(Some(&before), &card("A", true)), FocusMove::None);
        let before = card("A", false);
        assert_eq!(
            focus_move(Some(&before), &card("A", false)),
            FocusMove::None
        );
    }
}
