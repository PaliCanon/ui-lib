//! PaliCanon's UI incubator: reusable Leptos pieces that one project uses so
//! far. When a second project needs one, it moves to `PaliCanon/ui` (see
//! `docs/conventions/shared-ui.md`).
//!
//! - [`persisted`] — a value kept in a cookie shared by every PaliCanon
//!   subdomain, read on the server for the first render and written by the
//!   browser on change.
//! - [`shell`] — the app shell: the navigation rail (desktop) and the
//!   full-screen navigation (mobile), opened and closed with the lotus.

pub mod persisted;
pub mod shell;

pub use persisted::persisted;
pub use shell::{AppShell, ShellState};
