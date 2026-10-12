//! The carousel's tests: its stylesheet rule and its rendered markup.

/// The shared stylesheet, where `.pc-carousel` lives.
const CSS: &str = include_str!("../../style/theme.css");

/// The `scroll-behavior: smooth` of `.pc-carousel`, in `@layer components`.
fn base_rule() -> usize {
    CSS.find(".pc-carousel { scroll-behavior: smooth; }")
        .unwrap_or_else(|| panic!(".pc-carousel has no smooth rule: {CSS}"))
}

#[test]
fn pc_carousel_scrolls_smoothly() {
    let base = base_rule();
    let layer = CSS
        .find("@layer components {")
        .expect("the components layer exists");
    assert!(base > layer, "the smooth rule is in the components layer");
}

#[test]
fn reduced_motion_turns_it_off_for_the_os_preference_and_the_toggle() {
    let base = base_rule();
    let media = CSS
        .find("@media (prefers-reduced-motion: reduce)")
        .expect("the reduced-motion media query exists");
    let after_media = &CSS[media..];
    let os = media
        + after_media
            .find(".pc-carousel { scroll-behavior: auto; }")
            .unwrap_or_else(|| panic!("no carve-out in the media query: {CSS}"));
    let toggle = CSS
        .find(r#"[data-motion="reduce"] .pc-carousel { scroll-behavior: auto; }"#)
        .unwrap_or_else(|| panic!("no carve-out for the toggle: {CSS}"));
    assert!(os > base, "the media carve-out comes after the base rule");
    assert!(
        toggle > base,
        "the toggle carve-out comes after the base rule"
    );
}

// --- the rendered markup ---

use leptos::prelude::*;

use super::{Carousel, CarouselProps, Controls, Direction};
use crate::language::{Language, language_signal};

/// A carousel of `n` images with the id "strip", rendered on the server.
fn carousel(n: usize) -> String {
    Owner::new().with(|| carousel_view(n).to_html())
}

fn carousel_view(n: usize) -> AnyView {
    let children: ChildrenFragment = Box::new(move || {
        Fragment::new(
            (0..n)
                .map(|i| view! { <img alt=format!("shot {i}") /> }.into_any())
                .collect(),
        )
    });
    Carousel(
        CarouselProps::builder()
            .id("strip")
            .label("Screenshots")
            .slide_layout("sm:w-1/2")
            .children(children)
            .build(),
    )
    .into_any()
}

/// The whole opening tag of the first element whose tag contains `needle`.
fn tag_with<'a>(html: &'a str, needle: &str) -> &'a str {
    let at = html
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} not found in {html}"));
    let start = html[..at].rfind('<').expect("an opening tag");
    let end = at + html[at..].find('>').expect("a closing >");
    &html[start..=end]
}

