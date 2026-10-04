//! A generic page footer: the caller gives its own links and credits, the
//! footer owns only the layout and its own small chrome (the nav's accessible
//! name, translated).
//!
//! Modelled on the site's own footer
//! (`web/src/components/Footer.tsx`): on a phone, a centred column — a row
//! of plain links, then the credits (texts, translators, a dictionary's
//! licence…) beneath — never spelled out here, since they differ service to
//! service. The footer spans the page column's width. From `sm:` up, a three-column grid:
//! links centred on the page in the middle column, credits at the footer's right
//! edge, their lines still centred on each other; no credits, no block;
//! no links, no `<nav>`.
//! The bar keeps the page's background.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::catalogue::strings;

/// One footer link: a label and where it goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FooterLink {
    /// What it says.
    pub label: &'static str,
    /// Where it goes.
    pub href: &'static str,
}

/// A page footer: `links` (a real `<a>` each) and `credits` (one line each,
/// e.g. "Texts © SuttaCentral and translators", "Dictionary: DPD, CC BY-NC 4.0").
/// The credits are rendered as a list (one item per line) so a screen reader
/// reads them as separate items. The footer spans the page column's width. On phone:
/// a centred column, links row then credits. From `sm:` up: a three-column grid
/// with links centred on the page in the middle column and credits at the footer's
/// right edge, their lines still centred on each other. No credits (the default):
/// no credits block at all; no links: no `<nav>` (no empty landmark).
/// Optional `leading` content (a build badge, a version) sits at the footer's left:
/// first in the phone's centred column, in the first grid column at the left from `sm:` up.
#[component]
pub fn Footer(
    /// The service's links, in order.
    #[prop(into)]
    links: Vec<FooterLink>,
    /// The service's credits, one per line; none by default (no block).
    #[prop(into, default = Vec::new())]
    credits: Vec<&'static str>,
    /// Something at the footer's left — a build badge, a version: first in
    /// the phone's centred column, in the first column at the left from
    /// `sm:` up. None by default (nothing rendered).
    #[prop(optional, into)]
    leading: Option<ViewFn>,
) -> impl IntoView {
    let t = strings();
    view! {
        <footer class="border-t border-line">
            <div class="flex w-full flex-col items-center gap-4 px-6 py-4 sm:grid sm:grid-cols-[1fr_auto_1fr] sm:items-start">
                {leading.map(|leading| view! {
                    <div class="text-[13px] text-subtle sm:col-start-1 sm:justify-self-start">{leading.run()}</div>
                })}
                {(!links.is_empty()).then(|| view! {
                    <nav
                        aria-label=move || td_string!(t.locale(), footer_navigation)
                        class="flex flex-wrap justify-center gap-x-3 gap-y-2 sm:col-start-2 sm:gap-x-10"
                    >
                        {links
                            .into_iter()
                            .map(|link| {
                                view! {
                                    <a
                                        href=link.href
                                        class="text-[13px] text-muted transition hover:text-gold-text sm:text-sm"
                                    >
                                        {link.label}
                                    </a>
                                }
                            })
                            .collect_view()}
                    </nav>
                })}
                {(!credits.is_empty()).then(|| view! {
                    <ul class="list-none m-0 p-0 text-center text-[13px] leading-5 text-subtle sm:col-start-3 sm:justify-self-end">
                        {credits
                            .into_iter()
                            .map(|line| {
                                view! {
                                    <li>{line}</li>
                                }
                            })
                            .collect_view()}
                    </ul>
                })}
            </div>
        </footer>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::language::{Language, language_signal};

    fn links() -> Vec<FooterLink> {
        vec![
            FooterLink {
                label: "Terms",
                href: "/terms",
            },
            FooterLink {
                label: "Privacy",
                href: "/privacy",
            },
        ]
    }

    fn credits() -> Vec<&'static str> {
        vec![
            "Texts © SuttaCentral and translators",
            "Dictionary: DPD, CC BY-NC 4.0",
        ]
    }

    #[test]
    fn the_footer_bar_keeps_the_pages_background() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());
        let start = html.find("<footer").expect("footer");
        let tag = &html[start..start + html[start..].find('>').unwrap()];
        assert!(!tag.contains("bg-rail"), "{tag}");
    }

    #[test]
    fn footer_links_hover_with_the_gold_text_token_that_reaches_aa() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());
        let a = html.find("<a ").expect("a link");
        let tag = &html[a..a + html[a..].find('>').unwrap()];
        assert!(tag.contains("hover:text-gold-text"), "{tag}");
        // `hover:text-gold-text` contains the old class: assert it with a boundary.
        assert!(!tag.contains("hover:text-gold\""), "{tag}");
        assert!(!tag.contains("hover:text-gold "), "{tag}");
    }

    #[test]
    fn the_bar_is_tight_with_14px_links_on_desktop() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());
        let start = html.find("<footer").expect("footer");
        let div = start + html[start..].find("<div").expect("the container");
        let tag = &html[div..div + html[div..].find('>').unwrap()];
        assert!(tag.contains("py-4"), "{tag}");
        assert!(!tag.contains("py-6"), "{tag}");
        let a = html.find("<a ").expect("a link");
        let tag = &html[a..a + html[a..].find('>').unwrap()];
        assert!(tag.contains("sm:text-sm"), "{tag}");
        assert!(!tag.contains("sm:text-[15px]"), "{tag}");
    }

    #[test]
    fn every_link_is_a_real_anchor() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());
        assert!(html.contains(r#"<a href="/terms""#), "{html}");
        assert!(html.contains(">Terms<"), "{html}");
        assert!(html.contains(r#"<a href="/privacy""#), "{html}");
        assert!(html.contains(">Privacy<"), "{html}");
    }

    #[test]
    fn every_credit_is_its_own_line() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());
        assert!(
            html.contains("Texts © SuttaCentral and translators"),
            "{html}"
        );
        assert!(html.contains("Dictionary: DPD, CC BY-NC 4.0"), "{html}");
        assert_eq!(
            html.matches("<li").count(),
            2,
            "two <li>, one per credit line: {html}"
        );
        assert_eq!(
            html.matches("<ul").count(),
            1,
            "one <ul> wrapping the credits: {html}"
        );
    }

    #[test]
    fn links_centred_on_desktop() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());

        // Container should have the grid layout on desktop
        assert!(
            html.contains("flex-col items-center"),
            "phone: centred column: {html}"
        );
        assert!(
            html.contains("sm:grid-cols-[1fr_auto_1fr]"),
            "desktop: three-column grid: {html}"
        );

        // Container should span full width (no max-w constraint)
        let footer_start = html.find("<footer").expect("footer tag");
        let div_start = html[footer_start..].find("<div").expect("main div");
        let actual_div_start = footer_start + div_start;
        let div_end = actual_div_start + html[actual_div_start..].find('>').expect("div end");
        let div_tag = &html[actual_div_start..div_end];
        assert!(
            !div_tag.contains("max-w-"),
            "footer spans full width, no max-w: {div_tag}"
        );

        // Nav should be centred and in the middle column on desktop
        let nav_start = html.find("<nav").expect("nav tag");
        let nav_end = nav_start + html[nav_start..].find('>').expect("nav end");
        let nav_tag = &html[nav_start..nav_end];
        assert!(
            nav_tag.contains("justify-center"),
            "nav: centred links: {nav_tag}"
        );
        assert!(
            nav_tag.contains("sm:col-start-2"),
            "nav: middle column on desktop: {nav_tag}"
        );
    }

    #[test]
    fn credits_at_the_footers_right_edge_on_desktop() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());

        // Credits should sit at the footer's right edge
        let ul_start = html.find("<ul").expect("credits list");
        let ul_end = ul_start + html[ul_start..].find('>').expect("ul end");
        let ul_tag = &html[ul_start..ul_end];
        assert!(ul_tag.contains("text-center"), "ul: centred text: {ul_tag}");
        assert!(
            ul_tag.contains("sm:justify-self-end"),
            "ul: at right edge on desktop: {ul_tag}"
        );
        assert!(
            !ul_tag.contains("sm:justify-self-start"),
            "ul: not aligned to start: {ul_tag}"
        );
        assert!(
            !ul_tag.contains("sm:border-l"),
            "ul: no left border (vertical rule): {ul_tag}"
        );
        assert!(!ul_tag.contains("sm:ml-8"), "ul: no left margin: {ul_tag}");
        assert!(!ul_tag.contains("sm:pl-8"), "ul: no left padding: {ul_tag}");
    }

    #[test]
    fn the_credits_are_centered_on_each_other_at_every_width() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=credits() /> }.to_html());
        let start = html.find("<ul").expect("the credits list");
        let tag_end = start + html[start..].find('>').expect("its end");
        let tag = &html[start..tag_end];
        assert!(tag.contains("text-center"), "{tag}");
    }

    #[test]
    fn no_links_renders_no_nav() {
        let html =
            Owner::new().with(|| view! { <Footer links=vec![] credits=credits() /> }.to_html());
        assert!(!html.contains("<a "), "{html}");
        assert!(!html.contains("<nav"), "no empty landmark: {html}");
        assert!(html.contains("Dictionary: DPD, CC BY-NC 4.0"), "{html}");
    }

    #[test]
    fn one_link_renders_one_anchor() {
        let html = Owner::new().with(|| {
            view! {
                <Footer links=vec![FooterLink { label: "Terms", href: "/terms" }] />
            }
            .to_html()
        });
        assert_eq!(html.matches("<a ").count(), 1, "{html}");
        assert!(html.contains(r#"<a href="/terms""#), "{html}");
        assert_eq!(html.matches("<nav").count(), 1, "{html}");
    }

    #[test]
    fn no_credits_renders_no_line_and_no_br() {
        let html =
            Owner::new().with(|| view! { <Footer links=links() credits=vec![] /> }.to_html());
        assert!(!html.contains("<ul"), "{html}");
        assert!(!html.contains("<li"), "no empty credits block: {html}");
    }

    #[test]
    fn credits_are_optional() {
        let html = Owner::new().with(|| view! { <Footer links=links() /> }.to_html());
        assert!(html.contains(r#"<a href="/terms""#), "{html}");
        assert!(!html.contains("<p"), "no credits, no block: {html}");
    }

    #[test]
    fn one_credit_line_has_no_br() {
        let html = Owner::new().with(|| {
            view! { <Footer links=links() credits=vec!["Texts © SuttaCentral"] /> }.to_html()
        });
        assert!(html.contains("Texts © SuttaCentral"), "{html}");
        assert_eq!(
            html.matches("<li").count(),
            1,
            "one <li> for the single credit: {html}"
        );
        assert_eq!(
            html.matches("<ul").count(),
            1,
            "one <ul> wrapping the credit: {html}"
        );
    }

    #[test]
    fn three_or_more_credit_lines_each_get_their_own_br() {
        let html = Owner::new().with(|| {
            view! {
                <Footer
                    links=links()
                    credits=vec!["Texts © SuttaCentral", "Dictionary: DPD", "Icons: Lucide"]
                />
            }
            .to_html()
        });
        assert!(html.contains("Texts © SuttaCentral"), "{html}");
        assert!(html.contains("Dictionary: DPD"), "{html}");
        assert!(html.contains("Icons: Lucide"), "{html}");
        assert_eq!(
            html.matches("<li").count(),
            3,
            "three <li>, one per credit line: {html}"
        );
        assert_eq!(
            html.matches("<ul").count(),
            1,
            "one <ul> wrapping the credits: {html}"
        );
    }

    #[test]
    fn the_nav_aria_label_follows_the_language() {
        Owner::new().with(|| {
            let html = view! { <Footer links=links() credits=credits() /> }.to_html();
            assert!(html.contains(r#"aria-label="Footer navigation""#), "{html}");
            language_signal().set(Language::Fr);
            let html = view! { <Footer links=links() credits=credits() /> }.to_html();
            assert!(
                html.contains(r#"aria-label="Navigation de pied de page""#),
                "{html}"
            );
        });
    }

    /// 044 (user, 2026-10-05): a `leading` view sits at the footer's left
    /// — first in the column, first grid column on desktop, before the nav.
    #[test]
    fn leading_sits_first_at_the_left_on_desktop() {
        let html = Owner::new().with(|| {
            view! { <Footer links=links() leading=|| view! { <span id="badge">"dev · abc1234"</span> } /> }
                .to_html()
        });
        let badge = html.find(r#"id="badge""#).expect(&html);
        let nav = html.find("<nav").expect(&html);
        assert!(badge < nav, "{html}");
        let start = html[..badge].rfind("<div").expect(&html);
        let tag = &html[start..start + html[start..].find('>').expect(&html)];
        assert!(tag.contains("sm:col-start-1"), "{tag}");
        assert!(tag.contains("sm:justify-self-start"), "{tag}");
        assert!(!tag.contains("justify-self-end"), "{tag}");
    }

    /// No `leading`: nothing rendered for it, the nav still first.
    #[test]
    fn no_leading_renders_no_first_column_item() {
        let html = Owner::new().with(|| view! { <Footer links=links() /> }.to_html());
        assert!(!html.contains("sm:col-start-1"), "{html}");
    }
}
