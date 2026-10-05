//! A search row: one `<form role="search" method="get">` on a single
//! line — a search icon, the field, a gold submit button flush at the
//! right ("Search" on a desktop, an arrow on a phone). The compact bar
//! of a results page's header, modelled on today's site's `SearchBar.tsx`
//! (the home's large card is [`super::SearchCard`]).
//!
//! Submitted by the browser itself; fields elsewhere on the page join the
//! same GET through the `form` attribute and the form's `id`.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// The row. The field's `name` (and `id`) is `name`; the form's `id` is
/// `form_id`, for controls outside it, and **has no default**: the caller
/// names it, so a page's form ids are chosen, never accidentally shared.
/// Ids are unique per page: a second form on the page passes a distinct
/// `name` and `form_id`. The row is the `group/row` its submit button dims
/// against while the field is empty, a name the caller's own markup must
/// not reuse.
#[component]
pub fn SearchRow(
    /// The field's starting text (e.g. from `?q=`).
    #[prop(into)]
    value: String,
    /// The field's accessible name (a visually hidden `<label>`).
    #[prop(into)]
    label: Signal<String>,
    /// What the empty field shows.
    #[prop(into)]
    placeholder: Signal<String>,
    /// The submit button's text (its accessible name on a phone, where
    /// only the arrow shows).
    #[prop(into)]
    submit_label: Signal<String>,
    /// The form's `id`, the target of other fields' `form` attribute.
    /// Required: there is no default, and a page's form ids are unique.
    form_id: &'static str,
    /// The field's form name and id.
    #[prop(default = "q")]
    name: &'static str,
    /// The field itself, for a caller that focuses it (a shortcut key).
    #[prop(optional)]
    input_ref: Option<NodeRef<leptos::html::Input>>,
) -> impl IntoView {
    let input_ref = input_ref.unwrap_or_default();
    view! {
        <form id=form_id role="search" method="get" class=ROW>
            <span class="flex items-center pl-3.5 text-muted sm:pl-4">
                <Icon name=IconName::Search />
            </span>
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
                class="min-w-0 flex-1 bg-transparent px-3 py-2 text-[15px] text-heading outline-none placeholder:text-subtle"
            />
            <button
                type="submit"
                aria-label=move || submit_label.get()
                class="flex cursor-pointer items-center bg-gold px-3 text-sm font-semibold text-on-gold transition hover:bg-gold-hover group-has-[input:placeholder-shown]/row:opacity-40 sm:px-4"
            >
                <span class="hidden sm:inline">{move || submit_label.get()}</span>
                <span class="sm:hidden" aria-hidden="true">
                    <svg
                        width="18"
                        height="18"
                        viewBox="0 0 16 16"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="M3 8h9M8.5 4.5 12 8l-3.5 3.5" />
                    </svg>
                </span>
            </button>
        </form>
    }
}

/// The row itself: a card-edge border on the field surface, a gold ring
/// while anything inside has focus; the button clipped to its corners.
const ROW: &str = "group/row flex items-stretch overflow-hidden rounded-xl border border-card-edge bg-field \
    transition focus-within:border-gold focus-within:ring-4 focus-within:ring-glow";

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn row_with(value: &str, name: &'static str, form_id: &'static str) -> String {
        let value = value.to_owned();
        Owner::new().with(|| {
            view! {
                <SearchRow
                    value=value
                    label=Signal::stored("Search the texts".to_owned())
                    placeholder=Signal::stored("Search the canon".to_owned())
                    submit_label=Signal::stored("Search".to_owned())
                    name=name
                    form_id=form_id
                />
            }
            .to_html()
        })
    }

    fn row(value: &str) -> String {
        row_with(value, "q", "search-form")
    }

    #[test]
    fn it_is_a_get_form_named_search_with_an_id_for_outside_fields() {
        let html = row("");
        assert!(html.contains(r#"role="search""#), "{html}");
        assert!(html.contains(r#"method="get""#), "{html}");
        assert!(html.contains(r#"id="search-form""#), "{html}");
    }

    #[test]
    fn the_field_carries_its_name_value_and_placeholder() {
        let html = row("metta");
        assert!(html.contains(r#"name="q""#), "{html}");
        assert!(html.contains(r#"value="metta""#), "{html}");
        assert!(html.contains(r#"placeholder="Search the canon""#), "{html}");
        assert!(
            html.contains(r#"<label for="q" class="sr-only">Search the texts</label>"#),
            "{html}"
        );
    }

    /// The gold button says "Search" on a desktop, shows an arrow on a
    /// phone, and is named either way.
    #[test]
    fn the_submit_says_search_on_a_desktop_and_an_arrow_on_a_phone() {
        let html = row("");
        assert!(html.contains(r#"aria-label="Search""#), "{html}");
        assert!(
            html.contains(r#"<span class="hidden sm:inline">Search</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="sm:hidden" aria-hidden="true">"#),
            "{html}"
        );
        assert!(html.contains("bg-gold"), "{html}");
    }

    /// Like the card, the submit dims while the field is empty, in CSS.
    /// SSR cannot run CSS: only the classes are asserted.
    #[test]
    fn the_submit_dims_while_the_field_is_empty() {
        let html = row("");
        assert!(html.contains("group/row"), "{html}");
        assert!(
            html.contains("group-has-[input:placeholder-shown]/row:opacity-40"),
            "{html}"
        );
    }

    /// The form's `id` is the caller's: a second form on a page names its
    /// own, and the default of the old service is nowhere.
    #[test]
    fn a_custom_form_id_is_the_form_id_and_the_default_is_absent() {
        let html = row_with("", "q", "results-form");
        assert!(html.contains(r#"id="results-form""#), "{html}");
        assert!(!html.contains(r#"id="search-form""#), "{html}");
    }

    /// A custom `name` is the field's `id` and `name` and the label's
    /// `for` target; `q` is gone.
    #[test]
    fn a_custom_name_is_the_field_id_name_and_the_label_target() {
        let html = row_with("", "site-q", "search-form");
        assert!(html.contains(r#"id="site-q""#), "{html}");
        assert!(html.contains(r#"name="site-q""#), "{html}");
        assert!(html.contains(r#"<label for="site-q""#), "{html}");
        assert!(!html.contains(r#"name="q""#), "{html}");
        assert!(!html.contains(r#"id="q""#), "{html}");
    }

    /// Both together, as a second row on a page does: each id is the
    /// caller's, and the form's id and the field's id differ.
    #[test]
    fn a_custom_name_and_form_id_together() {
        let html = row_with("", "second-q", "second-form");
        assert!(html.contains(r#"<form id="second-form""#), "{html}");
        assert!(html.contains(r#"id="second-q""#), "{html}");
        assert!(html.contains(r#"<label for="second-q""#), "{html}");
        assert!(!html.contains(r#"id="search-form""#), "{html}");
        assert!(!html.contains(r#"id="q""#), "{html}");
    }
}
