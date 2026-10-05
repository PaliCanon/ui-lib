//! A search card: one `<form role="search" method="get">` holding a large
//! field on top and a bottom row — the caller's controls (filters,
//! dropdowns) at its left, a round submit button with an up arrow at its
//! right — a soft halo behind it. Submitted by the browser itself: no
//! JavaScript is needed to search, and every control nested in its `children`
//! with a `name` travels in the same GET.
//!
//! Modelled on today's site's home composer (`web/src/components/
//! Composer.tsx`), without its AI button.

use leptos::prelude::*;

/// The card. The field's `name` (and `id`) is `name`; its text, label,
/// placeholder and the submit button's accessible name come from the
/// caller. Ids are unique per page: a second form on the page passes a
/// distinct `name`. The card is the `group/card` its submit button dims
/// against while the field is empty, a name the caller's own markup must
/// not reuse.
#[component]
pub fn SearchCard(
    /// The field's starting text (e.g. from `?q=`).
    #[prop(into)]
    value: String,
    /// The field's accessible name (a visually hidden `<label>`).
    #[prop(into)]
    label: Signal<String>,
    /// What the empty field shows.
    #[prop(into)]
    placeholder: Signal<String>,
    /// The round submit button's accessible name (it shows an arrow only).
    #[prop(into)]
    submit_label: Signal<String>,
    /// The field's form name and id.
    #[prop(default = "q")]
    name: &'static str,
    /// The field itself, for a caller that focuses it (a shortcut key).
    #[prop(optional)]
    input_ref: Option<NodeRef<leptos::html::Input>>,
    /// The bottom row's controls, at its left (they wrap on a narrow
    /// card); the submit button stays at the right.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let input_ref = input_ref.unwrap_or_default();
    view! {
        <div class="relative isolate">
            <div aria-hidden="true" class=HALO></div>
            <form role="search" method="get" class=CARD>
                <label for=name class="sr-only">
                    {move || label.get()}
                </label>
                <input
                    id=name
                    name=name
                    type="search"
                    node_ref=input_ref
                    value=value
                    maxlength="200"
                    autocomplete="off"
                    spellcheck="false"
                    placeholder=move || placeholder.get()
                    class="w-full bg-transparent px-3 pt-3 pb-5 text-lg text-heading outline-none placeholder:text-subtle"
                />
                <div class="flex items-center justify-between gap-2 px-1 pb-1">
                    <div class="flex min-w-0 flex-wrap items-center gap-2">
                        {children.map(|c| c())}
                    </div>
                    <SubmitArrow label=submit_label />
                </div>
            </form>
        </div>
    }
}

/// The soft gold halo behind the card (the theme's `--ps-halo`), kept
/// inside the card's own stacking context (`isolate`).
const HALO: &str = "pointer-events-none absolute -inset-x-6 -inset-y-16 -z-10 rounded-[50%] \
    bg-[radial-gradient(closest-side,var(--ps-halo),transparent)] sm:-inset-x-32 sm:-inset-y-24";

/// The card itself: card-edge border, the theme's card gradient, a deep
/// soft shadow, a gold ring while anything inside has focus.
const CARD: &str = "group/card rounded-2xl border border-card-edge bg-linear-to-b from-card-from to-card-to p-2 \
    shadow-[0_24px_60px_-30px_rgb(0_0_0/0.7)] transition \
    focus-within:border-gold/70 focus-within:ring-4 focus-within:ring-glow";

