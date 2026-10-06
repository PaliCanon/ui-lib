//! `PaliTerm`: a Pāli term on two lines, generic, no PaliCanon data.
//!
//! Line 1 is the lemma in bold with an optional tag after it in
//! parentheses, and an optional end slot at the right; line 2 is the gloss,
//! one truncated line, and a second end slot at the right. It is not
//! interactive: the caller wraps it in a button or a link.

use leptos::prelude::*;

use crate::pali::Pali;

/// A Pāli term on two lines: the `lemma` in bold, an optional `tag` after
/// it in parentheses, the `gloss` under it (one line, truncated), and an
/// optional slot at the right of each line (`top_end`, `bottom_end`).
/// A block-level row, `w-full` by default (inside a centring parent such
/// as a `Tile` button it still takes the row's width); the ends sit flush
/// right. Not interactive: the caller wraps it in a button or a link. The
/// lemma is a [`Pali`] at body size, whose own size classes set its size; the
/// `<strong>` only adds the weight. The text props are reactive: a caller's
/// translated tag or gloss updates in place, without rendering the row again.
#[component]
pub fn PaliTerm(
    /// The Pāli word, shown in bold and truncated with an ellipsis when it
    /// does not fit; a string or a signal, escaped like any text.
    #[prop(into)]
    lemma: Signal<String>,
    /// What it means, on line 2: one line, truncated with an ellipsis; a
    /// string or a signal (a translation that follows the language).
    #[prop(into)]
    gloss: Signal<String>,
    /// A short label after the lemma in parentheses, muted and never
    /// truncated (a long lemma gives way first), e.g. `Day` shows
    /// `kho (Day)`; a string or a signal; none by default.
    #[prop(optional, into)]
    tag: Option<Signal<String>>,
    /// The right end of line 1 (a date, say), muted, flush right and never
    /// wrapping; nothing is rendered when absent. The slot runs once: a
    /// reactive end returns a view with a `move ||` inside. Ends are short
    /// (a date, a count): a long one squeezes the lemma column.
    #[prop(optional, into)]
    top_end: Option<ViewFn>,
    /// The right end of line 2 (a count, say), as `top_end`; nothing is
    /// rendered when absent, and line 2 still starts in column 1.
    #[prop(optional, into)]
    bottom_end: Option<ViewFn>,
    /// Layout classes only (margin, width, padding), never colours.
    #[prop(optional, into)]
    layout: String,
) -> impl IntoView {
    view! {
        <div class=format!(
            "grid w-full grid-cols-[minmax(0,1fr)_auto] items-baseline gap-x-4 gap-y-0.5 {layout}",
        )>
            <span class="flex min-w-0 items-baseline gap-1 text-heading">
                <span class="min-w-0 truncate">
                    <strong class="font-semibold">
                        <Pali>{move || lemma.get()}</Pali>
                    </strong>
                </span>
                {tag
                    .map(|tag| {
                        view! {
                            <span class="shrink-0 text-sm text-muted">
                                {move || format!("({})", tag.get())}
                            </span>
                        }
                    })}
            </span>
            {end(top_end)}
            <span class="col-start-1 min-w-0 truncate text-sm text-muted">
                {move || gloss.get()}
            </span>
            {end(bottom_end)}
        </div>
    }
}

