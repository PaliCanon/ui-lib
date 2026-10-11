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
//! - **Desktop** (`sm` and up): a [`Popover`] anchored under the pressed
//!   word (flipped above it when it would run past the viewport's bottom):
//!   the form pressed in bold, then each entry, its headword in bold, its
//!   tag muted, its meaning under them; at most three entries, then
//!   "+ n more".
//! - **Phone**: the same panel as a bottom sheet (fixed to the bottom,
//!   full width, rounded top) over a dimmed backdrop, with a close button;
//!   a tap on the backdrop closes it.
//! - **One word open at a time**: pressing another word moves the panel
//!   there, pressing the open word again closes it. Escape closes it, as
//!   any [`Popover`]: its window listener lives as long as the open panel
//!   and is removed with it.
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
fn segments(tokens: &[Token], entries: &HashMap<String, Vec<WordEntry>>) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    for token in tokens {
        let (text, found) = match token {
            Token::Word(form) => (form, entries.get(&form.to_lowercase())),
            Token::Other(text) => (text, None),
        };
        match (found.filter(|list| !list.is_empty()), out.last_mut()) {
            (Some(list), _) => out.push(Segment::Word {
                form: text.clone(),
                entries: list.clone(),
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
/// square bottom corners, scrolling past 70% of the screen), a popover
/// under the word from `sm`.
const PANEL_LAYOUT: &str = "z-50 max-sm:fixed max-sm:inset-x-0 max-sm:bottom-0 \
    max-sm:max-h-[70vh] max-sm:overflow-y-auto max-sm:rounded-b-none \
    sm:absolute sm:left-0 sm:top-full sm:mt-2 sm:w-72 sm:max-w-[calc(100vw-2rem)]";

/// The dimmed backdrop behind the sheet, on a phone only.
const BACKDROP: &str = "fixed inset-0 z-40 bg-black/40 sm:hidden";

/// The panel's content: its own text size and colour (whatever the passage
/// inherits), and room for the phone's home indicator.
const CONTENT: &str = "px-2 py-1 text-left text-sm leading-6 font-normal text-ink \
    max-sm:pb-[max(0.25rem,env(safe-area-inset-bottom))]";

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
}

/// A Pāli passage whose words open their meaning: each word with entries
/// is an inline button; pressing it shows its entries in a popover under
/// it (a bottom sheet on a phone). See the [module](self) for the rules.
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
    /// Layout classes only (margin, width, position), never colours.
    #[prop(optional, into)]
    layout: String,
) -> impl IntoView {
    let open = OpenWord {
        id,
        word: RwSignal::new(None),
    };
    let ctx = Ctx {
        open,
        size,
        more_label,
    };
    passage(&text, entries, ctx, layout)
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
                    .with(|map| segments(&tokens, map))
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
    view! {
        <span class="relative">
            <Pali size=ctx.size>
                <button
                    type="button"
                    class=WORD
                    aria-controls=open.id
                    aria-expanded=move || is_open().to_string()
                    on:click=move |_| open.press(index)
                >
                    {form}
                </button>
            </Pali>
            {move || {
                is_open()
                    .then(|| {
                        view! {
                            <WordPanel form=panel_form.clone() entries=entries.clone() ctx=ctx />
                        }
                    })
            }}
        </span>
    }
}

/// The open word's panel: the backdrop (phone) and the popover or sheet,
/// mounted only while the word is open, so the popover's listeners end
/// with it.
#[component]
fn WordPanel(form: String, entries: Vec<WordEntry>, ctx: Ctx) -> impl IntoView {
    let open = ctx.open;
    let (list, more) = shown(&entries);
    let list = list.to_vec();
    let more_label = ctx.more_label;
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
                <ul class="mt-1 space-y-2">{list.into_iter().map(entry_view).collect_view()}</ul>
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

/// One entry: the headword in bold and its tag, the meaning under them.
fn entry_view(entry: WordEntry) -> impl IntoView {
    view! {
        <li>
            <p class="flex flex-wrap items-baseline gap-x-2">
                <strong class="font-semibold text-heading">
                    <Pali>{entry.headword}</Pali>
                </strong>
                {entry.tag.map(|tag| view! { <span class="text-muted">{tag}</span> })}
            </p>
            <p>{entry.meaning}</p>
        </li>
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
        let got = segments(&words("alaṃ, vakkali, kiṃ"), &map(&["vakkali"]));
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
        let got = segments(&words("Alaṃ"), &map(&["alaṃ"]));
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
            segments(&words("te"), &entries),
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
        Owner::new().with(|| {
            let ctx = Ctx {
                open: OpenWord {
                    id: "pt-panel",
                    word: RwSignal::new(open),
                },
                size: PaliSize::Body,
                more_label,
            };
            passage(LINE, Signal::stored(vakkali_map()), ctx, "mt-2".to_owned()).to_html()
        })
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
        assert!(html.contains("text-[17px] leading-7"), "{html}");
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
        assert!(
            html.contains("max-sm:fixed") && html.contains("sm:absolute"),
            "{html}"
        );
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
            };
            passage("<b>", Signal::stored(map), ctx, String::new()).to_html()
        });
        assert!(!html.contains("<i>") && !html.contains("<b>"), "{html}");
        assert!(html.matches("&lt;i&gt;").count() >= 3, "{html}");
    }
}
