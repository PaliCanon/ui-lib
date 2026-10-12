//! The carousel's scroll decisions, as pure functions of numbers read from
//! the live element (`scrollLeft`, `clientWidth`, `scrollWidth`, the slides'
//! left edges), so every corner is tested without a browser.
//!
//! Only the browser calls most of them, so they are compiled for the tests
//! and the hydrate build.

/// Browsers report fractional scroll positions; within this many pixels of
/// an end (or of a slide's edge) counts as being there.
#[cfg(any(test, feature = "hydrate"))]
const TOLERANCE: f64 = 1.0;

/// Which way a button scrolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    /// Towards the first slide.
    Previous,
    /// Towards the last slide.
    Next,
}

/// Whether the scroller is at its left end.
#[cfg(any(test, feature = "hydrate"))]
pub(super) fn at_start(scroll_left: f64) -> bool {
    scroll_left <= TOLERANCE
}

/// Whether the scroller is at its right end; true too when the content is
/// not wider than the box (nothing to scroll).
#[cfg(any(test, feature = "hydrate"))]
pub(super) fn at_end(scroll_left: f64, client_width: f64, scroll_width: f64) -> bool {
    scroll_left + client_width >= scroll_width - TOLERANCE
}

/// The last position the scroller can rest at: the content's width less the
/// box's, never below the start (content narrower than the box).
#[cfg(any(test, feature = "hydrate"))]
fn max_scroll(client_width: f64, scroll_width: f64) -> f64 {
    (scroll_width - client_width).max(0.0)
}

/// The slide a press scrolls to: `lefts` are the slides' left edges,
/// relative to the scroller's content, in order. Forward, the first edge
/// beyond `scroll_left`; backward, the last edge before it — both past the
/// tolerance, so the slide already in view (at 99.5 for an edge at 100) is
/// never the target.
///
/// The scroller cannot go past `scroll_width - client_width`: a forward
/// target at or beyond it (its edge reaches the end within the tolerance)
/// is stopped there, and the press names the last slide, whatever the
/// number of slides in view. Already at the end (the position a press
/// would reach is where the scroller is), there is nowhere to go: `None`.
/// [`scroll_target`] gives the position to scroll to.
#[cfg(any(test, feature = "hydrate"))]
pub(super) fn target_index(
    lefts: &[f64],
    scroll_left: f64,
    client_width: f64,
    scroll_width: f64,
    dir: Direction,
) -> Option<usize> {
    match dir {
        Direction::Next => {
            let index = lefts
                .iter()
                .position(|&left| left > scroll_left + TOLERANCE)?;
            let max = max_scroll(client_width, scroll_width);
            if max <= scroll_left + TOLERANCE {
                return None;
            }
            Some(if lefts[index] >= max - TOLERANCE {
                lefts.len() - 1
            } else {
                index
            })
        }
        Direction::Previous => lefts
            .iter()
            .rposition(|&left| left < scroll_left - TOLERANCE),
    }
}

/// Where the scroller goes for a slide whose left edge is `left`: the edge,
/// stopped at the last position the scroller can rest at.
#[cfg(any(test, feature = "hydrate"))]
pub(super) fn scroll_target(left: f64, client_width: f64, scroll_width: f64) -> f64 {
    left.min(max_scroll(client_width, scroll_width))
}

/// The index of a slide in the DOM, kept within the `total` slides the
/// component rendered: the status never names a slide past "N of N" even if
/// the scroller holds one more `[data-slide]` than the render counted (a
/// nested carousel, a caller's own marker). With no slide, 0.
#[cfg(any(test, feature = "hydrate"))]
pub(super) fn clamp_index(index: usize, total: usize) -> usize {
    index.min(total.saturating_sub(1))
}

