//! `PaliText`: a Pāli passage whose words open their meaning.
//!
//! [`PaliText`] renders a passage (as [`Pali`] does, at a [`PaliSize`])
//! where every word the caller has entries for is a button; pressing it
//! opens those entries. It knows no dictionary: the caller gives the
//! entries, keyed by the word as written, lower-cased. A word without
//! entries, punctuation and numbers stay plain text.
//!
//! - **Words** ([`words`]): runs of letters, Pāli diacritics included
//!   (precomposed or combining), and the apostrophe of sandhi between two
//!   letters (`sampādethā'ti` is one word); everything else is plain.
//! - **Desktop** (`sm` and up): a [`Popover`] fixed in the viewport, so a
//!   box that scrolls around the passage neither clips it nor scrolls to
//!   show it. When it opens, it is measured and placed 8 px under the
//!   pressed word (above it when below lacks room and above has more),
//!   kept 16 px inside the viewport's sides; any scroll (the page's or a
//!   box's around the passage) or a resize closes it. It shows the form
//!   pressed in bold, then each entry, its headword in bold, its tag
//!   muted, its meaning under them; at most three entries, then
//!   "+ n more".
//!
//!   A fixed element is placed against the viewport unless one of its
//!   ancestors has a `transform`, a `filter` or a `perspective`: that
//!   ancestor then holds it as an `absolute` one would be, and the panel
//!   lands off its word. Keep such styles off a passage's ancestors.
//! - **Phone**: the same panel as a bottom sheet (fixed to the bottom,
//!   full width, rounded top) over a dimmed backdrop, with a close button;
//!   a tap on the backdrop closes it.
//! - **An action under each entry** (`entry_action`, optional): the caller
//!   gives an [`EntryAction`], called for every shown entry with the form
//!   pressed and the [`WordEntry`]; the view it returns is rendered under
//!   the entry's meaning, in the popover and the phone's sheet alike (a
//!   button, a status line). ui-lib owns no string and no rule there; the
//!   slot adds a top margin and no colour. Without it the panel's markup is
//!   what it was before the prop (a test pins it).
//! - **One word open at a time**: pressing another word moves the panel
//!   there, pressing the open word again closes it. Escape closes it, as
//!   any [`Popover`]: its window listeners (and, on desktop, the scroll
//!   and resize ones) live as long as the open panel and are removed with
//!   it.
//!
//! The word button sets state only (`aria-expanded`, `aria-controls`); its
//! look is one class set, `WORD`, whose underline reacts to that state.

use std::collections::HashMap;

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Button, Disclosure, Variant};
use crate::catalogue::strings;
use crate::icon::{Icon, IconName};
use crate::pali::{Pali, PaliSize};
use crate::popover::Popover;

/// One meaning of a word, as a dictionary gives it: the headword it comes
/// from, an optional tag (a part of speech, a grammar note) and the meaning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordEntry {
    /// The dictionary's headword (Pāli), shown in bold.
    pub headword: String,
    /// A short label after the headword, muted: `masc`, `pr`, `ind`…
    pub tag: Option<String>,
    /// What it means, in the caller's language.
    pub meaning: String,
}

/// What a service shows under each entry of the open panel: given the form
/// pressed (as written in the passage) and the entry, it returns the view.
/// ui-lib owns no string and no rule of what goes there (a button, a line of
/// text, nothing): see [`PaliText`]'s `entry_action`.
pub type EntryAction = Callback<(String, WordEntry), AnyView>;

/// A piece of a passage, as [`words`] cuts it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    /// A word: letters (diacritics included) and inner sandhi apostrophes.
    Word(String),
    /// Anything between words: spaces, punctuation, quotes, numbers.
    Other(String),
}

/// How many entries a panel shows before "+ n more".
const MAX_SHOWN: usize = 3;

/// Cuts `text` into words and what lies between them, in order; joined
/// back, the tokens give `text` unchanged. A word is a run of letters (any
/// alphabetic character, and the combining marks of a decomposed
/// diacritic) and of apostrophes (`'`, `’`) that stand between two letters.
#[must_use]
pub fn words(text: &str) -> Vec<Token> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    for (at, &c) in chars.iter().enumerate() {
        let next_is_letter = chars.get(at + 1).is_some_and(|&n| is_letter(n));
        let word_char = is_letter(c) || (in_word && is_apostrophe(c) && next_is_letter);
        if word_char != in_word && !current.is_empty() {
            tokens.push(token(in_word, std::mem::take(&mut current)));
        }
        in_word = word_char;
        current.push(c);
    }
    if !current.is_empty() {
        tokens.push(token(in_word, current));
    }
    tokens
}

/// A letter of a word: any alphabetic character (ā, ṃ, ñ…), or a
/// combining mark (U+0300–U+036F) of a decomposed diacritic.
fn is_letter(c: char) -> bool {
    c.is_alphabetic() || ('\u{0300}'..='\u{036f}').contains(&c)
}

/// An apostrophe that may join two letters in sandhi: straight or curly.
fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '\u{2019}')
}

/// A finished run: a word or what lies between words.
fn token(word: bool, text: String) -> Token {
    if word {
        Token::Word(text)
    } else {
        Token::Other(text)
    }
}

/// What [`PaliText`] renders: plain text, or a word with its entries.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    /// Text with no entries, consecutive pieces merged.
    Plain(String),
    /// A word with at least one entry.
    Word {
        /// The word as written.
        form: String,
        /// Its entries, never empty.
        entries: Vec<WordEntry>,
    },
}

/// The passage's segments: each word with entries (looked up lower-cased),
/// everything else merged into plain runs.
fn segments(
    tokens: &[Token],
    entries: &HashMap<String, Vec<WordEntry>>,
    every_word: bool,
) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    for token in tokens {
        let (text, found, is_word) = match token {
            Token::Word(form) => (form, entries.get(&form.to_lowercase()), true),
            Token::Other(text) => (text, None, false),
        };
        let found = found.filter(|list| !list.is_empty());
        let found = found.map(Vec::as_slice).or(if every_word && is_word {
            Some(&[][..])
        } else {
            None
        });
        match (found, out.last_mut()) {
            (Some(list), _) => out.push(Segment::Word {
                form: text.clone(),
                entries: list.to_vec(),
            }),
            (None, Some(Segment::Plain(plain))) => plain.push_str(text),
            (None, _) => out.push(Segment::Plain(text.clone())),
        }
    }
    out
}