/// One end slot: muted, small, flush right, never wrapping; nothing when absent.
fn end(slot: Option<ViewFn>) -> Option<impl IntoView> {
    let slot = slot?;
    Some(view! {
        <span class="justify-self-end text-right text-sm whitespace-nowrap text-muted">
            {slot.run()}
        </span>
    })
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn render(view: impl IntoView + 'static) -> String {
        Owner::new().with(|| view.to_html())
    }

    /// The opening tag that contains `needle`, from its `<` to its `>`.
    fn tag_around(html: &str, needle: &str) -> String {
        let at = html.find(needle).expect(html);
        let start = html[..at].rfind('<').expect(html);
        let end = at + html[at..].find('>').expect(html);
        html[start..=end].to_owned()
    }

    /// `html` without its tags: what a reader sees.
    fn text(html: &str) -> String {
        let mut out = String::new();
        let mut in_tag = false;
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                c if !in_tag => out.push(c),
                _ => {}
            }
        }
        out
    }

    fn plain() -> String {
        render(view! { <PaliTerm lemma="kho" gloss="indeed" /> })
    }

    #[test]
    fn the_lemma_is_bold_and_tagged_as_pali() {
        let html = plain();
        let strong = html.find("<strong").expect(&html);
        assert!(
            tag_around(&html, "<strong").contains("font-semibold"),
            "{html}"
        );
        let lang = html.find(r#"lang="pi""#).expect(&html);
        let lemma = html.find(">kho<").expect(&html);
        let close = html.find("</strong>").expect(&html);
        assert!(strong < lang && lang < lemma && lemma < close, "{html}");
    }

    #[test]
    fn the_lemma_uses_pali_at_body_size() {
        let html = plain();
        assert!(html.contains("font-serif"), "{html}");
        assert!(html.contains("text-[17px] leading-7"), "{html}");
    }

    #[test]
    fn a_tag_follows_the_lemma_in_parentheses_outside_the_truncating_span() {
        let html = render(view! { <PaliTerm lemma="kho" gloss="indeed" tag="Day" /> });
        let lemma_cell_end = html.find("</strong></span>").expect(&html);
        let tag = html.find("(Day)").expect(&html);
        assert!(lemma_cell_end < tag, "after the truncating span: {html}");
        let span = tag_around(&html, "(Day)");
        for class in ["shrink-0", "text-sm", "text-muted"] {
            assert!(
                span.split(['"', ' ']).any(|t| t == class),
                "{class} in {span}"
            );
        }
        assert!(!span.contains("truncate"), "{span}");
        assert_eq!(text(&html[lemma_cell_end..tag]), "", "right after: {html}");
    }

    #[test]
    fn the_lemma_cell_is_a_flex_row_whose_lemma_truncates_and_whose_tag_does_not() {
        let html = render(view! { <PaliTerm lemma="kho" gloss="indeed" tag="Day" /> });
        let cell = tag_around(&html, "items-baseline gap-1");
        for class in ["flex", "min-w-0", "items-baseline", "gap-1"] {
            assert!(
                cell.split(['"', ' ']).any(|t| t == class),
                "{class} in {cell}"
            );
        }
        let open = html
            .find(r#"<span class="min-w-0 truncate">"#)
            .expect(&html);
        let close = html.find("</strong></span>").expect(&html);
        assert!(open < close, "{html}");
        assert!(!html[open..close].contains("(Day)"), "{html}");
    }

    #[test]
    fn a_signal_prop_shows_its_value_set_before_the_render() {
        let html = Owner::new().with(|| {
            let lemma = RwSignal::new(String::from("kho"));
            let gloss = RwSignal::new(String::from("indeed"));
            let tag = RwSignal::new(String::from("Day"));
            lemma.set("citta".to_owned());
            gloss.set("mind".to_owned());
            tag.set("Week".to_owned());
            view! { <PaliTerm lemma=lemma gloss=gloss tag=tag /> }.to_html()
        });
        for shown in ["citta", "mind", "(Week)"] {
            assert!(html.contains(shown), "{shown} in {html}");
        }
        for old in ["kho", "indeed", "Day"] {
            assert!(!html.contains(old), "{old} in {html}");
        }
    }

    #[test]
    fn a_top_end_alone_leaves_the_gloss_in_column_one_and_no_trailing_cell() {
        let html =
            render(view! { <PaliTerm lemma="kho" gloss="indeed" top_end=|| "13 October 2026" /> });
        assert!(html.contains("13 October 2026"), "{html}");
        assert!(
            tag_around(&html, "indeed").contains("col-start-1"),
            "{html}"
        );
        let gloss_end = html.find("indeed").expect(&html);
        assert!(!html[gloss_end..].contains("<span"), "{html}");
    }

    #[test]
    fn no_tag_means_no_parentheses() {
        let html = plain();
        assert!(!text(&html).contains(['(', ')']), "{html}");
    }

    #[test]
    fn the_ends_are_rendered_when_given_and_never_wrap() {
        let html = render(view! {
            <PaliTerm
                lemma="kho"
                gloss="indeed"
                top_end=|| "13 October 2026"
                bottom_end=|| "Lapses 1"
            />
        });
        for text in ["13 October 2026", "Lapses 1"] {
            let span = tag_around(&html, text);
            assert!(span.contains("justify-self-end"), "{span}");
            assert!(span.contains("text-right"), "{span}");
            assert!(span.contains("whitespace-nowrap"), "{span}");
            assert!(span.contains("text-sm"), "{span}");
            assert!(span.contains("text-muted"), "{span}");
        }
        let top = html.find("13 October 2026").expect(&html);
        let gloss = html.find("indeed").expect(&html);
        let bottom = html.find("Lapses 1").expect(&html);
        assert!(top < gloss && gloss < bottom, "{html}");
    }

    #[test]
    fn absent_ends_render_no_cell() {
        let html = plain();
        assert!(!html.contains("whitespace-nowrap"), "{html}");
        assert_eq!(html.matches("<span").count(), 4, "{html}");
    }

    #[test]
    fn only_a_bottom_end_leaves_line_two_in_column_one() {
        let html =
            render(view! { <PaliTerm lemma="kho" gloss="indeed" bottom_end=|| "Lapses 1" /> });
        assert!(!html.contains("13 October"), "{html}");
        assert!(
            tag_around(&html, "indeed").contains("col-start-1"),
            "{html}"
        );
    }

    #[test]
    fn the_gloss_is_one_truncated_line_in_the_first_column() {
        let html = plain();
        let gloss = tag_around(&html, "indeed");
        for class in [
            "truncate",
            "min-w-0",
            "col-start-1",
            "text-sm",
            "text-muted",
        ] {
            assert!(
                gloss.split(['"', ' ']).any(|t| t == class),
                "{class} in {gloss}"
            );
        }
    }

    #[test]
    fn the_lemma_truncates_inside_its_cell() {
        let html = plain();
        let at = html.find("<strong").expect(&html);
        let span = tag_around(&html[..at], "min-w-0 truncate");
        for class in ["truncate", "min-w-0"] {
            assert!(
                span.split(['"', ' ']).any(|t| t == class),
                "{class} in {span}"
            );
        }
        assert!(html[..at].contains("text-heading"), "{html}");
    }

    #[test]
    fn the_grid_keeps_the_ends_at_their_width() {
        let html = plain();
        let grid = html[..html.find('>').expect(&html)].to_owned();
        assert!(html.starts_with("<div"), "{html}");
        assert!(grid.contains("grid-cols-[minmax(0,1fr)_auto]"), "{grid}");
        assert!(grid.contains("items-baseline"), "{grid}");
        assert!(grid.split(['"', ' ']).any(|t| t == "w-full"), "{grid}");
    }

    #[test]
    fn layout_classes_are_appended_to_the_grid() {
        let html = render(view! { <PaliTerm lemma="kho" gloss="indeed" layout="w-full px-4" /> });
        let grid = html[..html.find('>').expect(&html)].to_owned();
        assert!(grid.contains("w-full px-4"), "{grid}");
    }

    #[test]
    fn lemma_gloss_tag_and_ends_are_escaped() {
        let html = render(
            view! { <PaliTerm lemma="<b>x</b>" gloss="<b>y</b>" tag="<b>t</b>" top_end=|| "<b>u</b>" /> },
        );
        assert!(!html.contains("<b>"), "{html}");
        assert!(html.matches("&lt;b&gt;").count() >= 3, "{html}");
    }

    #[test]
    fn adds_no_state_classes() {
        let html = plain();
        for state in [
            "hover:",
            "focus",
            "active:",
            "disabled:",
            "aria-",
            "<button",
            "<a ",
        ] {
            assert!(!html.contains(state), "{state} in {html}");
        }
    }
}
