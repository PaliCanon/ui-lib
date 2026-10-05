//! `Pali`: a text primitive for Pāli terms, no behaviour: the language tag, the serif face and a line height that never clips diacritics.
//!
//! [`Pali`] is the one place that guarantees `lang="pi"` (screen readers
//! pick the right voice, the browser the right hyphenation and fonts), the
//! serif face and a line height that never clips the diacritics (ṭ ḍ ṃ ñ).
//! It has no colour and no state: the caller sets the colour from the
//! context (an inherited `text-*`), and layout classes through `layout`.

use leptos::prelude::*;

/// How large a [`Pali`] text is: the size and its line height together, so
/// the diacritics are never clipped at any size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaliSize {
    /// Running text, `text-[17px] leading-7`: a term inside a sentence.
    #[default]
    Body,
    /// A headword on its own line, `text-2xl leading-snug`.
    Lemma,
    /// The term a screen is about, `text-4xl leading-tight`.
    Display,
}

/// The size and line-height classes of a [`PaliSize`]: the one place the
/// sizes are written.
fn classes(size: PaliSize) -> &'static str {
    match size {
        PaliSize::Body => "text-[17px] leading-7",
        PaliSize::Lemma => "text-2xl leading-snug",
        PaliSize::Display => "text-4xl leading-tight",
    }
}

/// A Pāli text: a `<span lang="pi">` in the serif face, sized by `size`.
/// The one place that guarantees the language tag, the face and a line
/// height that never clips ṭ ḍ ṃ ñ. No colour (inherited) and no state: a
/// text has no behaviour.
#[component]
pub fn Pali(
    /// The size (and its line height); [`PaliSize::Body`] by default.
    #[prop(default = PaliSize::Body)]
    size: PaliSize,
    /// Layout classes only (margin, width, position), never colours.
    #[prop(into, default = String::new())]
    layout: String,
    /// The Pāli text; it is escaped like any text.
    children: Children,
) -> impl IntoView {
    view! {
        <span lang="pi" class=format!("font-serif {} {layout}", classes(size))>
            {children()}
        </span>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn render(view: impl IntoView + 'static) -> String {
        Owner::new().with(|| view.to_html())
    }

    #[test]
    fn tags_the_text_as_pali_on_a_span() {
        let html = render(view! { <Pali>"dhammo"</Pali> });
        assert!(html.contains("<span"), "{html}");
        assert!(html.contains(r#"lang="pi""#), "{html}");
        assert!(html.contains(">dhammo<"), "{html}");
    }

    #[test]
    fn uses_the_serif_face() {
        let html = render(view! { <Pali>"dhammo"</Pali> });
        assert!(html.contains("font-serif"), "{html}");
    }

    #[test]
    fn each_size_has_its_own_classes() {
        let body = render(view! { <Pali size=PaliSize::Body>"x"</Pali> });
        assert!(body.contains("text-[17px] leading-7"), "{body}");
        let lemma = render(view! { <Pali size=PaliSize::Lemma>"x"</Pali> });
        assert!(lemma.contains("text-2xl leading-snug"), "{lemma}");
        let display = render(view! { <Pali size=PaliSize::Display>"x"</Pali> });
        assert!(display.contains("text-4xl leading-tight"), "{display}");
    }

    #[test]
    fn a_size_does_not_carry_another_sizes_classes() {
        let lemma = render(view! { <Pali size=PaliSize::Lemma>"x"</Pali> });
        assert!(!lemma.contains("text-[17px]"), "{lemma}");
        assert!(!lemma.contains("text-4xl"), "{lemma}");
    }

    #[test]
    fn the_default_size_is_body() {
        assert_eq!(PaliSize::default(), PaliSize::Body);
        let html = render(view! { <Pali>"x"</Pali> });
        assert!(html.contains("text-[17px] leading-7"), "{html}");
    }

    #[test]
    fn a_layout_class_passes_through_verbatim() {
        let html = render(view! { <Pali layout="mt-2 max-w-prose">"x"</Pali> });
        assert!(html.contains("mt-2 max-w-prose"), "{html}");
    }

    #[test]
    fn adds_no_colour_class() {
        for size in [PaliSize::Body, PaliSize::Lemma, PaliSize::Display] {
            let html = render(view! { <Pali size=size>"x"</Pali> });
            for colour in [
                "text-ink",
                "text-muted",
                "text-gold",
                "text-subtle",
                "text-fg",
            ] {
                assert!(!html.contains(colour), "{colour} in {html}");
            }
        }
    }

    #[test]
    fn empty_children_render_an_empty_pali_span() {
        let html = render(view! { <Pali>""</Pali> });
        // Leptos writes an empty text node as one space (a hydration anchor).
        assert!(html.starts_with(r#"<span lang="pi""#), "{html}");
        let inner = &html[html.find('>').expect(&html) + 1..html.rfind("</span>").expect(&html)];
        assert_eq!(inner.trim(), "", "{html}");
    }

    #[test]
    fn escapes_its_text() {
        let html = render(view! { <Pali>"<b>x</b>"</Pali> });
        assert!(html.contains("&lt;b&gt;"), "{html}");
        assert!(!html.contains("<b>"), "{html}");
    }

    #[test]
    fn carries_no_state_classes() {
        let html = render(view! { <Pali>"x"</Pali> });
        for state in ["hover:", "focus", "active:", "disabled:", "aria-"] {
            assert!(!html.contains(state), "{state} in {html}");
        }
    }
}