/// Whether the controls row is hidden: before hydration (the buttons would
/// do nothing), and when nothing overflows (both ends reached at once).
pub(super) fn controls_hidden(hydrated: bool, at_start: bool, at_end: bool) -> bool {
    !hydrated || (at_start && at_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_start_within_a_pixel_of_the_left_edge() {
        for left in [0.0, 0.5, 1.0] {
            assert!(at_start(left), "at_start({left}) should be true");
        }
        assert!(!at_start(1.5), "at_start(1.5) should be false");
    }

    #[test]
    fn at_end_within_a_pixel_of_the_right_edge() {
        // scroll_width 1000, client_width 400: the last position is 600.
        for left in [600.0, 599.5, 599.0] {
            assert!(at_end(left, 400.0, 1000.0), "at_end({left}) should be true");
        }
        for left in [598.0, 0.0, 300.0] {
            assert!(
                !at_end(left, 400.0, 1000.0),
                "at_end({left}) should be false"
            );
        }
    }

    #[test]
    fn at_end_when_the_content_is_not_wider_than_the_box() {
        assert!(at_end(0.0, 400.0, 400.0), "at_end: equal widths");
        assert!(at_end(0.0, 400.0, 300.0), "at_end: narrower content");
    }

    const LEFTS: [f64; 4] = [0.0, 100.0, 200.0, 300.0];
    /// Four slides of 100px: the content is 400 wide, the box 250, so the
    /// last scroll position is 150.
    const BOX: f64 = 250.0;
    const CONTENT: f64 = 400.0;

    fn next(scroll_left: f64) -> Option<usize> {
        target_index(&LEFTS, scroll_left, BOX, CONTENT, Direction::Next)
    }

    fn previous(scroll_left: f64) -> Option<usize> {
        target_index(&LEFTS, scroll_left, BOX, CONTENT, Direction::Previous)
    }

    #[test]
    fn next_is_the_first_edge_beyond_the_position() {
        assert_eq!(next(0.0), Some(1), "next from 0");
        assert_eq!(next(50.0), Some(1), "next from mid-slide 50");
        assert_eq!(next(100.0), Some(3), "next from 100 reaches the end");
    }

    #[test]
    fn next_that_would_pass_the_end_is_the_last_slide() {
        // From 100 the next edge is 200, but the scroller stops at 150: the
        // slide that ends in view there is the last one.
        assert_eq!(next(100.0), Some(3), "clamped from 100");
        assert_eq!(next(120.0), Some(3), "clamped from 120");
    }

    #[test]
    fn next_at_the_end_has_nowhere_to_go() {
        // The last slide is narrower than the box: the scroller rests at 150,
        // before the last slide's edge (300), which is not reachable.
        assert_eq!(next(150.0), None, "next at the end");
        assert_eq!(next(149.5), None, "next within a pixel of the end");
        assert_eq!(next(300.0), None, "next past the end");
    }

    #[test]
    fn next_with_every_edge_in_range_is_unclamped() {
        // Slides of 100 in a 100 box: every edge is a scroll position.
        let at = |left| target_index(&LEFTS, left, 100.0, 400.0, Direction::Next);
        assert_eq!(at(0.0), Some(1), "from 0");
        assert_eq!(at(200.0), Some(3), "from 200");
        assert_eq!(at(300.0), None, "at the end");
    }

    #[test]
    fn next_does_not_pick_the_slide_it_already_shows() {
        // Every edge in range (a 100 box): 99.5 is the slide at 100, a
        // fraction of a pixel short.
        let at = |left| target_index(&LEFTS, left, 100.0, CONTENT, Direction::Next);
        assert_eq!(at(99.5), Some(2), "next from 99.5");
        // The tolerance is exclusive: an edge exactly 1px beyond is the current slide.
        assert_eq!(at(99.0), Some(2), "next from 99.0");
    }

    #[test]
    fn previous_is_the_last_edge_before_the_position() {
        assert_eq!(previous(150.0), Some(1), "previous from the end, 150");
        assert_eq!(previous(120.0), Some(1), "previous from 120");
        assert_eq!(previous(100.0), Some(0), "previous from 100");
        assert_eq!(previous(0.0), None, "previous from the first slide");
    }

    #[test]
    fn previous_does_not_pick_the_slide_it_already_shows() {
        // 100.5 is the slide at 100, a fraction of a pixel past.
        assert_eq!(previous(100.5), Some(0), "previous from 100.5");
        // The tolerance is exclusive: an edge exactly 1px before is the current slide.
        assert_eq!(previous(101.0), Some(0), "previous from 101.0");
    }

    #[test]
    fn one_slide_or_none_has_nowhere_to_go() {
        for dir in [Direction::Next, Direction::Previous] {
            assert_eq!(
                target_index(&[0.0], 0.0, 100.0, 100.0, dir),
                None,
                "one slide, {dir:?}"
            );
            assert_eq!(
                target_index(&[], 0.0, 100.0, 0.0, dir),
                None,
                "no slide, {dir:?}"
            );
        }
    }

    #[test]
    fn the_scroll_target_is_the_slide_edge_stopped_at_the_end() {
        assert_eq!(scroll_target(100.0, BOX, CONTENT), 100.0, "within range");
        assert_eq!(scroll_target(300.0, BOX, CONTENT), 150.0, "clamped");
        assert_eq!(scroll_target(0.0, 400.0, 300.0), 0.0, "content narrower");
        assert_eq!(scroll_target(100.0, 400.0, 300.0), 0.0, "never negative");
    }

    #[test]
    fn the_last_slides_scroll_target_is_a_position_at_the_end() {
        // Whatever the last slide's width, scrolling to it rests at the end.
        for (client, content) in [
            (250.0, 400.0),
            (250.0, 450.0),
            (100.0, 400.0),
            (400.0, 300.0),
        ] {
            let target = scroll_target(LEFTS[3], client, content);
            assert!(
                at_end(target, client, content),
                "box {client}, content {content}: target {target} is not the end"
            );
        }
    }

    /// Five slides, the last one 50 wide: the box 250, the content 450, so
    /// the last scroll position is 200 and the slide at 200 ends in view.
    const WIDE: [f64; 5] = [0.0, 100.0, 200.0, 300.0, 400.0];

    #[test]
    fn near_the_end_the_press_that_reaches_it_names_the_last_slide() {
        // The edge at 200 is the end of the scroll (a box of 250 in 450):
        // the press that reaches it names the last slide (review round 4;
        // it named slide 2 before).
        let next = |left| target_index(&WIDE, left, 250.0, 450.0, Direction::Next);
        assert_eq!(next(0.0), Some(1), "from 0");
        assert_eq!(next(100.0), Some(4), "from 100, the edge 200 is the end");
        assert_eq!(next(150.0), Some(4), "from 150");
        assert_eq!(next(190.0), Some(4), "from 190, 10px short of the end");
        // Within a pixel of the end (200): nowhere to go.
        assert_eq!(next(199.0), None, "from the end minus 1");
        assert_eq!(next(200.0), None, "at the end");
        assert!(at_end(scroll_target(WIDE[4], 250.0, 450.0), 250.0, 450.0));
    }

    /// Five slides, two in view on a 390px phone (measured): the last
    /// reachable edge, 531, equals the end of the scroll.
    const PHONE: [f64; 5] = [0.0, 177.0, 354.0, 531.0, 708.0];

    #[test]
    fn the_press_that_reaches_the_end_names_the_last_slide_several_in_view() {
        let next = |left| target_index(&PHONE, left, 342.0, 873.0, Direction::Next);
        assert_eq!(next(0.0), Some(1), "from 0");
        assert_eq!(next(177.0), Some(2), "from 177");
        assert_eq!(next(354.0), Some(4), "from 354, the edge 531 is the end");
        assert_eq!(next(531.0), None, "at the end");
        assert_eq!(
            target_index(&PHONE, 531.0, 342.0, 873.0, Direction::Previous),
            Some(2),
            "previous from the end"
        );
        assert_eq!(scroll_target(PHONE[4], 342.0, 873.0), 531.0, "last edge");
    }

    #[test]
    fn the_index_is_clamped_to_the_slides_rendered() {
        assert_eq!(clamp_index(0, 3), 0);
        assert_eq!(clamp_index(2, 3), 2);
        assert_eq!(clamp_index(3, 3), 2, "one more in the DOM than rendered");
        assert_eq!(clamp_index(9, 3), 2);
        assert_eq!(clamp_index(5, 0), 0, "no slide never underflows");
    }

    #[test]
    fn clamp_index_before_accessing_array_prevents_out_of_bounds() {
        // Verify that clamping must happen BEFORE using the index to access an array.
        // If more slides exist in the DOM than the component rendered (a nested
        // carousel), the index from target_index could exceed the rendered count.
        // Clamping before access ensures we always get a valid array element.
        let lefts = [0.0, 100.0, 200.0];
        let index = 5; // Out of bounds
        let total = 3; // Only 3 slides rendered
        let clamped = clamp_index(index, total);
        // Verify the clamped index can safely access the array.
        assert_eq!(lefts[clamped], 200.0);
        assert_eq!(clamped, 2, "clamped to the last rendered slide");
    }

    #[test]
    fn an_overflow_smaller_than_one_slide_shows_the_controls() {
        // 24px of overflow (the track's gaps): the box is 400, the content
        // 424, one slide 130. At the start, the end is not reached.
        assert!(at_start(0.0));
        assert!(!at_end(0.0, 400.0, 424.0), "24px is more than a tolerance");
        assert!(!controls_hidden(
            true,
            at_start(0.0),
            at_end(0.0, 400.0, 424.0)
        ));
        // At the other end, the start is not reached.
        assert!(!at_start(24.0));
        assert!(at_end(24.0, 400.0, 424.0));
        assert!(!controls_hidden(
            true,
            at_start(24.0),
            at_end(24.0, 400.0, 424.0)
        ));
        // Within the tolerance of a fit, the row hides.
        assert!(controls_hidden(
            true,
            at_start(0.0),
            at_end(0.0, 400.0, 400.5)
        ));
    }

    #[test]
    fn controls_are_hidden_before_hydration_and_when_nothing_overflows() {
        for (at_start, at_end) in [(false, false), (true, false), (false, true), (true, true)] {
            assert!(
                controls_hidden(false, at_start, at_end),
                "not hydrated, start {at_start}, end {at_end}"
            );
        }
        assert!(controls_hidden(true, true, true), "hydrated, both ends");
        assert!(!controls_hidden(true, false, false), "hydrated, mid");
        assert!(!controls_hidden(true, true, false), "hydrated, at start");
        assert!(!controls_hidden(true, false, true), "hydrated, at end");
    }
}