/// The entries a panel shows (at most [`MAX_SHOWN`]) and how many more
/// there are.
fn shown(entries: &[WordEntry]) -> (&[WordEntry], usize) {
    let count = entries.len().min(MAX_SHOWN);
    (&entries[..count], entries.len() - count)
}

/// The "+ n more" line: the caller's label, or the English default.
fn more_text(label: Option<Callback<usize, String>>, more: usize) -> String {
    label.map_or_else(|| format!("+ {more} more"), |label| label.run(more))
}

/// The look of a word that opens its meaning, every state in one place: no
/// mark at rest; a dotted gold underline while hovered, keyboard-focused
/// (with the focus ring) or open. A real `<button>` reset by the page's
/// preflight (no border, background or padding; font and colour
/// inherited).
const WORD: &str = "cursor-pointer rounded-sm decoration-gold decoration-dotted \
    decoration-2 underline-offset-4 hover:underline focus-visible:underline \
    focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold \
    aria-expanded:underline";

/// The panel's placement: a bottom sheet below `sm` (fixed, full width,
/// square bottom corners, scrolling past 70% of the screen); from `sm`, a
/// popover fixed in the viewport, its top and left read from the custom
/// properties `--pc-word-top` and `--pc-word-left`, set when it opens
/// (unset until then: it sits where it would flow, and is placed before
/// the next frame is painted). Custom properties rather than an inline
/// `top`: [`Popover`] clears an inline `top` each time it opens.
const PANEL_LAYOUT: &str = "z-50 max-sm:fixed max-sm:inset-x-0 max-sm:bottom-0 \
    max-sm:max-h-[70vh] max-sm:overflow-y-auto max-sm:rounded-b-none \
    sm:fixed sm:top-(--pc-word-top) sm:left-(--pc-word-left) \
    sm:w-72 sm:max-w-[calc(100vw-2rem)]";

/// The custom property the desktop panel's `top` reads, in [`PANEL_LAYOUT`].
#[cfg(feature = "browser")]
const TOP_VAR: &str = "--pc-word-top";

/// The custom property the desktop panel's `left` reads, in [`PANEL_LAYOUT`].
#[cfg(feature = "browser")]
const LEFT_VAR: &str = "--pc-word-left";

/// The dimmed backdrop behind the sheet, on a phone only.
const BACKDROP: &str = "fixed inset-0 z-40 bg-black/40 sm:hidden";

/// The panel's content: its own text size and colour (whatever the passage
/// inherits), and room for the phone's home indicator.
const CONTENT: &str = "px-2 py-1 text-left text-sm leading-6 font-normal text-ink \
    max-sm:pb-[max(0.25rem,env(safe-area-inset-bottom))]";

/// Where the desktop panel goes: pure, so it is tested natively; the
/// browser feeds it measured boxes (`place_panel`).
#[cfg(any(test, feature = "browser"))]
mod placement {
    /// The gap between the word and the panel; also the least room kept
    /// between the panel and the viewport's top and bottom edges.
    const GAP_PX: f64 = 8.0;

    /// The least room kept between the panel and the viewport's sides.
    const SIDE_PX: f64 = 16.0;

    /// A box in the viewport, in CSS pixels: the pressed word's left, top
    /// and bottom edges.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(super) struct Rect {
        pub(super) left: f64,
        pub(super) top: f64,
        pub(super) bottom: f64,
    }

    /// A width and a height, in CSS pixels: the panel's, or the viewport's.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(super) struct Size {
        pub(super) width: f64,
        pub(super) height: f64,
    }

    /// Where the panel's top-left corner goes, in viewport pixels.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(super) struct Pos {
        pub(super) top: f64,
        pub(super) left: f64,
    }

    /// The panel's corner for a `word`, the panel's measured size and the
    /// viewport's: [`GAP_PX`] under the word; above it instead when the
    /// panel would run past the viewport's bottom there and the word has
    /// more room above than below (never above the viewport's top: at
    /// least [`GAP_PX`] down). Its left is the word's, clamped to stay
    /// [`SIDE_PX`] inside both sides; a panel wider than that room starts
    /// [`SIDE_PX`] in.
    pub(super) fn place(word: Rect, panel: Size, viewport: Size) -> Pos {
        let below = word.bottom + GAP_PX;
        let fits_below = below + panel.height <= viewport.height - GAP_PX;
        let more_above = word.top > viewport.height - word.bottom;
        let top = if fits_below || !more_above {
            below
        } else {
            (word.top - GAP_PX - panel.height).max(GAP_PX)
        };
        let left = word
            .left
            .min(viewport.width - panel.width - SIDE_PX)
            .max(SIDE_PX);
        Pos { top, left }
    }
}

/// Which word of a passage is open: the passage's one [`Disclosure`], so
/// its [`Popover`] closes it (Escape, a click outside) like any other.
#[derive(Clone, Copy, Debug)]
struct OpenWord {
    id: &'static str,
    word: RwSignal<Option<usize>>,
}

impl OpenWord {
    /// The open word's segment index, if any (tracked); none once disposed.
    fn current(self) -> Option<usize> {
        self.word.try_get().flatten()
    }
    /// Opens the word at `index`, or closes it when it is the open one.
    fn press(self, index: usize) {
        let _ = self.word.try_update(|open| {
            *open = if *open == Some(index) {
                None
            } else {
                Some(index)
            }
        });
    }
    /// Closes whichever word is open.
    fn close(self) {
        let _ = self.word.try_set(None);
    }
}

impl Disclosure for OpenWord {
    fn id(&self) -> &'static str {
        self.id
    }
    fn is_open(&self) -> bool {
        self.current().is_some()
    }
    /// Closing closes the open word; opening alone names no word, so it
    /// does nothing (a word opens itself, [`OpenWord::press`]).
    fn set_open(&self, open: bool) {
        if !open {
            self.close();
        }
    }
}

/// What every word of one passage shares.
#[derive(Clone, Copy)]
struct Ctx {
    open: OpenWord,
    size: PaliSize,
    more_label: Option<Callback<usize, String>>,
    entry_action: Option<EntryAction>,
    /// With a lookup every word is a button; opening one with no entry
    /// yet calls it with the form, so the caller can fetch and update the
    /// `entries` signal (learn-pali 065).
    lookup: Option<Callback<String>>,
    /// What the panel of a word with no entry says (looking up, or none).
    empty_label: Option<Signal<String>>,
}

