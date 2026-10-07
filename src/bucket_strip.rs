//! `BucketStrip`: a strip of labelled counts, each a box or a link, knowing neither stage nor term.
//!
//! The strip is generic: it knows neither "stage" nor "term", only labelled
//! counts. Phone: a grid of three columns (two rows of three for six cells);
//! from `sm:` up, one row, every cell the same width — or, with
//! [`Columns::Three`], the grid of three at every width. A cell with a target
//! is a real link (new tab, middle click) on the [`Variant::Tile`] look; one
//! without is a plain box with the tile's resting surface and no state, since
//! it does nothing. In the DOM the label comes before the count (a screen
//! reader hears the name before the number); the count is shown above it.
//!
//! With `on_select` the strip filters in place instead of navigating: every
//! cell is a `<button aria-pressed>` in the same tile look, the `selected`
//! one pressed, and the cells' `to` are ignored. The pressed look belongs to
//! the variant, not to this file.

use leptos::prelude::*;

use crate::button::{LinkButton, Size, TILE_SURFACE, Variant, classes, tile_spacing};

/// How a [`BucketStrip`] lays its cells out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Columns {
    /// Three columns on a phone, one row of equal cells from `sm:` up.
    #[default]
    Auto,
    /// Three columns at every width (two rows of three for six cells).
    Three,
}

