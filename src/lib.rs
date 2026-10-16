//! PaliCanon's UI incubator: reusable Leptos pieces that one project uses so
//! far. When a second project needs one, it moves to `PaliCanon/ui` (see
//! `docs/conventions/ui-lib.md`).
//!
//! - [`persisted`](mod@persisted) — a value kept in a cookie shared by every PaliCanon
//!   subdomain, read on the server for the first render and written by the
//!   browser on change.
//! - [`button`] — the button family: variants and sizes for the look, one
//!   small component per behaviour.
//! - [`language`] — the interface language: [`language::Language`], the
//!   one [`language::language_signal`], and `resolve_language` (the
//!   server's choice: cookie `pc_lang`, then `Accept-Language`, then
//!   English).
//! - [`icon`] — the icons of the identity, one at a time as they are needed.
//! - [`env_badge`] — which build is running: a small mono badge, `env ·
//!   sha7`, never interactive; [`shell::AppShell`]'s `env_badge` prop shows
//!   it at the rail bottom and in the mobile top bar.
//! - [`nav`] — the navigation list: an app's sections and entries, rendered
//!   by the shell in the rail and in the mobile navigation.
//! - [`popover`] — a floating panel tied to a [`button::Disclosure`],
//!   closing itself on Escape and on an outside click once hydrated.
//! - [`motion`] — the motion tokens (durations, easings) a [`motion::Motion`]
//!   builds its classes on.
//! - [`presence`] — [`presence::Presence`], a primitive that keeps its
//!   content mounted through its exit animation, played by a
//!   [`motion::Motion`].
//! - [`menu`] — a list of [`menu::MenuItem`]s, with submenus: [`menu::Menu`]
//!   (a popover, nested side by side) on desktop, [`menu::DrillMenu`]
//!   (inline, drilling down) on mobile.
//! - [`dialog`] — a modal [`dialog::Dialog`] over a dimmed backdrop:
//!   a header, a scrolling body of [`dialog::DialogSection`]s, an optional
//!   footer. Focus is managed by `focus_trap`, an internal module it alone
//!   uses.
//! - [`theme`] — the reading theme (Light, Dark, System), [`theme::theme_signal`],
//!   [`theme::ThemePicker`] (picture cards, built on [`button::ChoiceCards`])
//!   and [`theme::theme_menu`], a Menu-based theme submenu kept for callers
//!   still using [`menu::Menu`] directly.
//! - [`shell`] — the app shell: the navigation rail (desktop) and the
//!   full-screen navigation (mobile), opened and closed with the lotus, with
//!   Language (the globe and the current language's name, opening a menu of
//!   the languages) and Settings (a [`dialog::Dialog`], the theme as a
//!   [`theme::ThemePicker`] by default) pinned at the bottom.
//! - [`footer`] — [`footer::Footer`]: the caller's own links and credits;
//!   on a phone stacked and centred, from `sm` a 1fr/auto/1fr grid — the
//!   links centred on the full width, the credits at the right edge; no
//!   credits, no block. It owns only the layout and its own nav label.
//! - [`hydration`] — hydrating behind Cloudflare: [`hydration::HydrationRoot`]
//!   (`<div id="pc-app">`, put around the app by the server's shell) and
//!   `hydrate`, the browser entry hydrating from that root instead of
//!   `<body>`, so nodes injected into `<body>` are never visited.
//! - [`search`] — the search forms: [`search::SearchCard`], the home's large
//!   card (the field, the caller's controls and a round submit), and
//!   [`search::SearchRow`], a results page's one-line bar (an icon, the
//!   field, a gold submit); plain GET forms, ids unique per page, the
//!   row's `form_id` required.
//! - [`pali`] — [`pali::Pali`], a Pāli text: `lang="pi"`, the serif face and
//!   a line height that never clips the diacritics, in three sizes.
//! - [`pali_text`] — [`pali_text::PaliText`], a Pāli passage whose words
//!   open their meaning: a popover under the word (a bottom sheet on a
//!   phone) listing the caller's [`pali_text::WordEntry`]s.
//! - [`pali_term`] — [`pali_term::PaliTerm`], a Pāli term on two lines: the
//!   lemma in bold with a tag after it, the gloss under it, and a slot at
//!   the right of each line; not interactive, the caller wraps it.
//! - [`reveal_card`] — [`reveal_card::RevealCard`], a card with a front, a
//!   reveal shown in place and actions pinned at the bottom; Space and Enter
//!   reveal it, and a new key resets it in the same tick.
//! - [`bucket_strip`] — [`bucket_strip::BucketStrip`], n labelled counts in a
//!   row (three columns on a phone), each cell a plain box or a link on
//!   [`button::Variant::Tile`].
//! - [`carousel`] — [`carousel::Carousel`], a row of slides scrolled by
//!   swipe, the arrow keys or Previous and Next buttons (shown once
//!   hydrated), each slide named "n of N"; native scroll-snap, no autoplay.
//! - [`slider`] — [`slider::Slider`], a native range slider with its value
//!   shown at the right, the value held inside `[min, max]`.
//! - [`session_header`] — [`session_header::SessionHeader`], a counter
//!   ("3 / 10 cards") and a close button; the dialog is the caller's.