/// A Pāli passage whose words open their meaning: each word with entries
/// is an inline button; pressing it shows its entries in a popover under
/// it, fixed in the viewport (a bottom sheet on a phone). See the
/// [module](self) for the rules.
///
/// The entries come as a signal of a map rather than a lookup callback:
/// whether a word is a button must be known for every word at the first
/// render, on the server as in the browser, and a map answers that at
/// once; as a signal, entries that arrive later (an API's answer) turn
/// their words into buttons in place.
// A component's prop is one concrete type: a hasher parameter would only
// make every caller name it.
#[allow(clippy::implicit_hasher)]
#[component]
pub fn PaliText(
    /// The panel's id, unique on the page: every word names it in
    /// `aria-controls`.
    id: &'static str,
    /// The passage, escaped like any text.
    #[prop(into)]
    text: String,
    /// Each word's entries, keyed by the word as written, lower-cased
    /// (`alaṃ`, `sampādethā'ti`); the caller normalises anything else
    /// (ṃ or ṁ, quotes). A word with no entry, or an empty list, stays
    /// plain text. A plain map or a signal.
    #[prop(into)]
    entries: Signal<HashMap<String, Vec<WordEntry>>>,
    /// The passage's size (and line height); [`PaliSize::Body`] by default.
    #[prop(optional)]
    size: PaliSize,
    /// The line under a capped list, given how many entries are hidden;
    /// `+ n more` (English) by default. Read inside a reactive closure: a
    /// label that reads a language signal follows it.
    #[prop(optional)]
    more_label: Option<Callback<usize, String>>,
    /// What to show under each entry, in the desktop popover and the phone's
    /// sheet alike: called with the form pressed (as written in the passage)
    /// and the entry, it returns the view (a button, a status line). The
    /// slot sets layout only (a small top margin around the view) and no
    /// colour or state: the view brings its own, from `ui-lib`'s components.
    /// The callback runs once per entry, untracked, each time the panel
    /// opens: a signal read in its body is read once. To follow a signal
    /// (a button that becomes an "Added" line), read it inside the returned
    /// view, in a closure. Absent by default: the panel
    /// is then exactly what it is without the prop.
    #[prop(optional)]
    entry_action: Option<EntryAction>,
    /// Layout classes only (margin, width, position), never colours.
    #[prop(optional, into)]
    layout: String,
    /// Makes every word a button, entries or not: opening a word that has
    /// none calls this with the form as written, once per opening; the
    /// caller looks it up and puts the entries into the `entries` signal,
    /// and the open panel shows them. Absent by default: a word with no
    /// entry then stays plain text.
    #[prop(optional)]
    lookup: Option<Callback<String>>,
    /// The line the panel of a word with no entry shows (`Looking up…`,
    /// then `No entry` once the caller knows), read live; only with
    /// `lookup`. Absent: an empty panel.
    #[prop(optional, into)]
    empty_label: Option<Signal<String>>,
) -> impl IntoView {
    passage(
        &text,
        entries,
        new_ctx(
            id,
            size,
            more_label,
            entry_action,
            None,
            lookup,
            empty_label,
        ),
        layout,
    )
}

/// What the component's props make of one passage's shared context, its open
/// word given (none when the component mounts; a test opens one, since a
/// click is a browser's).
fn new_ctx(
    id: &'static str,
    size: PaliSize,
    more_label: Option<Callback<usize, String>>,
    entry_action: Option<EntryAction>,
    open: Option<usize>,
    lookup: Option<Callback<String>>,
    empty_label: Option<Signal<String>>,
) -> Ctx {
    Ctx {
        open: OpenWord {
            id,
            word: RwSignal::new(open),
        },
        size,
        more_label,
        entry_action,
        lookup,
        empty_label,
    }
}

/// The passage itself, its open word given (so a test can render it open).
fn passage(
    text: &str,
    entries: Signal<HashMap<String, Vec<WordEntry>>>,
    ctx: Ctx,
    layout: String,
) -> impl IntoView + use<> {
    let tokens = words(text);
    view! {
        <span class=layout>
            {move || {
                entries
                    .with(|map| segments(&tokens, map, ctx.lookup.is_some()))
                    .into_iter()
                    .enumerate()
                    .map(|(index, segment)| segment_view(index, segment, ctx))
                    .collect_view()
            }}
        </span>
    }
}

/// One segment: plain Pāli text, or a word that opens its entries.
fn segment_view(index: usize, segment: Segment, ctx: Ctx) -> AnyView {
    match segment {
        Segment::Plain(text) => view! { <Pali size=ctx.size>{text}</Pali> }.into_any(),
        Segment::Word { form, entries } => {
            view! { <WordButton index=index form=form entries=entries ctx=ctx /> }.into_any()
        }
    }
}

/// A word with entries: an inline button (the panel's disclosure) and,
/// while it is the open word, its panel.
#[component]
fn WordButton(index: usize, form: String, entries: Vec<WordEntry>, ctx: Ctx) -> impl IntoView {
    let open = ctx.open;
    let is_open = move || open.current() == Some(index);
    let panel_form = form.clone();
    let word: NodeRef<leptos::html::Button> = NodeRef::new();
    let empty = entries.is_empty();
    let asked = form.clone();
    let on_press = move |_| {
        open.press(index);
        if empty
            && open.current() == Some(index)
            && let Some(lookup) = ctx.lookup
        {
            lookup.run(asked.clone());
        }
    };
    view! {
        <span>
            <Pali size=ctx.size>
                <button
                    node_ref=word
                    type="button"
                    class=WORD
                    aria-controls=open.id
                    aria-expanded=move || is_open().to_string()
                    on:click=on_press
                >
                    {form}
                </button>
            </Pali>
            {move || {
                is_open()
                    .then(|| {
                        view! {
                            <WordPanel
                                form=panel_form.clone()
                                entries=entries.clone()
                                ctx=ctx
                                word=word
                            />
                        }
                    })
            }}
        </span>
    }
}

