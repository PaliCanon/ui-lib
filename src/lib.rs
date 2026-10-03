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
//! - [`nav`] — the navigation list: an app's sections and entries, rendered
//!   by the shell in the rail and in the mobile navigation.
//! - [`popover`] — a floating panel tied to a [`button::Disclosure`],
//!   closing itself on Escape and on an outside click once hydrated.
//! - [`menu`] — a list of [`menu::MenuItem`]s, with submenus: [`menu::Menu`]
//!   (a popover, nested side by side) on desktop, [`menu::DrillMenu`]
//!   (inline, drilling down) on mobile.
//! - [`theme`] — the reading theme (Light, Dark, System) and
//!   [`theme::theme_menu`], the Settings submenu that chooses it.
//! - [`shell`] — the app shell: the navigation rail (desktop) and the
//!   full-screen navigation (mobile), opened and closed with the lotus, with
//!   Settings (a [`menu::Menu`], [`theme::theme_menu`] by default) pinned at
//!   the bottom.

pub mod button;
pub mod icon;
pub mod menu;
pub mod nav;
pub mod persisted;
pub mod popover;
pub mod shell;
pub mod theme;
mod viewport;

pub use button::{
    Button, Disclosed, Disclosure, DisclosureButton, LinkButton, RadioButton, Size, ToggleButton,
    Variant,
};
pub use icon::{Icon, IconName};
pub use menu::{DrillMenu, Menu, MenuItem};
pub use nav::{NavEntry, NavList, NavSection};
pub use persisted::persisted;
pub use popover::Popover;
pub use shell::{AppShell, ShellState};
pub use theme::{Theme, theme_attribute, theme_menu};