impl Columns {
    /// The list's classes: a grid of three, a row from `sm:` up for `Auto`.
    fn list_classes(self) -> &'static str {
        match self {
            Self::Auto => "grid grid-cols-3 gap-2 sm:flex sm:gap-3",
            Self::Three => "grid grid-cols-3 gap-2 sm:gap-3",
        }
    }

    /// A cell's item classes: equal widths in the `Auto` row, none in a grid.
    fn item_classes(self) -> Option<&'static str> {
        match self {
            Self::Auto => Some("sm:flex-1"),
            Self::Three => None,
        }
    }
}

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
    /// The cells' size: Md by default; Sm is compact (a smaller count and label); Lg draws an Lg box with the Md body.
    #[prop(optional)]
    size: Size,
    /// The layout: [`Columns::Auto`] by default (a row from `sm:` up);
    /// [`Columns::Three`] keeps three columns at every width.
    #[prop(optional)]
    columns: Columns,
    /// Called with the cell's index when a cell is pressed; given, every cell is a button and the cells' `to` are ignored.
    ///
    /// Selection is by index, so it follows the cells' order.
    #[prop(optional)]
    on_select: Option<Callback<usize>>,
    /// The index of a cell in `cells`, shown pressed; keep the cells' order
    /// stable while one is selected; omit it and nothing is pressed.
    #[prop(optional, into)]
    selected: Option<Signal<Option<usize>>>,
) -> impl IntoView {
    let (list_class, item_class) = (columns.list_classes(), columns.item_classes());
    (!cells.is_empty()).then(|| {
        view! {
            <section>
                {heading
                    .map(|h| {
                        view! { <h2 class="mb-2 text-sm font-medium text-muted">{h}</h2> }
                    })}
                <ul class=list_class>
                    {cells
                        .into_iter()
                        .enumerate()
                        .map(|(index, cell)| {
                            view! {
                                <li class=item_class>
                                    <Cell
                                        cell=cell
                                        size=size
                                        index=index
                                        on_select=on_select
                                        selected=selected
                                    />
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </section>
        }
    })
}

/// One cell: a button when the strip selects, else a link when it has a
/// target, a plain box when not.
#[component]
fn Cell(
    cell: Bucket,
    size: Size,
    index: usize,
    on_select: Option<Callback<usize>>,
    selected: Option<Signal<Option<usize>>>,
) -> impl IntoView {
    let Bucket { label, count, to } = cell;
    if let Some(on_select) = on_select {
        return view! {
            <SelectCell
                label=label
                count=count
                size=size
                index=index
                on_select=on_select
                selected=selected
            />
        }
        .into_any();
    }
    // A blank `to` is none: a link to nowhere does nothing (review rounds 1
    // and 2); a padded one keeps its trimmed href.
    match to.as_deref().map(str::trim).filter(|href| !href.is_empty()) {
        Some(href) => {
            view! { <LinkCell href=href.to_owned() label=label count=count size=size /> }.into_any()
        }
        None => view! { <PlainCell label=label count=count size=size /> }.into_any(),
    }
}

/// A cell that leads somewhere: a real `<a href>` filling its box.
#[component]
fn LinkCell(href: String, label: String, count: usize, size: Size) -> impl IntoView {
    view! {
        <LinkButton href=href variant=Variant::Tile size=size layout="h-full w-full">
            <CellBody label=label count=count size=size />
        </LinkButton>
    }
}

/// A cell that selects: a `<button aria-pressed>` in the tile look, pressed
/// while it is the selected one. The variant draws the pressed state.
#[component]
fn SelectCell(
    label: String,
    count: usize,
    size: Size,
    index: usize,
    on_select: Callback<usize>,
    selected: Option<Signal<Option<usize>>>,
) -> impl IntoView {
    let pressed = move || selected.is_some_and(|s| s.get() == Some(index));
    view! {
        <button
            type="button"
            class=classes(Variant::Tile, size, "h-full w-full")
            aria-pressed=move || pressed().to_string()
            on:click=move |_| on_select.run(index)
        >
            <CellBody label=label count=count size=size />
        </button>
    }
}

/// A cell that does nothing: the tile's resting surface alone, no state.
#[component]
fn PlainCell(label: String, count: usize, size: Size) -> impl IntoView {
    view! {
        <div class=format!("flex h-full {TILE_SURFACE} {}", tile_spacing(size))>
            <CellBody label=label count=count size=size />
        </div>
    }
}

/// Size-specific classes for the cell body.
fn body_classes(size: Size) -> (&'static str, &'static str) {
    match size {
        Size::Sm => (
            "text-[11px] leading-tight text-muted",
            "order-first text-base font-semibold text-heading",
        ),
        Size::Md | Size::Lg => (
            "text-xs text-muted",
            "order-first text-2xl font-semibold text-heading",
        ),
    }
}

/// What a cell shows: the label first in the DOM, the count above it on screen.
#[component]
fn CellBody(label: String, count: usize, size: Size) -> impl IntoView {
    let (label_class, count_class) = body_classes(size);
    view! {
        <span class=label_class>{label}</span>
        <span class=count_class>{count}</span>
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

    fn render_with_size(cells: Vec<Bucket>, heading: Option<&str>, size: Size) -> String {
        Owner::new().with(|| match heading {
            Some(h) => view! { <BucketStrip cells=cells heading=h size=size /> }.to_html(),
            None => view! { <BucketStrip cells=cells size=size /> }.to_html(),
        })
    }

    fn render_selectable(cells: Vec<Bucket>, selected: Option<usize>) -> String {
        Owner::new().with(|| {
            let on_select = Callback::new(|_: usize| {});
            let selected = Signal::derive(move || selected);
            view! { <BucketStrip cells=cells size=Size::Sm on_select=on_select selected=selected /> }
                .to_html()
        })
    }

    fn render(cells: Vec<Bucket>, heading: Option<&str>) -> String {
        render_with_size(cells, heading, Size::Md)
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
        assert!(
            a.contains("rounded-xl border border-gold/25 bg-chip"),
            "{a}"
        );
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
    fn three_columns_stay_a_grid_of_three_at_every_width() {
        let cells = vec![bucket("A", 1, None), bucket("B", 2, None)];
        let html = Owner::new()
            .with(|| view! { <BucketStrip cells=cells columns=Columns::Three /> }.to_html());
        let ul = tag_around(&html, "<ul");
        let tokens: Vec<&str> = ul.split(['"', ' ']).collect();
        for class in ["grid", "grid-cols-3", "gap-2", "sm:gap-3"] {
            assert!(tokens.contains(&class), "{class} in {ul}");
        }
        assert!(!tokens.contains(&"sm:flex"), "{ul}");
        assert!(!html.contains("sm:flex-1"), "{html}");
        assert_eq!(html.matches("<li").count(), 2, "{html}");
    }

    #[test]
    fn the_default_columns_are_auto() {
        assert_eq!(Columns::default(), Columns::Auto);
        let cells = vec![bucket("A", 1, None)];
        let html = Owner::new()
            .with(|| view! { <BucketStrip cells=cells columns=Columns::Auto /> }.to_html());
        assert!(tag_around(&html, "<ul").contains("sm:flex"), "{html}");
        assert!(html.contains(r#"class="sm:flex-1""#), "{html}");
    }

    #[test]
    fn labels_are_escaped() {
        let html = render(vec![bucket("<b>x</b>", 1, None)], Some("<i>h</i>"));
        assert!(!html.contains("<b>x"), "{html}");
        assert!(!html.contains("<i>h"), "{html}");
        assert!(html.contains("&lt;b&gt;"), "{html}");
    }

    #[test]
    fn default_render_has_md_count_size() {
        let html = render(vec![bucket("Default", 5, None)], None);
        assert!(
            html.contains("text-2xl"),
            "default should have text-2xl count: {html}"
        );
    }

    #[test]
    fn sm_size_has_compact_count_and_label() {
        let html = render_with_size(vec![bucket("Compact", 3, None)], None, Size::Sm);
        assert!(
            html.contains("text-base"),
            "Sm should have text-base count: {html}"
        );
        assert!(
            html.contains("text-[11px]"),
            "Sm should have text-[11px] label: {html}"
        );
        assert!(
            !html.contains("text-2xl"),
            "Sm should not have text-2xl: {html}"
        );
    }

    #[test]
    fn sm_link_cell_carries_tile_sm_size_class() {
        let html = render_with_size(vec![bucket("Link", 2, Some("/test"))], None, Size::Sm);
        assert!(
            html.contains("min-h-8"),
            "Sm link cell should have min-h-8 from Tile Sm: {html}"
        );
        assert!(
            !html.contains("min-h-11"),
            "Sm should not have Md size class: {html}"
        );
    }

    #[test]
    fn with_on_select_every_cell_is_a_button_and_the_selected_one_is_pressed() {
        let html = render_selectable(
            vec![
                bucket("A", 1, Some("/a")),
                bucket("B", 2, None),
                bucket("C", 3, Some("/c")),
            ],
            Some(1),
        );
        assert!(!html.contains("<a "), "{html}");
        assert!(!html.contains("href"), "{html}");
        assert!(!html.contains("<div"), "{html}");
        assert_eq!(html.matches("<button").count(), 3, "{html}");
        assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 2, "{html}");
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1, "{html}");
        let pressed = tag_around(&html, r#"aria-pressed="true""#);
        assert!(pressed.starts_with("<button"), "{pressed}");
        assert!(pressed.contains(r#"type="button""#), "{pressed}");
        let on = html.find(r#"aria-pressed="true""#).expect(&html);
        assert!(html[on..].find(">B<").is_some() && html[..on].find(">C<").is_none());
    }

    #[test]
    fn a_selectable_cell_is_the_tile_at_the_strips_size() {
        let html = render_selectable(vec![bucket("A", 1, None)], None);
        let button = tag_around(&html, "<button");
        assert!(
            button.contains("rounded-xl border border-gold/25 bg-chip"),
            "{button}"
        );
        assert!(button.contains("aria-pressed:border-gold"), "{button}");
        assert!(button.contains("min-h-8"), "{button}");
        assert!(html.contains("text-base"), "{html}");
        assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 1, "{html}");
    }

    #[test]
    fn selectable_without_a_selection_presses_nothing() {
        let html = render_selectable(vec![bucket("A", 1, None), bucket("B", 2, None)], None);
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 0, "{html}");
        assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 2, "{html}");
    }

    #[test]
    fn a_selection_out_of_range_presses_nothing() {
        let html = render_selectable(vec![bucket("A", 1, None)], Some(7));
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 0, "{html}");
    }

    #[test]
    fn without_on_select_nothing_is_a_button() {
        let html = render(vec![bucket("A", 1, Some("/a")), bucket("B", 2, None)], None);
        assert!(!html.contains("<button"), "{html}");
        assert!(!html.contains(r#"aria-pressed=""#), "{html}");
        assert!(html.contains("<a "), "{html}");
    }

    #[test]
    fn selected_alone_does_not_make_buttons() {
        let html = Owner::new().with(|| {
            let selected = Signal::derive(|| Some(0));
            view! { <BucketStrip cells=vec![bucket("A", 1, Some("/a"))] selected=selected /> }
                .to_html()
        });
        assert!(!html.contains("<button"), "{html}");
        assert!(html.contains("<a "), "{html}");
    }

    #[test]
    fn the_compact_count_is_text_base_not_text_lg() {
        let html = render_with_size(vec![bucket("A", 3, None)], None, Size::Sm);
        assert!(html.contains("text-base"), "{html}");
        assert!(!html.contains("text-lg"), "{html}");
        let html = render_selectable(vec![bucket("A", 3, None)], None);
        assert!(html.contains("text-base"), "{html}");
    }

    #[test]
    fn the_plain_cell_takes_the_sizes_padding_and_gap() {
        for (size, spacing, other) in [
            (Size::Sm, ["gap-0", "py-1.5"], "py-2"),
            (Size::Md, ["gap-2", "py-2"], "py-1.5"),
        ] {
            let html = render_with_size(vec![bucket("Done", 9, None)], None, size);
            let div = tag_around(&html, TILE_SURFACE);
            let tokens: Vec<&str> = div.split(['"', ' ']).collect();
            for class in spacing {
                assert!(tokens.contains(&class), "{class} in {div}");
            }
            assert!(!tokens.contains(&other), "{other} in {div}");
        }
    }
}