/// The open word's panel: the backdrop (phone) and the popover or sheet,
/// mounted only while the word is open, so the popover's listeners (and
/// the desktop's scroll and resize ones) end with it. `word` is the
/// pressed button, the desktop panel's anchor.
#[component]
fn WordPanel(
    form: String,
    entries: Vec<WordEntry>,
    ctx: Ctx,
    word: NodeRef<leptos::html::Button>,
) -> impl IntoView {
    let open = ctx.open;
    place_on_desktop(open, word);
    let (list, more) = shown(&entries);
    let list = list.to_vec();
    let more_label = ctx.more_label;
    let action = ctx.entry_action;
    let pressed = form.clone();
    view! {
        <span class=BACKDROP aria-hidden="true" on:click=move |_| open.close()></span>
        <Popover controls=open aria_label=form.clone() layout=PANEL_LAYOUT>
            <div class=CONTENT>
                <div class="flex items-start justify-between gap-3">
                    <strong class="font-semibold text-heading">
                        <Pali>{form}</Pali>
                    </strong>
                    <span class="sm:hidden">
                        <CloseButton open=open />
                    </span>
                </div>
                <ul class="mt-1 space-y-2">{panel_items(list, action, &pressed, ctx.empty_label)}</ul>
                {(more > 0)
                    .then(|| {
                        view! {
                            <p class="mt-2 text-muted">{move || more_text(more_label, more)}</p>
                        }
                    })}
            </div>
        </Popover>
    }
}

/// The `sm` breakpoint (40rem), in CSS pixels: from it on, the panel is
/// the desktop popover rather than the sheet.
/// The panel's rows: one per entry; with none and a label (a lookup in
/// flight, or nothing found), one muted row saying it. A plain list, so a
/// panel without the label keeps the markup it always had.
fn panel_items(
    list: Vec<WordEntry>,
    action: Option<EntryAction>,
    pressed: &str,
    empty_label: Option<Signal<String>>,
) -> Vec<AnyView> {
    let mut items: Vec<AnyView> = list
        .into_iter()
        .map(|entry| entry_view(entry, action.map(|run| (run, pressed.to_string()))).into_any())
        .collect();
    if items.is_empty()
        && let Some(label) = empty_label
    {
        items.push(
            view! {
                <li class="text-muted" data-empty-entry="">
                    {move || label.get()}
                </li>
            }
            .into_any(),
        );
    }
    items
}

#[cfg(feature = "browser")]
const SM_PX: f64 = 640.0;

/// The browser only, on desktop: once the panel is laid out, place it by
/// its word (`place_panel`); close it on any scroll (captured, so a box
/// scrolling around the passage counts) or resize, since a fixed panel
/// would stay put while its word moves. The listeners end with the panel
/// (053). The phone's sheet is left as it is.
fn place_on_desktop(open: OpenWord, word: NodeRef<leptos::html::Button>) {
    #[cfg(feature = "browser")]
    {
        let wide = window()
            .inner_width()
            .ok()
            .and_then(|w| w.as_f64())
            .is_some_and(|w| w >= SM_PX);
        if wide {
            request_animation_frame(move || place_panel(open.id, word));
            close_on_scroll_or_resize(open);
        }
    }
    #[cfg(not(feature = "browser"))]
    let _ = (open, word);
}

/// Measures the pressed word and the panel `id`, and sets the panel's
/// [`TOP_VAR`] and [`LEFT_VAR`] from [`placement::place`] against the
/// visible viewport. Nothing when either is gone.
#[cfg(feature = "browser")]
fn place_panel(id: &str, word: NodeRef<leptos::html::Button>) {
    use placement::{Rect, Size, place};
    use wasm_bindgen::JsCast;
    let Some(button) = word.try_get_untracked().flatten() else {
        return;
    };
    let Some(panel) = document()
        .get_element_by_id(id)
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    let at = button.get_bounding_client_rect();
    let size = panel.get_bounding_client_rect();
    let (width, height) = crate::viewport::visible_size();
    let word = Rect {
        left: at.left(),
        top: at.top(),
        bottom: at.bottom(),
    };
    let panel_size = Size {
        width: size.width(),
        height: size.height(),
    };
    let pos = place(word, panel_size, Size { width, height });
    let style = panel.style();
    let _ = style.set_property(TOP_VAR, &format!("{}px", pos.top));
    let _ = style.set_property(LEFT_VAR, &format!("{}px", pos.left));
}

/// Closes the open word on a window `resize` and on any `scroll`, listened
/// to in the capture phase (an inner box's scroll does not bubble); both
/// listeners are removed when the panel's owner is cleaned up.
#[cfg(feature = "browser")]
fn close_on_scroll_or_resize(open: OpenWord) {
    use wasm_bindgen::{JsCast, closure::Closure};
    let resize = window_event_listener(leptos::ev::resize, move |_| open.close());
    let scroll = Closure::<dyn Fn()>::new(move || open.close()).into_js_value();
    let options = web_sys::AddEventListenerOptions::new();
    options.set_capture(true);
    options.set_passive(true);
    let _ = window().add_event_listener_with_callback_and_add_event_listener_options(
        "scroll",
        scroll.unchecked_ref(),
        &options,
    );
    // The JS function is not `Send`: kept in local storage, read at cleanup
    // (cleanups run before the owner's values are disposed).
    let scroll = StoredValue::new_local(scroll);
    on_cleanup(move || {
        resize.remove();
        scroll.try_with_value(|f| {
            let _ = window().remove_event_listener_with_callback_and_bool(
                "scroll",
                f.unchecked_ref(),
                true,
            );
        });
    });
}

/// One entry: the headword in bold and its tag, the meaning under them and,
/// when the caller gave an action (with the form pressed), its view.
fn entry_view(entry: WordEntry, action: Option<(EntryAction, String)>) -> AnyView {
    let slot = action.map(|(run, form)| run.run((form, entry.clone())));
    let text = view! {
        <p class="flex flex-wrap items-baseline gap-x-2">
            <strong class="font-semibold text-heading">
                <Pali>{entry.headword}</Pali>
            </strong>
            {entry.tag.map(|tag| view! { <span class="text-muted">{tag}</span> })}
        </p>
        <p>{entry.meaning}</p>
    };
    // Two branches, not an `Option` child: a `None` would still write a
    // placeholder node, and a panel without the slot must stay what it was.
    match slot {
        Some(slot) => view! { <li>{text} <div class="mt-1">{slot}</div></li> }.into_any(),
        None => view! { <li>{text}</li> }.into_any(),
    }
}

