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
//! - [`shell`] — the app shell: the navigation rail (desktop) and the
//!   full-screen navigation (mobile), opened and closed with the lotus.

pub mod button;
pub mod icon;
pub mod nav;
pub mod persisted;
pub mod shell;

pub use button::{
    Button, Disclosed, Disclosure, DisclosureButton, LinkButton, Size, ToggleButton, Variant,
};
pub use icon::{Icon, IconName};
pub use nav::{NavEntry, NavList, NavSection};
pub use persisted::persisted;
pub use shell::{AppShell, ShellState};