pub mod bucket_strip;
pub mod button;
pub mod carousel;
mod catalogue;
// Present only under the `csr` feature (an inner cfg in the file: the grep
// gates keep every `csr` cfg in csr.rs).
pub mod csr;
pub mod dialog;
pub mod env_badge;
mod focus_trap;
pub mod footer;
pub mod hydration;
pub mod icon;
pub mod language;
mod language_menu;
pub mod menu;
pub mod motion;
pub mod nav;
pub mod pali;
pub mod pali_term;
pub mod pali_text;
pub mod persisted;
pub mod popover;
pub mod presence;
pub mod reveal_card;
pub mod search;
pub mod session_header;
pub mod shell;
pub mod slider;
pub mod store;
pub mod theme;
mod viewport;

// The build script's placeholder-parity check, unit-tested here (it has no
// test harness of its own).
#[cfg(test)]
#[path = "../build/placeholders.rs"]
mod placeholders;

pub use bucket_strip::{Bucket, BucketStrip, Columns};
pub use button::{
    Button, ChoiceCardOption, ChoiceCards, Disclosed, Disclosure, DisclosureButton, LinkButton,
    RadioButton, Segmented, SegmentedOption, Size, ToggleButton, Variant,
};
pub use carousel::Carousel;
pub use dialog::{Dialog, DialogSection};
pub use env_badge::{EnvBadge, EnvBadgeInfo};
pub use footer::{Footer, FooterLink};
// A glob: `app_root` and `hydrate` exist only under `hydrate`, a cfg that
// lives in hydration.rs alone (the grep gates, tests/cfg_gates.rs).
// Every `pub` item of hydration.rs is the crate's API (today exactly
// APP_ROOT_ID, HydrationRoot, MissingRoot, app_root, hydrate): a new `pub`
// item there is a new export.
pub use hydration::*;
pub use icon::{Icon, IconName};
pub use language::{Language, language_attribute, language_signal, provide_language};
pub use menu::{DrillMenu, Menu, MenuItem};
pub use motion::{Edge, Motion};
pub use nav::{NavEntry, NavList, NavSection};
pub use pali::{Pali, PaliSize};
pub use pali_term::PaliTerm;
pub use pali_text::{PaliText, Token, WordEntry, words};
pub use persisted::persisted;
pub use popover::Popover;
pub use presence::Presence;
pub use reveal_card::RevealCard;
pub use search::{SearchCard, SearchRow};
pub use session_header::SessionHeader;
pub use shell::{AppShell, ShellState};
pub use slider::Slider;
pub use store::{Store, StoreError};
pub use theme::{Theme, ThemePicker, theme_attribute, theme_menu, theme_signal};