/// The round gold submit button, an up arrow inside; dimmed while the
/// card's field is empty (CSS only, `:has`). A raw `<button>` for now: a
/// gold-filled `Button` variant waits for a second consumer.
#[component]
fn SubmitArrow(label: Signal<String>) -> impl IntoView {
    view! {
        <button
            type="submit"
            aria-label=move || label.get()
            title=move || label.get()
            class="grid size-10 shrink-0 cursor-pointer place-items-center rounded-full bg-gold text-on-gold transition hover:bg-gold-hover group-has-[input:placeholder-shown]/card:opacity-40"
        >
            <svg
                width="20"
                height="20"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
            >
                <path d="M8 13V3M4 7l4-4 4 4" />
            </svg>
        </button>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn card(value: &str) -> String {
        let value = value.to_owned();
        Owner::new().with(|| {
            view! {
                <SearchCard
                    value=value
                    label=Signal::stored("Search the texts".to_owned())
                    placeholder=Signal::stored("Search the canon".to_owned())
                    submit_label=Signal::stored("Search".to_owned())
                >
                    <select name="mode"></select>
                </SearchCard>
            }
            .to_html()
        })
    }

    #[test]
    fn it_is_a_get_form_named_search() {
        let html = card("");
        assert!(html.contains(r#"role="search""#), "{html}");
        assert!(html.contains(r#"method="get""#), "{html}");
    }

    #[test]
    fn the_field_carries_its_name_value_and_placeholder() {
        let html = card("mindfulness");
        assert!(html.contains(r#"name="q""#), "{html}");
        assert!(html.contains(r#"value="mindfulness""#), "{html}");
        assert!(html.contains(r#"placeholder="Search the canon""#), "{html}");
        assert!(
            html.contains(r#"<label for="q" class="sr-only">Search the texts</label>"#),
            "{html}"
        );
    }

    #[test]
    fn the_round_submit_button_is_named_for_assistive_technology() {
        let html = card("");
        assert!(html.contains(r#"type="submit""#), "{html}");
        assert!(html.contains(r#"aria-label="Search""#), "{html}");
        assert!(html.contains("rounded-full bg-gold"), "{html}");
    }

    /// The controls sit inside the same form (so they submit with it),
    /// before the submit button on the bottom row.
    #[test]
    fn the_controls_are_inside_the_form_before_the_submit_button() {
        let html = card("");
        let select = html.find(r#"<select name="mode">"#).expect(&html);
        let submit = html.find(r#"type="submit""#).expect(&html);
        let form_end = html.find("</form>").expect(&html);
        assert!(select < submit && submit < form_end, "{html}");
    }

    /// The opening tag of the card's bottom row (the controls and the
    /// submit button).
    fn bottom_row(html: &str) -> &str {
        let field = html.find("<input").expect(html);
        let start = field + html[field..].find("<div").expect(html);
        let end = start + html[start..].find('>').expect(html);
        &html[start..=end]
    }

    /// With no children the controls `<div>` is still rendered (empty) and
    /// the bottom row keeps `justify-between`, so the submit stays at the
    /// right. SSR cannot prove placement itself: the gallery capture does.
    #[test]
    fn no_children_still_renders_the_controls_div_with_justify_between() {
        let html = card("");
        let row = bottom_row(&html);
        assert!(row.contains("justify-between"), "{row}");
        let after = &html[html.find(row).expect(&html) + row.len()..];
        let controls = &after[..after.find('>').expect(&html)];
        assert!(!controls.contains("justify-end"), "{controls}");
    }

    /// The submit dims while the field is empty — in CSS
    /// (`:has(input:placeholder-shown)`), so without JavaScript too. SSR
    /// cannot run CSS: only the classes are asserted (the gallery capture
    /// at phone width shows the rendering).
    #[test]
    fn the_submit_dims_while_the_field_is_empty() {
        let html = card("");
        assert!(html.contains("group/card"), "{html}");
        assert!(
            html.contains("group-has-[input:placeholder-shown]/card:opacity-40"),
            "{html}"
        );
    }

    fn card_named(name: &'static str) -> String {
        Owner::new().with(|| {
            view! {
                <SearchCard
                    value=String::new()
                    label=Signal::stored("Search the texts".to_owned())
                    placeholder=Signal::stored("Search the canon".to_owned())
                    submit_label=Signal::stored("Search".to_owned())
                    name=name
                />
            }
            .to_html()
        })
    }

    /// A second form on a page passes its own `name`: the field's `id` and
    /// `name` and the label's `for` all follow it, and `q` is gone.
    #[test]
    fn a_custom_name_is_the_field_id_name_and_the_label_target() {
        let html = card_named("site-q");
        assert!(html.contains(r#"id="site-q""#), "{html}");
        assert!(html.contains(r#"name="site-q""#), "{html}");
        assert!(html.contains(r#"<label for="site-q""#), "{html}");
        assert!(!html.contains(r#"name="q""#), "{html}");
        assert!(!html.contains(r#"id="q""#), "{html}");
    }

    /// Without children the controls div still renders (empty, Leptos's
    /// `<!>` marker), so `justify-between` keeps the submit at the right;
    /// where the submit lands is CSS, not asserted here.
    #[test]
    fn without_children_the_controls_div_renders_and_the_submit_is_present() {
        let html = card_named("q");
        let row = bottom_row(&html);
        assert!(row.contains("justify-between"), "{row}");
        assert!(
            html.contains(r#"<div class="flex min-w-0 flex-wrap items-center gap-2"><!></div>"#),
            "{html}"
        );
        assert!(html.contains(r#"type="submit""#), "{html}");
    }

    /// A value is data, never markup: a quote cannot break out of the
    /// attribute.
    #[test]
    fn a_value_with_a_quote_is_escaped() {
        let html = card(r#"a" onfocus="alert(1)"#);
        assert!(!html.contains(r#"value="a" onfocus"#), "{html}");
        assert!(!html.contains(r#"" onfocus=""#), "{html}");
        assert!(html.contains("&quot;"), "{html}");
    }

    /// The field's contract for the browser: a search input, capped at 200
    /// characters, without autocomplete.
    #[test]
    fn the_field_is_a_capped_search_input_without_autocomplete() {
        let html = card("");
        assert!(html.contains(r#"type="search""#), "{html}");
        assert!(html.contains(r#"maxlength="200""#), "{html}");
        assert!(html.contains(r#"autocomplete="off""#), "{html}");
    }
}