#[test]
fn the_scroller_is_the_labelled_focusable_region() {
    let html = carousel(3);
    let tag = tag_with(&html, r#"role="region""#);
    for attr in [
        r#"aria-roledescription="carousel""#,
        r#"aria-label="Screenshots""#,
        r#"tabindex="0""#,
        r#"id="strip""#,
    ] {
        assert!(tag.contains(attr), "{attr} missing: {tag}");
    }
}

#[test]
fn the_scroller_snaps_and_sets_no_scroll_padding() {
    let html = carousel(3);
    let tag = tag_with(&html, r#"role="region""#);
    for class in [
        "flex",
        "overflow-x-auto",
        "snap-x",
        "snap-mandatory",
        "pc-carousel",
    ] {
        assert!(tag.contains(class), "{class} missing: {tag}");
    }
    assert!(!tag.contains("scroll-p"), "no scroll padding: {tag}");
}

#[test]
fn the_scroller_shows_a_ring_when_focused_by_keyboard() {
    let html = carousel(3);
    let tag = tag_with(&html, r#"role="region""#);
    for class in [
        "focus-visible:outline-2",
        "focus-visible:outline-offset-2",
        "focus-visible:outline-gold",
    ] {
        assert!(tag.contains(class), "{class} missing: {tag}");
    }
}

#[test]
fn each_slide_is_a_numbered_group_with_the_callers_layout() {
    let html = carousel(3);
    assert_eq!(
        html.matches(r#"aria-roledescription="slide""#).count(),
        3,
        "{html}"
    );
    for n in 1..=3 {
        assert!(
            html.contains(&format!(r#"aria-label="{n} of 3""#)),
            "{n}: {html}"
        );
    }
    let tag = tag_with(&html, r#"aria-label="1 of 3""#);
    assert!(tag.contains(r#"role="group""#), "{tag}");
    assert!(tag.contains("snap-start"), "{tag}");
    assert!(tag.contains("sm:w-1/2"), "{tag}");
}

#[test]
fn slide_labels_follow_the_language() {
    Owner::new().with(|| {
        language_signal().set(Language::Fr);
        let fr = carousel_view(2).to_html();
        assert!(fr.contains(r#"aria-label="2 sur 2""#), "{fr}");
        language_signal().set(Language::Vi);
        let vi = carousel_view(2).to_html();
        assert!(vi.contains(r#"aria-label="2 trên 2""#), "{vi}");
    });
}

#[test]
fn a_list_as_the_only_child_is_one_slide_per_item() {
    let html = Owner::new().with(|| {
        view! {
            <Carousel id="strip" label="Screenshots">
                {(0..3).map(|i| view! { <img alt=format!("shot {i}") /> }).collect_view()}
            </Carousel>
        }
        .to_html()
    });
    assert_eq!(
        html.matches(r#"aria-roledescription="slide""#).count(),
        3,
        "{html}"
    );
    assert!(html.contains(r#"aria-label="3 of 3""#), "{html}");
}

#[test]
fn no_slide_renders_nothing() {
    let html = carousel(0);
    for absent in ["carousel", "<button", "<div"] {
        assert!(!html.contains(absent), "{absent} in {html}");
    }
}

#[test]
fn one_slide_is_one_of_one() {
    let html = carousel(1);
    assert!(html.contains(r#"aria-label="1 of 1""#), "{html}");
}

#[test]
fn the_controls_are_invisible_until_hydrated_and_name_the_scroller() {
    let html = carousel(3);
    assert!(html.contains("invisible"), "{html}");
    assert_eq!(
        html.matches(r#"aria-controls="strip""#).count(),
        2,
        "{html}"
    );
}

#[test]
fn two_carousels_keep_their_own_controls() {
    let html = Owner::new().with(|| {
        view! {
            <Carousel id="a" label="A" slide_layout="">
                <img alt="a1" />
            </Carousel>
            <Carousel id="b" label="B" slide_layout="">
                <img alt="b1" />
            </Carousel>
        }
        .to_html()
    });
    assert_eq!(html.matches(r#"aria-controls="a""#).count(), 2, "{html}");
    assert_eq!(html.matches(r#"aria-controls="b""#).count(), 2, "{html}");
}

/// The controls alone, with the three flags given.
fn controls(hydrated: bool, at_start: bool, at_end: bool) -> String {
    Owner::new().with(|| {
        view! {
            <Controls
                id="strip"
                hydrated=Signal::derive(move || hydrated)
                at_start=Signal::derive(move || at_start)
                at_end=Signal::derive(move || at_end)
                status=RwSignal::new(String::new())
                on_press=|_: Direction| ()
            />
        }
        .to_html()
    })
}

fn disabled(html: &str, label: &str) -> bool {
    // The attribute, not the `disabled:` variants in the class list.
    let tag = tag_with(html, &format!(r#"aria-label="{label}""#));
    tag.contains(" disabled ") || tag.contains(" disabled>")
}

#[test]
fn hydrated_at_both_ends_nothing_overflows_so_the_row_is_invisible() {
    assert!(controls(true, true, true).contains("invisible"));
}

#[test]
fn hydrated_at_the_start_only_previous_is_disabled() {
    let html = controls(true, true, false);
    assert!(!html.contains("invisible"), "{html}");
    assert!(disabled(&html, "Previous"), "{html}");
    assert!(!disabled(&html, "Next"), "{html}");
}

#[test]
fn hydrated_at_the_end_only_next_is_disabled() {
    let html = controls(true, false, true);
    assert!(!html.contains("invisible"), "{html}");
    assert!(!disabled(&html, "Previous"), "{html}");
    assert!(disabled(&html, "Next"), "{html}");
}

#[test]
fn hydrated_in_the_middle_both_are_enabled() {
    let html = controls(true, false, false);
    assert!(!html.contains("invisible"), "{html}");
    assert!(!disabled(&html, "Previous"), "{html}");
    assert!(!disabled(&html, "Next"), "{html}");
}

#[test]
fn not_hydrated_the_row_is_invisible_whatever_the_flags() {
    assert!(controls(false, false, false).contains("invisible"));
}

#[test]
fn the_buttons_are_named_in_the_language() {
    Owner::new().with(|| {
        language_signal().set(Language::Fr);
        let html = view! {
            <Controls
                id="strip"
                hydrated=Signal::derive(|| true)
                at_start=Signal::derive(|| false)
                at_end=Signal::derive(|| false)
                status=RwSignal::new(String::new())
                on_press=|_: Direction| ()
            />
        }
        .to_html();
        assert!(html.contains(r#"aria-label="Précédent""#), "{html}");
        assert!(html.contains(r#"aria-label="Suivant""#), "{html}");
    });
}

#[test]
fn the_status_is_a_polite_live_region_empty_until_a_press() {
    let html = controls(true, false, false);
    let tag = tag_with(&html, r#"role="status""#);
    assert!(tag.contains(r#"aria-live="polite""#), "{tag}");
    let at = html.find(tag).expect("the tag") + tag.len();
    let inside = &html[at..at + html[at..].find("</").expect("a closing tag")];
    assert_eq!(inside.trim(), "", "empty at rest: {html}");
}

// --- the default slide width, the status text ---

use super::scroll::{at_end, at_start, controls_hidden};
use super::status_text;
use crate::catalogue::Locale;

/// The default `slide_layout` of a carousel, rendered with none given.
#[test]
fn the_default_slide_width_is_half_the_scroller_less_half_a_gap() {
    let html = Owner::new().with(|| {
        let children: ChildrenFragment = Box::new(|| {
            Fragment::new(vec![
                view! { <img alt="a" /> }.into_any(),
                view! { <img alt="b" /> }.into_any(),
            ])
        });
        Carousel(
            CarouselProps::builder()
                .id("strip")
                .label("Screenshots")
                .children(children)
                .build(),
        )
        .into_any()
        .to_html()
    });
    let tag = tag_with(&html, r#"aria-label="1 of 2""#);
    assert!(tag.contains("sm:w-[calc((100%-0.75rem)/2)]"), "{tag}");
    assert!(!tag.contains("sm:w-1/2"), "the gap-blind width: {tag}");
}

/// A carousel of two images, with `phone_layout` given or left to its default.
fn carousel_with_phone(phone: Option<&str>) -> String {
    Owner::new().with(|| {
        let children: ChildrenFragment = Box::new(|| {
            Fragment::new(vec![
                view! { <img alt="a" /> }.into_any(),
                view! { <img alt="b" /> }.into_any(),
            ])
        });
        let builder = CarouselProps::builder()
            .id("strip")
            .label("Screenshots")
            .children(children);
        match phone {
            Some(layout) => Carousel(builder.phone_layout(layout).build()),
            None => Carousel(builder.build()),
        }
        .into_any()
        .to_html()
    })
}

#[test]
fn by_default_a_phone_slide_is_85_percent_so_the_next_peeks() {
    let html = carousel_with_phone(None);
    for n in 1..=2 {
        let tag = tag_with(&html, &format!(r#"aria-label="{n} of 2""#));
        assert_eq!(tag.matches("w-[85%]").count(), 1, "{tag}");
        assert!(tag.contains("sm:w-[calc((100%-0.75rem)/2)]"), "{tag}");
    }
}

#[test]
fn the_caller_sets_the_phone_width_and_the_default_is_gone() {
    let html = carousel_with_phone(Some("w-full"));
    for n in 1..=2 {
        let tag = tag_with(&html, &format!(r#"aria-label="{n} of 2""#));
        assert!(tag.contains("w-full"), "{tag}");
        assert!(!tag.contains("w-[85%]"), "two phone widths: {tag}");
        assert!(tag.contains("sm:w-[calc((100%-0.75rem)/2)]"), "{tag}");
    }
}

#[test]
fn two_slides_of_the_default_width_fit_the_scroller_so_the_controls_hide() {
    // The track has one gap of 0.75rem (12px at 16px) between two slides.
    for client in [320.0, 400.0, 768.0] {
        let slide = (client - 12.0) / 2.0;
        let content = slide + 12.0 + slide;
        assert!(
            controls_hidden(true, at_start(0.0), at_end(0.0, client, content)),
            "two slides in {client}px overflow ({content})"
        );
    }
    // The gap-blind half overflows by one gap: the controls would show.
    let content = 200.0 + 12.0 + 200.0;
    assert!(!controls_hidden(
        true,
        at_start(0.0),
        at_end(0.0, 400.0, content)
    ));
}

#[test]
fn the_status_names_the_slide_in_the_language() {
    assert_eq!(status_text(Locale::en, 3, 5, 0).trim(), "4 of 5");
    assert_eq!(status_text(Locale::fr, 3, 5, 0).trim(), "4 sur 5");
    assert_eq!(status_text(Locale::vi, 0, 5, 0).trim(), "1 trên 5");
}

#[test]
fn the_status_changes_on_every_press_even_for_the_same_slide() {
    let texts: Vec<String> = (0..4).map(|n| status_text(Locale::en, 3, 5, n)).collect();
    for pair in texts.windows(2) {
        assert_ne!(pair[0], pair[1], "two presses in a row read the same");
    }
    for text in &texts {
        assert_eq!(text.trim(), "4 of 5", "the visible words stay the same");
    }
}

#[test]
fn the_scroll_decisions_know_no_language() {
    let source = include_str!("scroll.rs");
    for word in ["leptos_i18n", "catalogue", "Locale", "td_string"] {
        let in_code = source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .any(|line| line.contains(word));
        assert!(!in_code, "scroll.rs mentions {word}");
    }
}
