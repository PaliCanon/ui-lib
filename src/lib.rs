//! PaliCanon's UI incubator: reusable Leptos pieces that one project uses so
//! far. When a second project needs one, it moves to `PaliCanon/ui` (see
//! `docs/conventions/shared-ui.md`).
//!
//! - [`persisted`](mod@persisted) — a value kept in a cookie shared by every PaliCanon
//!   subdomain, read on the server for the first render and written by the
//!   browser on change.
//! - [`button`] — the button family: variants and sizes for the look, one
//!   small component per behaviour.
//! - [`icon`] — the icons of the identity, one at a time as they are needed.
//! - [`env_badge`] — which build is running: a small mono badge, `env ·
//!   sha7`, never interactive; [`shell::AppShell`]'s `env_badge` prop shows
//!   it at the rail bottom and in the mobile top bar.
//! - [`nav`] — the navigation list: an app's sections and entries, rendered
//!   by the shell in the rail and in the mobile navigation.
//! - [`popover`] — a floating panel tied to a [`button::Disclosure`],
//!   closing itself on Escape and on an outside click once hydrated.
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
//!   Settings (a [`dialog::Dialog`], the theme as a [`theme::ThemePicker`] by
//!   default) pinned at the bottom.

pub mod button;
pub mod dialog;
pub mod env_badge;
mod focus_trap;
pub mod icon;
pub mod menu;
pub mod nav;
pub mod persisted;
pub mod popover;
pub mod shell;
pub mod theme;
mod viewport;

pub use button::{
    Button, ChoiceCardOption, ChoiceCards, Disclosed, Disclosure, DisclosureButton, LinkButton,
    RadioButton, Segmented, SegmentedOption, Size, ToggleButton, Variant,
};
pub use dialog::{Dialog, DialogSection};
pub use env_badge::{EnvBadge, EnvBadgeInfo};
pub use icon::{Icon, IconName};
pub use menu::{DrillMenu, Menu, MenuItem};
pub use nav::{NavEntry, NavList, NavSection};
pub use persisted::persisted;
pub use popover::Popover;
pub use shell::{AppShell, ShellState};
pub use theme::{Theme, ThemePicker, theme_attribute, theme_menu, theme_signal};
