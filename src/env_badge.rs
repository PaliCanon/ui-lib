//! Which build is running: a small mono badge, `env · sha7`, muted colours,
//! never interactive — so a tab left open (or a bug report) says at a
//! glance what is behind it.
//!
//! [`EnvBadge`] is the generic widget: `env` and `sha` are whatever the
//! caller already knows (the gallery's own [`crate::shell::AppShell`]
//! `env_badge` prop takes an [`EnvBadgeInfo`] and renders it twice — rail
//! bottom on desktop, top bar on mobile — from one value, built once). A
//! service captures its own `env` (`dev`, `prod`) and `sha` the same way
//! the gallery does: at build time, so the server and the hydrated page
//! always agree (there is no mechanism here for the two to disagree — both
//! read the very same compiled-in value).

use leptos::prelude::*;

/// Which build is running, as data: the environment it was built for (a
/// git branch in the gallery, `dev` / `prod` for a service) and the short
/// commit it came from. Built once by the caller, rendered by
/// [`crate::shell::AppShell`] wherever its `env_badge` prop asks for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnvBadgeInfo {
    /// The environment: a git branch, or a service's own `dev` / `prod`.
    pub env: &'static str,
    /// The short commit sha; a trailing `*` means the worktree had
    /// uncommitted changes when this was built.
    pub sha: &'static str,
}

/// A small mono badge, `env · sha7`: muted colours, never interactive (no
/// button, no link — there is nothing to do with it but read it). `env`
/// truncates in a narrow container (the mobile top bar); `sha` never does.
#[component]
pub fn EnvBadge(
    /// The environment: a git branch, or a service's own `dev` / `prod`.
    env: &'static str,
    /// The short commit sha, a trailing `*` for a dirty build.
    sha: &'static str,
) -> impl IntoView {
    view! {
        <span
            class="inline-flex max-w-full items-center gap-1 rounded-full border border-line bg-chip px-2 py-0.5 font-mono text-xs text-muted"
            title=format!("{env} · {sha}")
        >
            <span class="min-w-0 truncate">{env}</span>
            <span aria-hidden="true" class="shrink-0 text-subtle">
                "·"
            </span>
            <span class="shrink-0">{sha}</span>
        </span>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn shows_the_env_and_the_sha_separated_by_a_middle_dot() {
        let html = Owner::new().with(|| view! { <EnvBadge env="main" sha="a1b2c3f" /> }.to_html());
        assert!(html.contains(">main<"), "{html}");
        assert!(html.contains(">a1b2c3f<"), "{html}");
        assert!(html.contains("·"), "{html}");
    }

    #[test]
    fn a_dirty_sha_keeps_its_trailing_star() {
        let html = Owner::new().with(|| view! { <EnvBadge env="main" sha="a1b2c3f*" /> }.to_html());
        assert!(html.contains(">a1b2c3f*<"), "{html}");
    }

    #[test]
    fn carries_the_full_value_as_a_title_for_a_truncated_branch() {
        let html = Owner::new()
            .with(|| view! { <EnvBadge env="feat/a-long-branch-name" sha="a1b2c3f" /> }.to_html());
        assert!(
            html.contains(r#"title="feat/a-long-branch-name · a1b2c3f""#),
            "{html}"
        );
    }

    #[test]
    fn is_never_interactive() {
        let html = Owner::new().with(|| view! { <EnvBadge env="main" sha="a1b2c3f" /> }.to_html());
        assert!(!html.contains("<button"), "{html}");
        assert!(!html.contains("<a "), "{html}");
    }
}
