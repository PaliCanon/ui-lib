//! A carousel: a row of slides, one or more in view, scrolled by the
//! reader's finger, the keyboard or two buttons.
//!
//! The scroller is native (`overflow-x: auto` with scroll-snap), so it
//! scrolls, swipes and answers the arrow keys with no JavaScript; the two
//! buttons appear once the page is hydrated. The state is two flags
//! (`at_start`, `at_end`) read from the live element, never an index: the
//! number of slides in view depends on the width.
//!
//! The decisions that can be tested without a browser are the pure
//! functions of `scroll`.

mod browser;
mod scroll;

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Button, Variant};
use crate::catalogue::strings;
use crate::icon::{Icon, IconName};
use scroll::{Direction, controls_hidden};

/// The scroller: a flex row that scrolls sideways and snaps each slide's
/// left edge to its own. No scroll padding: the slides' left edges are the
/// snap positions the buttons read. `pc-carousel` adds the smooth scroll
/// (and its reduced-motion carve-out) in `theme.css`. The scroller is
/// focusable, so it shows the buttons' focus ring (`focus-visible`: the
/// keyboard, not a tap).
const SCROLLER: &str = "pc-carousel relative flex gap-3 overflow-x-auto snap-x snap-mandatory \
    focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold";

/// A slide's wrapper, without its widths: `phone_layout` sets the width
/// below `sm` and `slide_layout` from `sm`.
const SLIDE: &str = "shrink-0 snap-start";

/// The default width of a slide below `sm`: 85% of the scroller, so the
/// next slide peeks on a phone.
const DEFAULT_PHONE_LAYOUT: &str = "w-[85%]";

/// The default width of a slide from `sm`: half of the scroller less half
/// of the track's gap (`gap-3`, 0.75rem), so two slides fill the row exactly.
const DEFAULT_LAYOUT: &str = "sm:w-[calc((100%-0.75rem)/2)]";

/// The controls row, below the scroller.
const CONTROLS: &str = "mt-3 flex items-center justify-center gap-2";

/// A row of slides with Previous and Next buttons.
///
/// The component wraps each child in a slide (`role="group"`,
/// `aria-roledescription="slide"`, named "n of N" in the current language)
/// and knows how many there are; with none it renders nothing. The scroller
/// itself is the labelled region: focusable, so the arrow keys, Home and
/// End scroll it natively, and swipeable on a phone. There is no autoplay
/// and no loop.
///
/// The slides are the children: written one after another (each
/// expression is one slide), or as the only child a list, one slide per
/// item (`items.iter().map(…).collect_view()`). A list next to other
/// children is a single slide. A lone child that is not an element or a
/// list needs `.into_any()` (a bare `impl IntoView` is not a fragment).
///
/// The two buttons sit below the scroller. They are invisible (space kept,
/// out of the tab order and of the accessibility tree) until the page is
/// hydrated, and while nothing overflows, so a carousel with nothing to
/// scroll shows no controls. A press scrolls to the previous or next
/// slide's left edge, read from the DOM at that moment; a polite status
/// then announces the slide reached, "n of N" (again on a repeated press to
/// the same slide). A press never goes past the end of the scroll range:
/// when the last slide is narrower than the scroller, the press that
/// reaches the end names the last slide. A second press while the first is
/// still moving may aim at the same slide. Slide widths must not depend on
/// content that loads late (an image without a size would move the edges
/// after they were read).
///
/// The track has a gap (`gap-3`, 0.75rem) between slides: the caller's
/// `slide_layout` widths must account for it. Three slides of `sm:w-1/3`
/// are 2 gaps (24px) wider than the scroller and overflow by less than a
/// slide, so the controls show for nothing; three slides of
/// `sm:w-[calc((100%-1.5rem)/3)]` fit exactly and the controls stay
/// hidden.
///
/// Smooth scrolling is the stylesheet's (`.pc-carousel` in `theme.css`),
/// off under `prefers-reduced-motion` and under `data-motion="reduce"`.
#[component]
pub fn Carousel(
    /// The scroller's element id, unique on the page; the buttons name it
    /// with `aria-controls`.
    #[prop(into)]
    id: String,
    /// The region's accessible name (`aria-label`), e.g. "Screenshots".
    #[prop(into)]
    label: String,
    /// Width classes of a slide below `sm` (a phone); layout only, never
    /// colours. 85% of the scroller by default, so the next slide peeks.
    /// The gap (0.75rem) is not part of any width: two whole slides are
    /// `w-[calc((100%-0.75rem)/2)]`, one is `w-full`.
    #[prop(default = DEFAULT_PHONE_LAYOUT.to_owned(), into)]
    phone_layout: String,
    /// Width classes of a slide from `sm` up; layout only, never colours. A
    /// half of the scroller by default, less half of the track's gap, so
    /// two slides fill the row. The gap (0.75rem) is not part of any
    /// width: subtract it when slides are meant to fill the row. Its
    /// classes carry a breakpoint prefix (`sm:`, `md:`…): the width below
    /// `sm` is `phone_layout`'s, and an unprefixed width here would compete
    /// with it.
    #[prop(default = DEFAULT_LAYOUT.to_owned(), into)]
    slide_layout: String,
    /// The slides, one child each.
    children: ChildrenFragment,
) -> impl IntoView {
    let slides: Vec<AnyView> = children().nodes.into();
    let total = slides.len();
    if total == 0 {
        return ().into_any();
    }
    let hydrated = RwSignal::new(false);
    let at_start = RwSignal::new(true);
    let at_end = RwSignal::new(false);
    let status = RwSignal::new(String::new());
    let scroller: NodeRef<leptos::html::Div> = NodeRef::new();
    browser::track(scroller, hydrated, at_start, at_end);
    let presses = StoredValue::new(0);
    let on_press = Callback::new(move |direction| {
        browser::press(scroller, direction, total, status, presses);
    });
    view! {
        <div class="w-full min-w-0">
            <div
                node_ref=scroller
                id=id.clone()
                role="region"
                aria-roledescription="carousel"
                aria-label=label
                tabindex="0"
                class=SCROLLER
            >
                <Slides slides total phone=phone_layout layout=slide_layout />
            </div>
            <Controls
                id
                hydrated=hydrated.into()
                at_start=at_start.into()
                at_end=at_end.into()
                status
                on_press
            />
        </div>
    }
    .into_any()
}

