//! `Slider`: a range slider with its value shown beside it, generic.
//!
//! A native `<input type="range">`, so the keyboard, touch and screen
//! readers work with no code of ours; the theme only colours it.

use leptos::prelude::*;

/// `value` held inside `[min, max]`. When `max` is below `min` the range is
/// empty and `min` wins.
fn clamp(value: u32, min: u32, max: u32) -> u32 {
    value.min(max).max(min)
}

/// A range slider over `[min, max]` that reads and writes `value`, with the
/// number shown at its right. The slider is a native `<input type="range">`:
/// arrows, Home, End and touch drag work as in the browser, and the value is
/// always held inside the range, also when `max` shrinks below it. Disabled
/// when `max <= min` (there is nothing to choose). On a phone the track fills
/// the width it is given and the thumb is at least 24px tall for a finger.
/// The filled part and the thumb take the theme's gold through `accent-color`.
#[component]
pub fn Slider(
    /// The chosen value, written on every move; kept inside `[min, max]`.
    value: RwSignal<u32>,
    /// The lowest value.
    min: u32,
    /// The highest value; a signal, so a caller's changing limit follows.
    #[prop(into)]
    max: Signal<u32>,
    /// The distance between two values; 1 by default.
    #[prop(default = 1)]
    step: u32,
    /// The slider's accessible name (`aria-label`); a string or a signal.
    #[prop(into)]
    label: Signal<String>,
    /// Whether the value is shown at the right of the slider; true by default.
    #[prop(default = true)]
    show_value: bool,
    /// Layout classes only (margin, width), never colours.
    #[prop(optional, into)]
    layout: String,
) -> impl IntoView {
    // Written back, so a limit that shrinks below the chosen value moves the
    // value itself, not only what is shown (which is always the clamped one).
    // Browser only: the server's render shows the clamped value anyway.
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let held = clamp(value.get_untracked(), min, max.get());
        if held != value.get_untracked() {
            value.set(held);
        }
    });
    let shown = move || clamp(value.get(), min, max.get());
    let on_input = move |ev| {
        if let Ok(typed) = event_target_value(&ev).parse::<u32>() {
            value.set(clamp(typed, min, max.get_untracked()));
        }
    };
    view! {
        <div class=format!("flex items-center gap-3 {layout}")>
            <input
                type="range"
                class=INPUT
                min=min
                max=move || max.get()
                step=step
                value=shown
                prop:value=shown
                aria-label=move || label.get()
                disabled=move || max.get() <= min
                on:input=on_input
            />
            {show_value
                .then(|| {
                    view! { <span class="tabular-nums text-sm text-heading">{shown}</span> }
                })}
        </div>
    }
}

/// The range input's classes. `accent-gold` paints the filled part and the
/// thumb in the theme's gold (a token, so it follows light and dark); `h-6`
/// makes the touch target 24px; the focus ring is the buttons'.
const INPUT: &str = "h-6 w-full min-w-0 flex-1 cursor-pointer accent-gold \
    focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold \
    disabled:cursor-not-allowed disabled:opacity-40";

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

    /// Whether the opening tag carries the boolean attribute `name` (a word of
    /// its own, not a `disabled:` class).
    fn has_attribute(tag: &str, name: &str) -> bool {
        tag.trim_end_matches('>')
            .split_whitespace()
            .any(|word| word == name)
    }

    fn slider(value: u32, max: u32) -> String {
        let value = RwSignal::new(value);
        render(view! { <Slider value=value min=1 max=max step=5 label="Cards" /> })
    }

    #[test]
    fn clamp_holds_a_value_inside_the_range() {
        assert_eq!(clamp(0, 1, 20), 1);
        assert_eq!(clamp(7, 1, 20), 7);
        assert_eq!(clamp(99, 1, 20), 20);
        assert_eq!(clamp(1, 1, 20), 1);
        assert_eq!(clamp(20, 1, 20), 20);
    }

    #[test]
    fn clamp_gives_min_when_the_range_is_empty() {
        assert_eq!(clamp(5, 10, 3), 10);
        assert_eq!(clamp(5, 5, 5), 5);
    }

    #[test]
    fn it_is_a_native_range_input_with_its_limits() {
        let html = slider(5, 20);
        let input = tag_around(&html, "<input");
        for attr in [
            r#"type="range""#,
            r#"min="1""#,
            r#"max="20""#,
            r#"step="5""#,
            r#"value="5""#,
        ] {
            assert!(input.contains(attr), "{attr}: {input}");
        }
    }

    #[test]
    fn the_label_is_its_accessible_name() {
        let html = slider(5, 20);
        assert!(
            tag_around(&html, "<input").contains(r#"aria-label="Cards""#),
            "{html}"
        );
    }

    #[test]
    fn the_value_is_shown_at_the_right() {
        let html = slider(5, 20);
        let span = tag_around(&html, "tabular-nums");
        assert!(
            span.contains("text-sm") && span.contains("text-heading"),
            "{span}"
        );
        let at = html.find("tabular-nums").expect(&html);
        assert!(html[at..].contains(">5</span>"), "{html}");
        assert!(html.find("<input").expect(&html) < at, "{html}");
    }

    #[test]
    fn the_value_can_be_hidden() {
        let value = RwSignal::new(5);
        let html =
            render(view! { <Slider value=value min=1 max=20 label="Cards" show_value=false /> });
        assert!(!html.contains("tabular-nums"), "{html}");
    }

    #[test]
    fn the_step_is_one_by_default() {
        let value = RwSignal::new(5);
        let html = render(view! { <Slider value=value min=1 max=20 label="Cards" /> });
        assert!(
            tag_around(&html, "<input").contains(r#"step="1""#),
            "{html}"
        );
    }

    #[test]
    fn it_is_enabled_while_there_is_a_range() {
        let html = slider(5, 20);
        assert!(
            !has_attribute(&tag_around(&html, "<input"), "disabled"),
            "{html}"
        );
    }

    #[test]
    fn it_is_disabled_when_max_is_not_above_min() {
        for max in [1, 0] {
            let html = slider(1, max);
            assert!(
                has_attribute(&tag_around(&html, "<input"), "disabled"),
                "{html}"
            );
        }
    }

    #[test]
    fn a_value_above_max_is_shown_clamped() {
        let html = slider(15, 10);
        let input = tag_around(&html, "<input");
        assert!(input.contains(r#"value="10""#), "{input}");
        let at = html.find("tabular-nums").expect(&html);
        assert!(html[at..].contains(">10</span>"), "{html}");
    }

    #[test]
    fn a_label_signal_is_read() {
        let value = RwSignal::new(5);
        let label = RwSignal::new("Batch".to_owned());
        let html = render(view! { <Slider value=value min=1 max=20 label=label /> });
        assert!(html.contains(r#"aria-label="Batch""#), "{html}");
    }

    #[test]
    fn the_thumb_is_touch_sized_and_the_theme_gold_colours_it() {
        let html = slider(5, 20);
        let input = tag_around(&html, "<input");
        for class in ["h-6", "w-full", "cursor-pointer", "accent-gold"] {
            assert!(input.contains(class), "{class}: {input}");
        }
        assert!(input.contains("focus-visible:outline-gold"), "{input}");
    }

    #[test]
    fn the_layout_classes_go_on_the_wrapper() {
        let value = RwSignal::new(5);
        let html =
            render(view! { <Slider value=value min=1 max=20 label="Cards" layout="max-w-xs" /> });
        assert!(html.starts_with("<div"), "{html}");
        assert!(tag_around(&html, "max-w-xs").starts_with("<div"), "{html}");
    }
}
