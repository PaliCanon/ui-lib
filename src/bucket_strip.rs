//! `BucketStrip`: a strip of labelled counts, each a box or a link, knowing neither stage nor term.
//!
//! The strip is generic: it knows neither "stage" nor "term", only labelled
//! counts. Phone: a grid of three columns (two rows of three for six cells);
//! from `sm:` up, one row, every cell the same width. A cell with a target
//! is a real link (new tab, middle click) on the [`Variant::Tile`] look; one
//! without is a plain box with the tile's resting surface and no state, since
//! it does nothing. In the DOM the label comes before the count (a screen
//! reader hears the name before the number); the count is shown above it.

use leptos::prelude::*;

use crate::button::{LinkButton, Size, TILE_SURFACE, Variant};

/// One cell of a [`BucketStrip`]: a label, a count and, optionally, where the
/// cell leads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket {
    /// What is counted, e.g. "Due today".
    pub label: String,
    /// How many; zero is shown as `0`.
    pub count: usize,
    /// Where the cell leads; none (or a blank string, a link to nowhere)
    /// makes it a plain box, not a link; a padded one is trimmed.
    pub to: Option<String>,
}

/// A strip of labelled counts: `cells` in order, under an optional `heading`.
/// Nothing at all is rendered for no cells (no empty landmark).
#[component]
pub fn BucketStrip(
    /// The cells, in order.
    #[prop(into)]
    cells: Vec<Bucket>,
    /// A heading above the strip (an `<h2>`); none by default.
    #[prop(optional, into)]
    heading: Option<String>,
) -> impl IntoView {
    (!cells.is_empty()).then(|| {
        view! {
            <section>
                {heading
                    .map(|h| {
                        view! { <h2 class="mb-2 text-sm font-medium text-muted">{h}</h2> }
                    })}
                <ul class="grid grid-cols-3 gap-2 sm:flex sm:gap-3">
                    {cells
                        .into_iter()
                        .map(|cell| {
                            view! {
                                <li class="sm:flex-1">
                                    <Cell cell=cell />
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </section>
        }
    })
}

/// One cell: a link when it has a target, a plain box when not.
#[component]
fn Cell(cell: Bucket) -> impl IntoView {
    let Bucket { label, count, to } = cell;
    // A blank `to` is none: a link to nowhere does nothing (review rounds 1
    // and 2); a padded one keeps its trimmed href.
    match to.as_deref().map(str::trim).filter(|href| !href.is_empty()) {
        Some(href) => {
            view! { <LinkCell href=href.to_owned() label=label count=count /> }.into_any()
        }
        None => view! { <PlainCell label=label count=count /> }.into_any(),
    }
}

/// A cell that leads somewhere: a real `<a href>` filling its box.
#[component]
fn LinkCell(href: String, label: String, count: usize) -> impl IntoView {
    view! {
        <LinkButton href=href variant=Variant::Tile size=Size::Md layout="h-full w-full">
            <CellBody label=label count=count />
        </LinkButton>
    }
}

/// A cell that does nothing: the tile's resting surface alone, no state.
#[component]
fn PlainCell(label: String, count: usize) -> impl IntoView {
    view! {
        <div class=format!("flex h-full {TILE_SURFACE}")>
            <CellBody label=label count=count />
        </div>
    }
}

/// What a cell shows: the label first in the DOM, the count above it on screen.
#[component]
fn CellBody(label: String, count: usize) -> impl IntoView {
    view! {
        <span class="text-xs text-muted">{label}</span>
        <span class="order-first text-2xl font-semibold text-heading">{count}</span>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::button::TILE_SURFACE;

    fn bucket(label: &str, count: usize, to: Option<&str>) -> Bucket {
        Bucket {
            label: label.to_owned(),
            count,
            to: to.map(str::to_owned),
        }
    }

    fn render(cells: Vec<Bucket>, heading: Option<&str>) -> String {
        Owner::new().with(|| match heading {
            Some(h) => view! { <BucketStrip cells=cells heading=h /> }.to_html(),
            None => view! { <BucketStrip cells=cells /> }.to_html(),
        })
    }

    /// The opening tag that contains `needle`, from its `<` to its `>`.
    fn tag_around(html: &str, needle: &str) -> String {
        let at = html.find(needle).expect(html);
        let start = html[..at].rfind('<').expect(html);
        let end = at + html[at..].find('>').expect(html);
        html[start..=end].to_owned()
    }

    #[test]
    fn cells_come_in_the_given_order() {
        let html = render(
            vec![
                bucket("Alpha", 1, None),
                bucket("Beta", 2, None),
                bucket("Gamma", 3, None),
            ],
            None,
        );
        let a = html.find("Alpha").expect(&html);
        let b = html.find("Beta").expect(&html);
        let c = html.find("Gamma").expect(&html);
        assert!(a < b && b < c, "{html}");
        assert_eq!(html.matches("<li").count(), 3, "{html}");
    }

    #[test]
    fn a_cell_with_a_target_is_a_real_link_in_the_tile_look() {
        let html = render(vec![bucket("Due", 4, Some("/review?stage=1"))], None);
        let a = tag_around(&html, "<a ");
        assert!(a.contains(r#"href="/review?stage=1""#), "{a}");
        assert!(a.contains("rounded-xl border border-line bg-chip"), "{a}");
        assert!(a.contains("hover:border-gold/60"), "{a}");
        assert!(a.contains("min-h-11"), "{a}");
        assert!(!html.contains("<div"), "no plain cell: {html}");
    }

    #[test]
    fn a_cell_without_a_target_is_a_plain_box_that_reacts_to_nothing() {
        let html = render(vec![bucket("Done", 9, None)], None);
        assert!(!html.contains("<a"), "{html}");
        let div = tag_around(&html, TILE_SURFACE);
        assert!(div.starts_with("<div"), "{div}");
        assert!(div.contains(TILE_SURFACE), "{div}");
        for state in ["hover:", "active:", "cursor-pointer", "focus-visible:"] {
            assert!(!div.contains(state), "{state} in {div}");
        }
    }

    #[test]
    fn an_empty_target_is_a_plain_cell_not_a_link_to_nowhere() {
        let html = render(vec![bucket("Done", 9, Some(""))], None);
        assert!(!html.contains("<a"), "{html}");
        assert!(!html.contains("href"), "{html}");
        let div = tag_around(&html, TILE_SURFACE);
        assert!(div.starts_with("<div"), "{div}");
        for state in ["hover:", "active:", "cursor-pointer", "focus-visible:"] {
            assert!(!div.contains(state), "{state} in {div}");
        }
    }

    #[test]
    fn a_blank_target_is_a_plain_cell_too() {
        for to in [" ", "\t", " \n "] {
            let html = render(vec![bucket("Done", 9, Some(to))], None);
            assert!(!html.contains("<a"), "{to:?}: {html}");
            assert!(!html.contains("href"), "{to:?}: {html}");
            let div = tag_around(&html, TILE_SURFACE);
            assert!(div.starts_with("<div"), "{div}");
            for state in ["hover:", "active:", "cursor-pointer", "focus-visible:"] {
                assert!(!div.contains(state), "{state} in {div}");
            }
        }
    }

    #[test]
    fn a_padded_target_is_trimmed_in_the_link() {
        let html = render(vec![bucket("Due", 4, Some(" #x "))], None);
        let a = tag_around(&html, "<a ");
        assert!(a.contains(r##"href="#x""##), "{a}");
    }

    #[test]
    fn a_trimmed_link_cell_keeps_its_label_and_count() {
        let html = render(vec![bucket("Week", 4, Some(" #x "))], None);
        let open = html.find("<a ").expect(&html);
        let close = open + html[open..].find("</a>").expect(&html);
        let inner = &html[open..close];
        let label = inner.find("Week").expect(inner);
        let count = inner.find(">4<").expect(inner);
        assert!(label < count, "{inner}");
    }

    #[test]
    fn the_plain_cell_has_one_display_utility() {
        let html = render(vec![bucket("Done", 9, None)], None);
        let div = tag_around(&html, TILE_SURFACE);
        let tokens: Vec<&str> = div.split(['"', ' ']).collect();
        assert!(tokens.contains(&"flex"), "{div}");
        assert!(!tokens.contains(&"inline-flex"), "{div}");
    }

    #[test]
    fn the_label_is_read_before_the_count_and_the_count_is_shown_above() {
        for to in [None, Some("/x")] {
            let html = render(vec![bucket("Learning", 7, to)], None);
            let label = html.find("Learning").expect(&html);
            let count = html.find('7').expect(&html);
            assert!(label < count, "{html}");
            let span = tag_around(&html, "order-first");
            assert!(span.contains("text-2xl font-semibold"), "{span}");
            assert!(label < html.find("order-first").expect(&html), "{html}");
            assert!(
                tag_around(&html, "Learning").contains("text-xs text-muted"),
                "{html}"
            );
        }
    }

    #[test]
    fn a_heading_is_an_h2_when_given() {
        let html = render(vec![bucket("A", 1, None)], Some("Your stages"));
        assert!(html.contains("<h2"), "{html}");
        assert!(html.contains("Your stages"), "{html}");
        assert!(html.find("<h2").expect(&html) < html.find("<ul").expect(&html));
    }

    #[test]
    fn no_heading_means_no_h2() {
        let html = render(vec![bucket("A", 1, None)], None);
        assert!(html.contains("<section"), "{html}");
        assert!(!html.contains("<h2"), "{html}");
    }

    #[test]
    fn no_cells_render_nothing_even_with_a_heading() {
        for heading in [None, Some("Your stages")] {
            let html = render(vec![], heading);
            assert!(!html.contains("<section"), "{html}");
            assert!(!html.contains("<ul"), "{html}");
            assert!(!html.contains("<h2"), "{html}");
        }
    }

    #[test]
    fn a_zero_count_still_renders_its_cell() {
        let html = render(
            vec![bucket("Empty", 0, None), bucket("Next", 5, Some("/n"))],
            None,
        );
        assert!(html.contains("Empty"), "{html}");
        assert!(html.contains(">0<") || html.contains(">0</"), "{html}");
        assert_eq!(html.matches("<li").count(), 2, "{html}");
    }

    #[test]
    fn the_list_is_a_grid_on_a_phone_and_a_row_from_sm_up() {
        let html = render(vec![bucket("A", 1, None), bucket("B", 2, None)], None);
        let ul = tag_around(&html, "<ul");
        for class in ["grid", "grid-cols-3", "gap-2", "sm:flex", "sm:gap-3"] {
            assert!(ul.split(['"', ' ']).any(|t| t == class), "{class} in {ul}");
        }
        assert_eq!(html.matches(r#"class="sm:flex-1""#).count(), 2, "{html}");
    }

    #[test]
    fn labels_are_escaped() {
        let html = render(vec![bucket("<b>x</b>", 1, None)], Some("<i>h</i>"));
        assert!(!html.contains("<b>x"), "{html}");
        assert!(!html.contains("<i>h"), "{html}");
        assert!(html.contains("&lt;b&gt;"), "{html}");
    }
}
