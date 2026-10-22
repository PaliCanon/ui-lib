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

/// Formats the environment and sha into a label, with separator only when
/// sha is not empty.
fn label(env: &str, sha: &str) -> String {
    if sha.is_empty() {
        env.to_string()
    } else {
        format!("{env} · {sha}")
    }
}

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
/// When `sha` is empty, the separator is omitted.
#[component]
pub fn EnvBadge(
    /// The environment: a git branch, or a service's own `dev` / `prod`.
    /// Accepts owned strings via `#[prop(into)]`.
    #[prop(into)]
    env: String,
    /// The short commit sha, a trailing `*` for a dirty build.
    /// Accepts owned strings via `#[prop(into)]`; when empty, the separator is omitted.
    #[prop(into)]
    sha: String,
) -> impl IntoView {
    let title = label(&env, &sha);
    let show_separator = !sha.is_empty();
    let sha_display = sha.clone();

    view! {
        <span
            class="inline-flex max-w-full items-center gap-1 rounded-full border border-line bg-chip px-2 py-0.5 font-mono text-xs text-muted"
            title=title
        >
            <span class="min-w-0 truncate">{env}</span>
            {show_separator
                .then(|| {
                    view! {
                        <>
                            <span aria-hidden="true" class="shrink-0 text-subtle">
                                "·"
                            </span>
                            <span class="shrink-0">{sha_display}</span>
                        </>
                    }
                })}
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

    #[test]
    fn empty_sha_drops_the_separator() {
        let html = Owner::new().with(|| view! { <EnvBadge env="v0.1.8" sha="" /> }.to_html());
        assert!(html.contains(">v0.1.8<"), "{html}");
        assert!(!html.contains("·"), "{html}");
    }

    #[test]
    fn empty_sha_drops_the_separator_in_title() {
        let html = Owner::new().with(|| view! { <EnvBadge env="v0.1.8" sha="" /> }.to_html());
        assert!(html.contains(r#"title="v0.1.8""#), "{html}");
        assert!(!html.contains("·"), "{html}");
    }

    #[test]
    fn non_empty_sha_includes_the_separator() {
        let html = Owner::new().with(|| view! { <EnvBadge env="main" sha="34c489c" /> }.to_html());
        assert!(html.contains(">main<"), "{html}");
        assert!(html.contains(">34c489c<"), "{html}");
        assert!(html.contains("·"), "{html}");
    }

    #[test]
    fn accepts_owned_string_props() {
        let env = String::from("main");
        let sha = String::from("34c489c");
        let html = Owner::new().with(|| view! { <EnvBadge env=env sha=sha /> }.to_html());
        assert!(html.contains(">main<"), "{html}");
        assert!(html.contains(">34c489c<"), "{html}");
    }

    #[test]
    fn label_helper_formats_env_and_sha() {
        assert_eq!(label("v0.1.8", ""), "v0.1.8");
        assert_eq!(label("main", "34c489c"), "main · 34c489c");
        assert_eq!(label("main", ""), "main");
    }
}
