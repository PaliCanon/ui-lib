//! The navigation list: what an app's navigation holds, as data, rendered by
//! the shell twice — as the desktop rail (icons, labels when open) and as the
//! rows of the mobile full-screen navigation.
//!
//! A list is ordered [`NavSection`]s of [`NavEntry`]s. Each app builds its
//! own; the shell holds none by default (the lotus only). An entry is a link
//! for now; entries that open something or run an action come when one is
//! needed.

use leptos::prelude::*;

use crate::button::{LinkButton, Size, Variant};
use crate::icon::{Icon, IconName};

/// One entry: a link with an icon and a label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavEntry {
    /// What it says (shown beside the icon; its name for screen readers).
    pub label: &'static str,
    /// Its icon.
    pub icon: IconName,
    /// Where it goes; it is the current entry when this is the current path.
    pub href: &'static str,
}

/// A group of entries, with an optional title shown above them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavSection {
    /// Shown above the entries when there is room (the rail open, the mobile
    /// navigation).
    pub title: Option<&'static str>,
    /// The entries, in order.
    pub entries: Vec<NavEntry>,
}

/// An app's navigation: its sections, and the path being shown (which
/// marks the current entry).
#[derive(Clone, Debug)]
pub struct NavList {
    /// The sections, top to bottom.
    pub sections: Vec<NavSection>,
    /// The path of the page being shown, followed live.
    pub current: Signal<String>,
}

impl NavList {
    /// A list of `sections`; `current` is the path being shown.
    #[must_use]
    pub fn new(sections: Vec<NavSection>, current: impl Into<Signal<String>>) -> Self {
        Self {
            sections,
            current: current.into(),
        }
    }
}

/// The sections of `nav` as the desktop rail shows them: titles and labels
/// fade in when `open`, entries stay as icons otherwise.
#[component]
pub(crate) fn RailSections(nav: NavList, #[prop(into)] open: Signal<bool>) -> impl IntoView {
    let current = nav.current;
    nav.sections
        .into_iter()
        .map(|section| {
            view! {
                <SectionTitle title=section.title shown=open />
                {section
                    .entries
                    .into_iter()
                    .map(|entry| view! { <Entry entry=entry current=current size=Size::Md /> })
                    .collect_view()}
            }
        })
        .collect_view()
}

/// The sections of `nav` as the mobile navigation shows them: large rows,
/// titles always shown.
#[component]
pub(crate) fn MobileSections(nav: NavList) -> impl IntoView {
    let current = nav.current;
    nav.sections
        .into_iter()
        .map(|section| {
            view! {
                <SectionTitle title=section.title shown=Signal::derive(|| true) />
                {section
                    .entries
                    .into_iter()
                    .map(|entry| view! { <Entry entry=entry current=current size=Size::Lg /> })
                    .collect_view()}
            }
        })
        .collect_view()
}

/// A section's title; a section without one still gets the spacing.
#[component]
fn SectionTitle(title: Option<&'static str>, #[prop(into)] shown: Signal<bool>) -> impl IntoView {
    view! {
        <p
            class="mt-4 h-6 truncate px-3 text-xs font-medium text-subtle transition-opacity duration-150"
            class:opacity-0=move || !shown.get()
            aria-hidden=title.is_none().then_some("true")
        >
            {title}
        </p>
    }
}

/// One entry, as a link that looks like a navigation entry.
#[component]
fn Entry(entry: NavEntry, current: Signal<String>, size: Size) -> impl IntoView {
    let is_current = Signal::derive(move || current.get() == entry.href);
    view! {
        <LinkButton href=entry.href variant=Variant::Nav size=size current=is_current>
            <Icon name=entry.icon />
            <span>{entry.label}</span>
        </LinkButton>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn list() -> NavList {
        let entry = |label, href| NavEntry {
            label,
            icon: IconName::Button,
            href,
        };
        NavList::new(
            vec![NavSection {
                title: Some("Components"),
                entries: vec![entry("Buttons", "/buttons"), entry("Shell", "/shell")],
            }],
            "/buttons".to_string(),
        )
    }

    #[test]
    fn the_rail_lists_every_entry_and_marks_the_current_one() {
        let out = Owner::new()
            .with(|| view! { <RailSections nav=list() open=Signal::derive(|| false) /> }.to_html());
        assert!(out.contains("Components"), "{out}");
        assert_eq!(out.matches("<a").count(), 2, "{out}");
        assert_eq!(out.matches(r#"aria-current="page""#).count(), 1, "{out}");
        let current = &out[..out.find(r#"href="/shell""#).unwrap()];
        assert!(
            current.contains(r#"aria-current="page""#),
            "Buttons is current: {out}"
        );
        assert!(
            out.contains("opacity-0"),
            "the title fades out while closed: {out}"
        );
    }

    #[test]
    fn the_mobile_navigation_uses_large_rows() {
        let out = Owner::new().with(|| view! { <MobileSections nav=list() /> }.to_html());
        assert_eq!(out.matches("h-13").count(), 2, "{out}");
        assert!(!out.contains("opacity-0"), "titles always shown: {out}");
    }
}
