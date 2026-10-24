//! Surfaces (079): [`Card`], the box everything sits in; [`Section`], a
//! titled card with its actions; [`PageHeader`], a screen's title row.
//!
//! On a phone the actions of a [`Section`] and a [`PageHeader`] wrap under
//! the title rather than squeezing it.

use leptos::either::Either;
use leptos::prelude::*;

use crate::button::{LinkButton, Size, Variant};
use crate::icon::{Icon, IconName};
use crate::tone::{Tone, edge_classes, text_classes};

fn padding(size: Size) -> &'static str {
    match size {
        Size::Sm => "p-3",
        Size::Md => "p-5",
        Size::Lg => "p-6",
    }
}

/// A rounded box on the card surface. `tone` colours its edge (a danger
/// zone); `raised` lifts it with a shadow; `href` makes the whole card a
/// link, which then shows its focus and hover on the edge.
///
/// A link card holds no interactive content (a button, another link): it
/// would nest inside the `<a>`, which HTML forbids and keyboards cannot reach.
#[component]
pub fn Card(
    /// Lifts the card with a shadow.
    #[prop(optional)]
    raised: bool,
    /// Colours the edge; the plain card edge when absent.
    #[prop(optional)]
    tone: Option<Tone>,
    /// The inner space; medium by default.
    #[prop(optional)]
    padding: Size,
    /// Makes the whole card a link to this address.
    #[prop(optional, into)]
    href: Option<String>,
    /// Layout classes added (gap, grid placement) — never colours.
    #[prop(optional, into)]
    layout: String,
    /// The content.
    children: Children,
) -> impl IntoView {
    let edge = tone.map_or("border-card-edge", edge_classes);
    let shadow = if raised { "shadow-lg" } else { "" };
    let base = format!(
        "flex flex-col rounded-2xl border bg-card {edge} {shadow} {} {layout}",
        self::padding(padding)
    );
    match href {
        Some(href) => Either::Left(view! {
            <a
                href=href
                class=format!(
                    "{base} no-underline text-ink transition-colors hover:border-gold focus-visible:outline-2 focus-visible:outline-gold",
                )
            >
                {children()}
            </a>
        }),
        None => Either::Right(view! { <div class=base>{children()}</div> }),
    }
}

/// A card with a title row: its title, an optional description, and its
/// actions on the right (under the title on a phone).
#[component]
pub fn Section(
    /// The title, as an `<h2>`.
    #[prop(into)]
    title: String,
    /// One line under the title.
    #[prop(optional, into)]
    description: Option<String>,
    /// The section's actions: buttons, a switch.
    #[prop(optional)]
    actions: Option<Children>,
    /// Colours the card's edge (a danger zone).
    #[prop(optional)]
    tone: Option<Tone>,
    /// The content under the title row.
    children: Children,
) -> impl IntoView {
    let description = description.map(|d| view! { <p class="m-0 text-sm text-muted">{d}</p> });
    let actions =
        actions.map(|a| view! { <div class="flex flex-wrap items-center gap-2">{a()}</div> });
    // Any tone but Neutral colours the title as it colours the edge.
    let heading = match tone {
        Some(t) if t != Tone::Neutral => format!("m-0 text-base font-semibold {}", text_classes(t)),
        _ => "m-0 text-base font-semibold text-heading".to_string(),
    };
    view! {
        <Card tone=tone.unwrap_or_default() layout="gap-4">
            <div class="flex flex-wrap items-start justify-between gap-3">
                <div class="flex min-w-0 flex-col gap-1">
                    <h2 class=heading>{title}</h2>
                    {description}
                </div>
                {actions}
            </div>
            {children()}
        </Card>
    }
}

/// A screen's title row: an optional way back above, the `<h1>`, a subtitle,
/// and the screen's main actions on the right (under it on a phone).
#[component]
pub fn PageHeader(
    /// The screen's title, as its `<h1>`.
    #[prop(into)]
    title: String,
    /// One line under the title.
    #[prop(optional, into)]
    subtitle: Option<String>,
    /// The way back: its label and address (a breadcrumb of one step).
    #[prop(optional)]
    back: Option<(String, String)>,
    /// The screen's main actions.
    #[prop(optional)]
    actions: Option<Children>,
) -> impl IntoView {
    let back = back.map(|(label, href)| {
        view! {
            <LinkButton variant=Variant::Ghost size=Size::Sm href=href layout="self-start -ml-2">
                <Icon name=IconName::ChevronLeft size=14 />
                {label}
            </LinkButton>
        }
    });
    let subtitle = subtitle.map(|s| view! { <p class="m-0 text-sm text-muted">{s}</p> });
    let actions =
        actions.map(|a| view! { <div class="flex flex-wrap items-center gap-2">{a()}</div> });
    view! {
        <header class="flex flex-col gap-2">
            {back} <div class="flex flex-wrap items-end justify-between gap-4">
                <div class="flex min-w-0 flex-col gap-1.5">
                    <h1 class="m-0 text-2xl font-semibold text-heading">{title}</h1>
                    {subtitle}
                </div>
                {actions}
            </div>
        </header>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn html(view: impl FnOnce() -> AnyView) -> String {
        Owner::new().with(|| view().to_html())
    }

    #[test]
    fn a_card_is_a_box_and_a_link_card_is_a_link() {
        let plain = html(|| view! { <Card>"x"</Card> }.into_any());
        assert!(
            plain.starts_with("<div") && plain.contains("border-card-edge"),
            "{plain}"
        );
        let link = html(|| view! { <Card href="/apps/1">"x"</Card> }.into_any());
        assert!(
            link.starts_with("<a") && link.contains(r#"href="/apps/1""#),
            "{link}"
        );
        assert!(link.contains("focus-visible:outline-gold"), "{link}");
    }

    #[test]
    fn a_danger_card_has_a_danger_edge_and_a_raised_one_a_shadow() {
        let out = html(|| {
            view! {
                <Card tone=Tone::Danger raised=true>
                    "x"
                </Card>
            }
            .into_any()
        });
        assert!(
            out.contains("border-danger") && out.contains("shadow-lg"),
            "{out}"
        );
    }

    #[test]
    fn a_section_titles_its_card_and_carries_its_actions() {
        let out = html(|| {
            view! {
                <Section
                    title="Client secret"
                    description="Rotate it"
                    actions=Box::new(|| view! { <span>"act"</span> }.into_any())
                >
                    "body"
                </Section>
            }
            .into_any()
        });
        assert!(
            out.contains("<h2") && out.contains("Client secret"),
            "{out}"
        );
        assert!(
            out.contains("Rotate it") && out.contains("act") && out.contains("body"),
            "{out}"
        );
        assert!(out.contains("flex-wrap"), "actions wrap on a phone: {out}");
    }

    #[test]
    fn a_page_header_has_one_h1_and_its_way_back() {
        let out = html(|| {
            view! {
                <PageHeader
                    title="Applications"
                    subtitle="What lsh owns"
                    back=("Back".to_string(), "/apps".to_string())
                />
            }
            .into_any()
        });
        assert_eq!(out.matches("<h1").count(), 1, "{out}");
        assert!(
            out.contains(r#"href="/apps""#) && out.contains("What lsh owns"),
            "{out}"
        );
    }
}
