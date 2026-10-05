//! [`Presence`]: a primitive that keeps its content mounted through its
//! exit animation, instead of removing it the instant it closes.
//!
//! The state machine (`mounted`, and `data-state` of "open" or "closed" on
//! one wrapper `<div data-presence data-state>`):
//! - **First render**: `show` true paints `data-state="open"` at once, no
//!   animation; `show` false renders nothing at all.
//! - **Open** (`show` false → true): mounted at `data-state="closed"`
//!   first, then `"open"` on the next frame, so the enter transition has a
//!   closed frame to animate from. If `show` goes false again before that
//!   frame runs, the frame never applies "open": nothing is ever shown (a
//!   refused open, e.g. a second open a non-dismissable dialog ignores).
//! - **Close** (`show` true → false): `data-state="closed"` at once (the
//!   exit transition plays from wherever the content currently is, open or
//!   still mid-enter), unmounted once the longest transition `theme.css`
//!   computes for it has run its course.
//! - **Re-trigger**: `show` flipping again before a pending frame or exit
//!   timer fires cancels it; the next step picks up from the current
//!   state, which a CSS transition already mid-flight continues from
//!   cleanly on its own.
//! - **`cut`**: true at the moment `show` goes false means this close is
//!   really a replace (something else is taking its place at once, e.g.
//!   012's shared slot) — unmounted immediately, no exit.
//!
//! The frame/timer glue only runs once hydrated: the server renders the
//! first state and nothing more.

use leptos::prelude::*;

use crate::motion::Motion;

/// A primitive that keeps its content mounted through its exit animation:
/// see the module docs for the full state machine. `motion` picks which of
/// the library's own classes plays it.
#[component]
pub fn Presence(
    /// Shown while true; kept mounted during the exit once it turns false.
    #[prop(into)]
    show: Signal<bool>,
    /// Which animation plays on enter and exit.
    motion: Motion,
    /// Layout classes added to the wrapper (position, sizing) — never
    /// colours.
    #[prop(optional, into)]
    layout: String,
    /// True for exactly the close that is really a replace (something else
    /// takes this one's place at once): no exit animation, unmounted right
    /// away. See the module docs.
    #[prop(optional, into)]
    cut: Option<Signal<bool>>,
    /// The content, rendered fresh each time `show` turns true.
    children: ChildrenFn,
) -> impl IntoView {
    let initial = show.get_untracked();
    let mounted = RwSignal::new(initial);
    let state = RwSignal::new(if initial { "open" } else { "closed" });
    let wrapper: NodeRef<leptos::html::Div> = NodeRef::new();
    drive(show, cut, mounted, state, wrapper);
    let class = format!("{} {layout}", motion.classes());
    move || {
        mounted.get().then(|| {
            let class = class.clone();
            view! {
                <div node_ref=wrapper data-presence="" data-state=move || state.get() class=class>
                    {children()}
                </div>
            }
        })
    }
}

/// The browser only: drives `mounted` and `state` from `show` and `cut`.
/// There is no window, no frame and no timer while the server renders —
/// the caller already set both from `show`'s initial value.
#[cfg(feature = "hydrate")]
fn drive(
    show: Signal<bool>,
    cut: Option<Signal<bool>>,
    mounted: RwSignal<bool>,
    state: RwSignal<&'static str>,
    wrapper: NodeRef<leptos::html::Div>,
) {
    use std::cell::Cell;
    use std::rc::Rc;

    let version: Rc<Cell<u64>> = Rc::new(Cell::new(0));
    Effect::new(move |_| {
        let my_version = version.get() + 1;
        version.set(my_version);
        if show.get() {
            open(mounted, state, &version, my_version);
        } else if cut.is_some_and(|c| c.get()) {
            mounted.set(false);
        } else {
            close(mounted, state, wrapper, &version, my_version);
        }
    });
}

/// The server: there is no window, no frame and no timer to drive anything
/// with. The caller already rendered `show`'s initial value directly.
#[cfg(not(feature = "hydrate"))]
fn drive(
    show: Signal<bool>,
    cut: Option<Signal<bool>>,
    mounted: RwSignal<bool>,
    state: RwSignal<&'static str>,
    wrapper: NodeRef<leptos::html::Div>,
) {
    let _ = (show, cut, mounted, state, wrapper);
}

