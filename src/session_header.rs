//! `SessionHeader`: a counter of progress and a close button, the caller composing any confirmation.
//!
//! The header holds exactly two things: the counter (`3 / 10 cards`, plain
//! text in tabular digits so it does not jitter as the numbers change) and
//! a close button ([`Variant::Icon`], a 44 px target at the right edge, on
//! one line on a phone). It holds no dialog.
//!
//! - The counter counts the caller's unit (cards, terms): `unit` is
//!   the caller's word, in the caller's language.
//! - The counter is **not** a live region: announcing after every answer
//!   would talk over the next card. The caller announces the end of the
//!   session itself.
//! - Closing is the caller's decision: `on_close` runs on a click, and the
//!   caller composes the confirmation [`crate::dialog::Dialog`] (stop or
//!   keep going) around it.

use leptos::prelude::*;

use crate::button::{Button, Variant};
use crate::icon::{Icon, IconName};

/// The click handler of the close button: runs `on_close`, whatever the
/// event. Generic over the event so that a native test, which cannot build
/// a `MouseEvent`, can run it with a unit one.
fn close_handler<E>(on_close: Callback<()>) -> impl Fn(E) + Send + Sync + 'static {
    move |_| on_close.run(())
}

/// A session's header: `done / total unit` as plain text, and a close
/// button calling `on_close`. No dialog inside — the caller composes the
/// confirmation and announces the end of the session itself.
#[component]
pub fn SessionHeader(
    /// How many are done.
    done: usize,
    /// How many there are in all.
    total: usize,
    /// What is counted, in the caller's language: "cards".
    #[prop(into)]
    unit: String,
    /// The close button's name for screen readers and its tooltip: "Stop the session".
    #[prop(into)]
    close_label: String,
    /// Runs when the close button is clicked.
    on_close: Callback<()>,
) -> impl IntoView {
    let on_click = close_handler::<leptos::ev::MouseEvent>(on_close);
    view! {
        <header class="flex items-center justify-between gap-3">
            <span class="text-sm tabular-nums text-muted">
                {format!("{done} / {total} {unit}")}
            </span>
            <Button variant=Variant::Icon label=close_label on_click=on_click>
                <Icon name=IconName::Close />
            </Button>
        </header>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn render() -> String {
        Owner::new().with(|| {
            view! {
                <SessionHeader
                    done=3
                    total=10
                    unit="cards"
                    close_label="Stop the session"
                    on_close=Callback::new(|()| {})
                />
            }
            .to_html()
        })
    }

    #[test]
    fn the_counter_reads_done_over_total_and_the_unit_in_tabular_digits() {
        let html = render();
        assert!(html.contains("3 / 10 cards"), "{html}");
        assert!(html.contains("tabular-nums"), "{html}");
    }

    #[test]
    fn nothing_is_announced_live() {
        assert!(!render().contains("aria-live"));
    }

    #[test]
    fn the_close_button_is_named_by_the_caller_and_shows_the_close_icon() {
        let html = render();
        assert!(html.contains(r#"aria-label="Stop the session""#), "{html}");
        assert!(html.contains("M4 4l16 16"), "{html}");
    }

    #[test]
    fn the_close_target_is_44_px() {
        assert!(render().contains("size-11"));
    }

    #[test]
    fn it_is_a_header_row_with_the_close_at_the_right_edge() {
        let html = render();
        assert!(html.contains("<header"), "{html}");
        assert!(html.contains("flex items-center justify-between"), "{html}");
    }

    #[test]
    fn nothing_else_is_inside_no_dialog_no_second_button() {
        let html = render();
        assert_eq!(html.matches("<button").count(), 1, "{html}");
        assert!(!html.contains("<dialog"), "{html}");
        assert!(!html.contains("role=\"dialog\""), "{html}");
    }

    #[test]
    fn the_counter_follows_the_callers_numbers_and_unit() {
        let html = Owner::new().with(|| {
            view! {
                <SessionHeader
                    done=0
                    total=0
                    unit="termes"
                    close_label="Arrêter"
                    on_close=Callback::new(|()| {})
                />
            }
            .to_html()
        });
        assert!(html.contains("0 / 0 termes"), "{html}");
        assert!(html.contains(r#"aria-label="Arrêter""#), "{html}");
    }

    #[test]
    fn the_numbers_are_shown_verbatim_even_when_done_exceeds_total_or_total_is_zero() {
        for (done, total, text) in [(5, 0, "5 / 0 cards"), (12, 10, "12 / 10 cards")] {
            let html = Owner::new().with(|| {
                view! {
                    <SessionHeader
                        done=done
                        total=total
                        unit="cards"
                        close_label="Stop"
                        on_close=Callback::new(|()| {})
                    />
                }
                .to_html()
            });
            assert!(html.contains(text), "{text} in {html}");
        }
    }

    // A native test cannot build a `MouseEvent` (Q1 ruling): the click
    // handler is generic over the event, so it is run here with a unit one.
    #[test]
    fn the_click_handler_runs_the_callers_on_close() {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = calls.clone();
        let on_close = Callback::new(move |()| {
            seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        let handler = close_handler::<()>(on_close);
        handler(());
        handler(());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