/// The sheet's close ✕, named `close` in the current language.
#[component]
fn CloseButton(open: OpenWord) -> impl IntoView {
    let t = strings();
    move || {
        view! {
            <Button
                variant=Variant::Icon
                label=td_string!(t.locale(), close)
                on_click=move |_| open.close()
            >
                <Icon name=IconName::Close />
            </Button>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::placement::{Pos, Rect, Size, place};
    use super::*;

    fn word(s: &str) -> Token {
        Token::Word(s.to_owned())
    }

    fn other(s: &str) -> Token {
        Token::Other(s.to_owned())
    }

    fn entry(headword: &str, tag: Option<&str>, meaning: &str) -> WordEntry {
        WordEntry {
            headword: headword.to_owned(),
            tag: tag.map(str::to_owned),
            meaning: meaning.to_owned(),
        }
    }

    #[test]
    fn diacritics_stay_inside_their_word() {
        for w in [
            "saṃ",
            "ārāma",
            "īsa",
            "ūna",
            "ñāṇa",
            "aṅga",
            "kaṭa",
            "daḍḍha",
            "maṇi",
            "aḷāra",
        ] {
            assert_eq!(words(w), vec![word(w)], "{w}");
        }
    }

    #[test]
    fn decomposed_diacritics_stay_inside_their_word() {
        let w = "sam\u{0323}vara";
        assert_eq!(words(w), vec![word(w)]);
    }

    #[test]
    fn a_sandhi_apostrophe_between_letters_is_part_of_the_word() {
        assert_eq!(words("sampādethā'ti"), vec![word("sampādethā'ti")]);
        assert_eq!(
            words("sampādethā\u{2019}ti"),
            vec![word("sampādethā\u{2019}ti")]
        );
    }

    #[test]
    fn quotes_and_punctuation_around_words_are_plain() {
        assert_eq!(
            words("'alaṃ, vakkali!'"),
            vec![
                other("'"),
                word("alaṃ"),
                other(", "),
                word("vakkali"),
                other("!'")
            ]
        );
        assert_eq!(
            words("“kiṃ te?”"),
            vec![other("“"), word("kiṃ"), other(" "), word("te"), other("?”")]
        );
    }

    #[test]
    fn numbers_are_plain() {
        assert_eq!(
            words("SN 22.87 vakkali"),
            vec![word("SN"), other(" 22.87 "), word("vakkali")]
        );
    }

    #[test]
    fn the_tokens_join_back_into_the_text() {
        let text = "alaṃ, vakkali, kiṃ te iminā pūtikāyena diṭṭhena? 'sampādethā'ti' 3";
        let joined: String = words(text)
            .into_iter()
            .map(|t| match t {
                Token::Word(s) | Token::Other(s) => s,
            })
            .collect();
        assert_eq!(joined, text);
    }

    #[test]
    fn an_empty_text_has_no_tokens() {
        assert_eq!(words(""), Vec::<Token>::new());
    }

    fn map(keys: &[&str]) -> HashMap<String, Vec<WordEntry>> {
        keys.iter()
            .map(|k| ((*k).to_owned(), vec![entry(k, None, "m")]))
            .collect()
    }

    #[test]
    fn words_without_entries_merge_into_plain_text() {
        let got = segments(&words("alaṃ, vakkali, kiṃ"), &map(&["vakkali"]), false);
        assert_eq!(
            got,
            vec![
                Segment::Plain("alaṃ, ".to_owned()),
                Segment::Word {
                    form: "vakkali".to_owned(),
                    entries: vec![entry("vakkali", None, "m")],
                },
                Segment::Plain(", kiṃ".to_owned()),
            ]
        );
    }

    #[test]
    fn a_word_is_looked_up_lower_cased_and_shown_as_written() {
        let got = segments(&words("Alaṃ"), &map(&["alaṃ"]), false);
        assert_eq!(
            got,
            vec![Segment::Word {
                form: "Alaṃ".to_owned(),
                entries: vec![entry("alaṃ", None, "m")],
            }]
        );
    }

    #[test]
    fn an_empty_entry_list_is_no_entry() {
        let mut entries = map(&[]);
        entries.insert("te".to_owned(), Vec::new());
        assert_eq!(
            segments(&words("te"), &entries, false),
            vec![Segment::Plain("te".to_owned())]
        );
    }

    #[test]
    fn at_most_three_entries_are_shown_and_the_rest_counted() {
        let five: Vec<WordEntry> = (0..5).map(|i| entry(&i.to_string(), None, "m")).collect();
        let (list, more) = shown(&five);
        assert_eq!(list, &five[..3]);
        assert_eq!(more, 2);
        let (list, more) = shown(&five[..3]);
        assert_eq!(list.len(), 3);
        assert_eq!(more, 0);
        let (list, more) = shown(&five[..1]);
        assert_eq!(list.len(), 1);
        assert_eq!(more, 0);
    }

    #[test]
    fn the_more_line_defaults_to_english_or_takes_the_callers_label() {
        assert_eq!(more_text(None, 2), "+ 2 more");
        let label = Callback::new(|n: usize| format!("+ {n} autres"));
        assert_eq!(more_text(Some(label), 4), "+ 4 autres");
    }

    const WORD_AT: Rect = Rect {
        left: 100.0,
        top: 100.0,
        bottom: 120.0,
    };
    const PANEL: Size = Size {
        width: 288.0,
        height: 200.0,
    };
    const DESKTOP: Size = Size {
        width: 1280.0,
        height: 800.0,
    };

    #[test]
    fn the_panel_goes_8px_below_the_word_at_its_left_edge() {
        assert_eq!(
            place(WORD_AT, PANEL, DESKTOP),
            Pos {
                top: 128.0,
                left: 100.0
            }
        );
    }

    #[test]
    fn the_panel_goes_above_when_below_lacks_room_and_above_has_more() {
        let low = Rect {
            top: 700.0,
            bottom: 720.0,
            ..WORD_AT
        };
        assert_eq!(
            place(low, PANEL, DESKTOP),
            Pos {
                top: 492.0,
                left: 100.0
            }
        );
    }

    #[test]
    fn the_panel_stays_below_when_above_has_less_room() {
        let tall = Size {
            height: 600.0,
            ..PANEL
        };
        let word = Rect {
            top: 300.0,
            bottom: 320.0,
            ..WORD_AT
        };
        assert_eq!(
            place(word, tall, DESKTOP),
            Pos {
                top: 328.0,
                left: 100.0
            }
        );
    }

    #[test]
    fn a_panel_above_never_starts_above_the_viewport() {
        let word = Rect {
            top: 100.0,
            bottom: 120.0,
            ..WORD_AT
        };
        let short = Size {
            height: 150.0,
            ..DESKTOP
        };
        // Above it would start at 100 - 8 - 200 = -108.
        assert_eq!(place(word, PANEL, short).top.to_bits(), 8.0_f64.to_bits());
    }

    #[test]
    fn the_panel_is_clamped_16px_inside_the_left_edge() {
        let word = Rect {
            left: 4.0,
            ..WORD_AT
        };
        assert_eq!(
            place(word, PANEL, DESKTOP).left.to_bits(),
            16.0_f64.to_bits()
        );
    }

    #[test]
    fn the_panel_is_clamped_16px_inside_the_right_edge() {
        let word = Rect {
            left: 1200.0,
            ..WORD_AT
        };
        // 1280 - 288 - 16.
        assert_eq!(
            place(word, PANEL, DESKTOP).left.to_bits(),
            976.0_f64.to_bits()
        );
    }

    #[test]
    fn a_panel_wider_than_the_viewport_starts_16px_in() {
        let wide = Size {
            width: 700.0,
            ..PANEL
        };
        let narrow = Size {
            width: 600.0,
            ..DESKTOP
        };
        assert_eq!(
            place(WORD_AT, wide, narrow).left.to_bits(),
            16.0_f64.to_bits()
        );
    }

    #[test]
    fn the_word_is_underlined_only_in_a_state() {
        for class in WORD.split_whitespace() {
            if class.ends_with("underline") {
                assert!(class.contains(':'), "{class} at rest");
            }
        }
        for state in [
            "hover:underline",
            "focus-visible:underline",
            "aria-expanded:underline",
        ] {
            assert!(WORD.split_whitespace().any(|c| c == state), "{state}");
        }
        assert!(WORD.contains("decoration-dotted") && WORD.contains("decoration-gold"));
    }
}

#[cfg(all(test, feature = "ssr"))]
mod render_tests {
    use super::*;

    fn entry(headword: &str, tag: Option<&str>, meaning: &str) -> WordEntry {
        WordEntry {
            headword: headword.to_owned(),
            tag: tag.map(str::to_owned),
            meaning: meaning.to_owned(),
        }
    }

    fn vakkali_map() -> HashMap<String, Vec<WordEntry>> {
        let mut map = HashMap::new();
        map.insert(
            "alaṃ".to_owned(),
            vec![entry("alaṃ", Some("ind"), "enough!; stop!")],
        );
        map.insert(
            "vakkali".to_owned(),
            (0..5)
                .map(|i| entry(&format!("vakkali {i}"), Some("masc"), "a monk"))
                .collect(),
        );
        map
    }

    const LINE: &str = "alaṃ, vakkali, kiṃ te?";

    /// The passage, with the word at segment `open` open (or none).
    fn rendered(open: Option<usize>, more_label: Option<Callback<usize, String>>) -> String {
        rendered_with(open, more_label, None)
    }

    /// The same, with the caller's per-entry action.
    fn rendered_with(
        open: Option<usize>,
        more_label: Option<Callback<usize, String>>,
        entry_action: Option<EntryAction>,
    ) -> String {
        Owner::new().with(|| {
            let ctx = Ctx {
                open: OpenWord {
                    id: "pt-panel",
                    word: RwSignal::new(open),
                },
                size: PaliSize::Body,
                more_label,
                entry_action,
                lookup: None,
                empty_label: None,
            };
            passage(LINE, Signal::stored(vakkali_map()), ctx, "mt-2".to_owned()).to_html()
        })
    }

    fn rendered_lookup(open: Option<usize>, lookup: Callback<String>, label: &str) -> String {
        let label = label.to_string();
        Owner::new().with(|| {
            let ctx = Ctx {
                open: OpenWord {
                    id: "pt-panel",
                    word: RwSignal::new(open),
                },
                size: PaliSize::Body,
                more_label: None,
                entry_action: None,
                lookup: Some(lookup),
                empty_label: Some(Signal::derive(move || label.clone())),
            };
            passage(LINE, Signal::stored(vakkali_map()), ctx, String::new()).to_html()
        })
    }

    /// With a lookup every word is a button, entries or not (065): the
    /// line's four words, not only the two the map knows.
    #[test]
    fn with_a_lookup_every_word_is_a_button() {
        let html = rendered_lookup(None, Callback::new(|_| {}), "Looking up");
        assert_eq!(buttons(&html).len(), 4, "{html}");
        assert!(html.contains(">kiṃ<"), "{html}");
        assert!(html.contains(">te<"), "{html}");
        let plain = rendered(None, None);
        assert_eq!(
            buttons(&plain).len(),
            2,
            "without a lookup, as before: {plain}"
        );
    }

    /// The panel of a word with no entry says the label; a word with
    /// entries shows them and no label.
    #[test]
    fn an_empty_word_panel_says_the_label() {
        // The segments of LINE with every word a button: punctuation and
        // spaces sit between them, so kiṃ's index is found, not assumed.
        let kim = segments(&words(LINE), &vakkali_map(), true)
            .iter()
            .position(|s| matches!(s, Segment::Word { form, .. } if form == "kiṃ"))
            .unwrap();
        let html =
            rendered_lookup(Some(kim), Callback::new(|_| {}), "Looking up").replace("<!>", "");
        assert!(html.contains("data-empty-entry"), "{html}");
        assert!(html.contains(">Looking up</li>"), "{html}");
        let html = rendered_lookup(Some(0), Callback::new(|_| {}), "Looking up");
        assert!(
            !html.contains("data-empty-entry"),
            "entries: no label: {html}"
        );
    }

    /// The lookup runs when an empty word opens, with the form as written;
    /// never for a word that has entries.
    #[test]
    fn opening_an_empty_word_asks_the_lookup_once() {
        Owner::new().with(|| {
            let asked = RwSignal::new(Vec::<String>::new());
            let lookup = Callback::new(move |form: String| asked.update(|v| v.push(form)));
            let ctx = Ctx {
                open: OpenWord {
                    id: "pt-panel",
                    word: RwSignal::new(None),
                },
                size: PaliSize::Body,
                more_label: None,
                entry_action: None,
                lookup: Some(lookup),
                empty_label: None,
            };
            let tokens = words(LINE);
            let segs = segments(&tokens, &vakkali_map(), true);
            // press as the button does: open, then ask when empty
            let press = |index: usize, entries: &[WordEntry], form: &str| {
                ctx.open.press(index);
                if entries.is_empty() && ctx.open.current() == Some(index) {
                    ctx.lookup.unwrap().run(form.to_string());
                }
            };
            let word = |i: usize| match &segs[i] {
                Segment::Word { form, entries } => (form.clone(), entries.clone()),
                Segment::Plain(_) => panic!("a word"),
            };
            let (form, entries) = word(0);
            press(0, &entries, &form);
            assert_eq!(
                asked.get_untracked().len(),
                0,
                "alaṃ has entries: no lookup"
            );
            ctx.open.close();
            let kim = segs
                .iter()
                .position(|s| matches!(s, Segment::Word { form, .. } if form == "kiṃ"))
                .unwrap();
            let (form, entries) = word(kim);
            press(kim, &entries, &form);
            assert_eq!(asked.get_untracked(), vec!["kiṃ".to_string()]);
            press(kim, &entries, &form);
            assert_eq!(asked.get_untracked().len(), 1, "closing asks nothing");
        });
    }

    /// The opening tags of every `<button`.
    fn buttons(html: &str) -> Vec<&str> {
        html.match_indices("<button")
            .map(|(at, _)| &html[at..=at + html[at..].find('>').expect(html)])
            .collect()
    }

    #[test]
    fn only_words_with_entries_are_buttons() {
        let html = Owner::new().with(|| {
            view! { <PaliText id="pt-panel" text=LINE entries=vakkali_map() /> }.to_html()
        });
        let tags = buttons(&html);
        assert_eq!(tags.len(), 2, "{html}");
        assert!(html.contains(">alaṃ</button>"), "{html}");
        assert!(html.contains(">vakkali</button>"), "{html}");
        assert!(!html.contains(">kiṃ</button>"), "{html}");
        assert!(html.contains("kiṃ te?"), "plain, untouched: {html}");
    }

    #[test]
    fn each_word_button_is_a_closed_disclosure_of_the_panel() {
        let html = rendered(None, None);
        assert_eq!(buttons(&html).len(), 2, "{html}");
        for tag in buttons(&html) {
            assert!(tag.contains(r#"type="button""#), "{tag}");
            assert!(tag.contains(r#"aria-controls="pt-panel""#), "{tag}");
            assert!(tag.contains(r#"aria-expanded="false""#), "{tag}");
            assert!(tag.contains("decoration-dotted"), "{tag}");
        }
        assert!(
            !html.contains(r#"role="dialog""#),
            "no panel while closed: {html}"
        );
    }

    #[test]
    fn the_passage_is_pali_and_keeps_its_punctuation_and_layout() {
        let html = rendered(None, None);
        assert!(html.starts_with(r#"<span class="mt-2""#), "{html}");
        assert!(html.contains(r#"lang="pi""#), "{html}");
        assert!(html.contains("text-[19px] leading-8"), "{html}");
        assert!(html.contains(", "), "{html}");
        for tag in buttons(&html) {
            assert!(!tag.contains(','), "{tag}");
        }
    }

    #[test]
    fn the_open_word_shows_its_panel_named_by_the_form() {
        let html = rendered(Some(0), None);
        assert!(html.contains(r#"id="pt-panel""#), "{html}");
        assert!(html.contains(r#"role="dialog""#), "{html}");
        assert!(html.contains(r#"aria-label="alaṃ""#), "{html}");
        assert!(html.contains("enough!; stop!"), "{html}");
        assert!(html.contains(">ind<"), "{html}");
        assert_eq!(html.matches(r#"aria-expanded="true""#).count(), 1, "{html}");
        assert!(html.contains(r#"aria-label="Close""#), "{html}");
        assert!(html.contains("bg-black/40"), "the phone backdrop: {html}");
        assert!(html.contains("max-sm:fixed"), "{html}");
    }

    /// The class tokens of the open panel (`role="dialog"`).
    fn panel_classes(html: &str) -> Vec<&str> {
        let at = html.find(r#"role="dialog""#).expect(html);
        let start = html[..at].rfind('<').expect(html);
        let tag = &html[start..at + html[at..].find('>').expect(html)];
        let class = tag.split(r#"class=""#).nth(1).expect(tag);
        class[..class.find('"').expect(tag)]
            .split_whitespace()
            .collect()
    }

    #[test]
    fn the_desktop_panel_is_fixed_in_the_viewport_not_in_its_container() {
        let html = rendered(Some(0), None);
        let classes = panel_classes(&html);
        for wanted in [
            "sm:fixed",
            "sm:top-(--pc-word-top)",
            "sm:left-(--pc-word-left)",
            "max-sm:fixed",
            "max-sm:bottom-0",
        ] {
            assert!(classes.contains(&wanted), "{wanted}: {classes:?}");
        }
        for gone in ["sm:absolute", "sm:top-full", "sm:left-0", "sm:mt-2"] {
            assert!(!classes.contains(&gone), "{gone}: {classes:?}");
        }
    }

    #[test]
    fn a_long_entry_list_shows_three_and_counts_the_rest() {
        let html = rendered(Some(2), None);
        assert!(html.contains(r#"aria-label="vakkali""#), "{html}");
        for shown in ["vakkali 0", "vakkali 1", "vakkali 2"] {
            assert!(html.contains(shown), "{shown}: {html}");
        }
        for hidden in ["vakkali 3", "vakkali 4"] {
            assert!(!html.contains(hidden), "{hidden}: {html}");
        }
        assert!(html.contains("+ 2 more"), "{html}");
    }

    #[test]
    fn the_more_line_takes_the_callers_label() {
        let label = Callback::new(|n: usize| format!("{n} de plus"));
        let html = rendered(Some(2), Some(label));
        assert!(html.contains("2 de plus"), "{html}");
        assert!(!html.contains("+ 2 more"), "{html}");
    }

    #[test]
    fn a_short_list_has_no_more_line() {
        let html = rendered(Some(0), None);
        assert!(!html.contains("more"), "{html}");
    }

    /// The action a service passes: one `<button>` naming the pressed form
    /// and the entry's headword.
    fn add_action() -> EntryAction {
        Callback::new(|(form, entry): (String, WordEntry)| {
            view! { <button data-add=form>{format!("Add {}", entry.headword)}</button> }.into_any()
        })
    }

    #[test]
    fn without_an_action_the_markup_is_what_it_was_before_the_slot() {
        for (open, golden) in [
            (
                None,
                include_str!("../tests/fixtures/pali_text_closed.html"),
            ),
            (
                Some(0),
                include_str!("../tests/fixtures/pali_text_open.html"),
            ),
            (
                Some(2),
                include_str!("../tests/fixtures/pali_text_open_capped.html"),
            ),
        ] {
            assert_eq!(rendered(open, None), golden, "{open:?}");
        }
    }

    #[test]
    fn each_shown_entry_gets_the_action_with_the_form_and_the_entry() {
        let html = rendered_with(Some(2), None, Some(add_action()));
        for shown in 0..3 {
            let wanted = format!(r#"<button data-add="vakkali">Add vakkali {shown}</button>"#);
            assert_eq!(html.matches(&wanted).count(), 1, "{wanted}: {html}");
        }
        assert_eq!(html.matches("data-add").count(), 3, "{html}");
        assert!(!html.contains("Add vakkali 3"), "hidden entries: {html}");
    }

    #[test]
    fn the_action_sits_in_its_entry_after_the_meaning() {
        let html = rendered_with(Some(0), None, Some(add_action()));
        let item = &html[html.find("<li>").expect(&html)..html.find("</li>").expect(&html)];
        let meaning = item.find("enough!; stop!").expect(item);
        let action = item.find("data-add").expect(item);
        assert!(meaning < action, "{item}");
        assert_eq!(html.matches("<li>").count(), 1, "{html}");
    }

    #[test]
    fn the_action_is_in_the_open_panel_only() {
        let closed = rendered_with(None, None, Some(add_action()));
        assert!(!closed.contains("data-add"), "{closed}");
    }

    #[test]
    fn the_slot_wrapper_sets_layout_only() {
        let html = rendered_with(Some(0), None, Some(add_action()));
        let at = html.find("<button data-add").expect(&html);
        let wrapper = &html[html[..at].rfind("<div").expect(&html)..at];
        assert_eq!(wrapper, r#"<div class="mt-1">"#, "{html}");
    }

    #[test]
    fn the_component_takes_the_prop() {
        let html = Owner::new().with(|| {
            view! { <PaliText id="pt-panel" text=LINE entries=vakkali_map() entry_action=add_action() /> }
            .to_html()
        });
        assert_eq!(buttons(&html).len(), 2, "closed: no action yet: {html}");
        assert!(!html.contains("data-add"), "{html}");
    }

    /// The passage as `PaliText` builds it from its props (`new_ctx`), with
    /// the word at segment `open` open: the way to render a panel open
    /// through the component's own wiring, a click being a browser's.
    fn component_open(open: usize, entry_action: Option<EntryAction>) -> String {
        Owner::new().with(|| {
            let ctx = new_ctx(
                "pt-panel",
                PaliSize::Body,
                None,
                entry_action,
                Some(open),
                None,
                None,
            );
            passage(LINE, Signal::stored(vakkali_map()), ctx, String::new()).to_html()
        })
    }

    #[test]
    fn the_prop_reaches_the_open_panel_through_the_components_wiring() {
        let html = component_open(0, Some(add_action()));
        assert_eq!(html.matches("data-add").count(), 1, "{html}");
        assert!(
            html.contains(r#"<button data-add="alaṃ">Add alaṃ</button>"#),
            "{html}"
        );
        let bare = component_open(0, None);
        assert!(!bare.contains("data-add"), "{bare}");
    }

    /// The calls an action recorded: (form, entry).
    type Calls = std::sync::Arc<std::sync::Mutex<Vec<(String, WordEntry)>>>;

    /// An action that records every call it gets.
    fn recording() -> (EntryAction, Calls) {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let seen = calls.clone();
        let action = Callback::new(move |(form, entry): (String, WordEntry)| {
            seen.lock().expect("lock").push((form, entry));
            ().into_any()
        });
        (action, calls)
    }

    #[test]
    fn the_action_is_called_once_per_visible_entry_with_the_form_and_the_entry() {
        let (action, calls) = recording();
        let _ = component_open(2, Some(action));
        let expected: Vec<(String, WordEntry)> = (0..3)
            .map(|i| {
                (
                    "vakkali".to_owned(),
                    entry(&format!("vakkali {i}"), Some("masc"), "a monk"),
                )
            })
            .collect();
        assert_eq!(
            *calls.lock().expect("lock"),
            expected,
            "three of five, in order"
        );
    }

    #[test]
    fn the_action_is_not_called_while_the_panel_is_closed() {
        let (action, calls) = recording();
        let _ = rendered_with(None, None, Some(action));
        assert_eq!(calls.lock().expect("lock").len(), 0);
    }

    #[test]
    fn a_tall_action_stays_inside_the_phone_sheets_scroll() {
        let html = rendered_with(Some(0), None, Some(add_action()));
        let classes = panel_classes(&html);
        for wanted in ["max-sm:max-h-[70vh]", "max-sm:overflow-y-auto"] {
            assert!(classes.contains(&wanted), "{wanted}: {classes:?}");
        }
        let panel = html.find(r#"role="dialog""#).expect(&html);
        let end = html.find("</ul>").expect(&html);
        let action = html.find("data-add").expect(&html);
        assert!(
            panel < action && action < end,
            "inside the panel's list: {html}"
        );
    }

    #[test]
    fn text_and_entries_are_escaped() {
        let mut map = HashMap::new();
        map.insert(
            "b".to_owned(),
            vec![entry("<i>h</i>", Some("<i>t</i>"), "<i>m</i>")],
        );
        let html = Owner::new().with(|| {
            let ctx = Ctx {
                open: OpenWord {
                    id: "pt-esc",
                    word: RwSignal::new(Some(1)),
                },
                size: PaliSize::Body,
                more_label: None,
                entry_action: None,
                lookup: None,
                empty_label: None,
            };
            passage("<b>", Signal::stored(map), ctx, String::new()).to_html()
        });
        assert!(!html.contains("<i>") && !html.contains("<b>"), "{html}");
        assert!(html.matches("&lt;i&gt;").count() >= 3, "{html}");
    }
}