/// `show` turned (or already was) true: already mounted just goes straight
/// to open (a CSS transition mid-exit continues cleanly from wherever it
/// is); not yet mounted paints closed first, then open a frame later —
/// unless `version` moved on by then (`show` went false again first: a
/// refused open, nothing is ever shown).
#[cfg(feature = "hydrate")]
fn open(
    mounted: RwSignal<bool>,
    state: RwSignal<&'static str>,
    version: &std::rc::Rc<std::cell::Cell<u64>>,
    my_version: u64,
) {
    if mounted.get_untracked() {
        state.set("open");
        return;
    }
    mounted.set(true);
    state.set("closed");
    let version = version.clone();
    request_animation_frame(move || {
        if version.get() == my_version {
            state.set("open");
        }
    });
}

/// `show` turned false and this is not a `cut`: play the exit from
/// wherever the content currently is, unmounting once the longest
/// transition `theme.css` gives its wrapper has run — unless `version`
/// moved on first (re-triggered, or cut, before the timer fires).
#[cfg(feature = "hydrate")]
fn close(
    mounted: RwSignal<bool>,
    state: RwSignal<&'static str>,
    wrapper: NodeRef<leptos::html::Div>,
    version: &std::rc::Rc<std::cell::Cell<u64>>,
    my_version: u64,
) {
    if !mounted.get_untracked() {
        return;
    }
    state.set("closed");
    let duration = exit_duration_ms(wrapper);
    let version = version.clone();
    set_timeout(
        move || {
            if version.get() == my_version {
                mounted.set(false);
            }
        },
        std::time::Duration::from_secs_f64(duration / 1000.0),
    );
}

/// The longest `transition-duration` the browser computes for `wrapper`
/// (several properties can each give their own, comma-separated), in
/// milliseconds; `0.0` if it cannot be read (no node, no window).
#[cfg(feature = "hydrate")]
fn exit_duration_ms(wrapper: NodeRef<leptos::html::Div>) -> f64 {
    let Some(el) = wrapper.get_untracked() else {
        return 0.0;
    };
    let Some(style) = window().get_computed_style(&el).ok().flatten() else {
        return 0.0;
    };
    let Ok(value) = style.get_property_value("transition-duration") else {
        return 0.0;
    };
    value
        .split(',')
        .filter_map(|part| parse_duration_ms(part.trim()))
        .fold(0.0_f64, f64::max)
}

/// Parses a single CSS `<time>` (`"200ms"` or `"0.2s"`) into milliseconds.
#[cfg(feature = "hydrate")]
fn parse_duration_ms(value: &str) -> Option<f64> {
    if let Some(ms) = value.strip_suffix("ms") {
        ms.parse().ok()
    } else {
        value
            .strip_suffix('s')?
            .parse::<f64>()
            .ok()
            .map(|s| s * 1000.0)
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::motion::Edge;

    #[test]
    fn open_on_first_render_shows_at_once_with_no_animation_step() {
        let html = Owner::new().with(|| {
            view! {
                <Presence show=Signal::derive(|| true) motion=Motion::Fade>
                    <p>"content"</p>
                </Presence>
            }
            .to_html()
        });
        assert!(html.contains("data-presence"), "{html}");
        assert!(html.contains(r#"data-state="open""#), "{html}");
        assert!(html.contains("<p>content</p>"), "{html}");
    }

    #[test]
    fn closed_on_first_render_renders_nothing() {
        let html = Owner::new().with(|| {
            view! {
                <Presence show=Signal::derive(|| false) motion=Motion::Fade>
                    <p>"content"</p>
                </Presence>
            }
            .to_html()
        });
        assert!(!html.contains("data-presence"), "{html}");
        assert!(!html.contains("content"), "{html}");
    }

    #[test]
    fn the_wrapper_wears_its_motion_and_layout_classes() {
        let html = Owner::new().with(|| {
            view! {
                <Presence
                    show=Signal::derive(|| true)
                    motion=Motion::Slide(Edge::Top)
                    layout="w-64"
                >
                    <p>"x"</p>
                </Presence>
            }
            .to_html()
        });
        assert!(html.contains("pc-motion pc-motion-slide-top"), "{html}");
        assert!(html.contains("w-64"), "{html}");
    }

    #[test]
    fn cut_does_not_affect_an_open_first_render() {
        let html = Owner::new().with(|| {
            view! {
                <Presence
                    show=Signal::derive(|| true)
                    motion=Motion::Scale
                    cut=Signal::derive(|| true)
                >
                    <p>"x"</p>
                </Presence>
            }
            .to_html()
        });
        assert!(html.contains(r#"data-state="open""#), "{html}");
    }
}