/// What the status region reads after a press: "n of N" in `locale`, for
/// the slide at `index` of `total`.
///
/// A screen reader announces a live region only when its text changes, and
/// pressing again towards the same slide (a second press while the first is
/// still moving) would write the same words. So every other press
/// (`press_count` odd) ends with a no-break space (U+00A0): consecutive
/// texts always differ, which is what makes the region announce again, and
/// the words read are the same. The language lives here, not in `scroll`,
/// whose functions are pure numbers.
#[cfg(any(test, feature = "hydrate"))]
fn status_text(
    locale: crate::catalogue::Locale,
    index: usize,
    total: usize,
    press_count: usize,
) -> String {
    let text = td_string!(locale, carousel_slide, n = index + 1, total = total).to_string();
    if press_count % 2 == 1 {
        format!("{text}\u{a0}")
    } else {
        text
    }
}

/// Wraps each slide, numbering it "n of `total`" in the current language;
/// `total` is the count of `slides`, passed by the one place that counts.
#[component]
fn Slides(slides: Vec<AnyView>, total: usize, phone: String, layout: String) -> impl IntoView {
    let t = strings();
    let class = format!("{SLIDE} {phone} {layout}");
    slides
        .into_iter()
        .enumerate()
        .map(|(index, slide)| {
            let label = move || {
                td_string!(t.locale(), carousel_slide, n = index + 1, total = total).to_string()
            };
            view! {
                <div
                    data-slide=""
                    role="group"
                    aria-roledescription="slide"
                    aria-label=label
                    class=class.clone()
                >
                    {slide}
                </div>
            }
        })
        .collect_view()
}

/// The Previous and Next buttons and the status. The row is invisible
/// before hydration and while both ends are reached (nothing overflows).
#[component]
fn Controls(
    #[prop(into)] id: String,
    hydrated: Signal<bool>,
    at_start: Signal<bool>,
    at_end: Signal<bool>,
    status: RwSignal<String>,
    #[prop(into)] on_press: Callback<Direction>,
) -> impl IntoView {
    let class = move || {
        if controls_hidden(hydrated.get(), at_start.get(), at_end.get()) {
            format!("{CONTROLS} invisible")
        } else {
            CONTROLS.to_owned()
        }
    };
    view! {
        <div class=class>
            <Step id=id.clone() direction=Direction::Previous disabled=at_start on_press />
            <Step id direction=Direction::Next disabled=at_end on_press />
        </div>
        <Status status />
    }
}

/// One of the two buttons, named in the current language (rebuilt when it
/// changes), disabled at its end.
#[component]
fn Step(
    id: String,
    direction: Direction,
    disabled: Signal<bool>,
    on_press: Callback<Direction>,
) -> impl IntoView {
    let t = strings();
    move || {
        let (label, icon) = match direction {
            Direction::Previous => (
                td_string!(t.locale(), carousel_previous),
                IconName::ChevronLeft,
            ),
            Direction::Next => (
                td_string!(t.locale(), carousel_next),
                IconName::ChevronRight,
            ),
        };
        view! {
            <Button
                variant=Variant::Icon
                label
                controls=id.clone()
                disabled
                on_click=move |_| on_press.run(direction)
            >
                <Icon name=icon />
            </Button>
        }
    }
}

/// Where a press announces the slide reached: visually hidden, polite, and
/// empty until the first press.
#[component]
fn Status(status: RwSignal<String>) -> impl IntoView {
    view! {
        <span role="status" aria-live="polite" class="sr-only">
            {move || status.get()}
        </span>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests;
