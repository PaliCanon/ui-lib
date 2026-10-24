//! Toasts (080): a short message after an act — "Saved", "Secret rotated" —
//! that leaves by itself. [`provide_toaster`] once at the app root, the
//! [`Toasts`] regions once in the page, [`use_toaster`]`().show(…)` from
//! anywhere under it.
//!
//! Two live regions, since a region's politeness is fixed when it is
//! created: `polite` for every tone but Danger, `assertive` for Danger. A
//! toast leaves after 5 s unless the pointer is over it or the focus inside
//! it (WCAG 2.2.1: the person controls the timing), and always has a close.
//! Its timer is cancelled when it goes, so a route change fires nothing into
//! a dead page.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Size, Variant, classes};
use crate::catalogue::strings;
use crate::icon::{Icon, IconName};
use crate::tone::{Tone, callout_classes};

/// How long a toast stays when nothing holds it.
pub const TOAST_LIFETIME: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Toast {
    id: u64,
    tone: Tone,
    text: String,
}

/// The page's toasts: shows and dismisses them. `Copy`; made by
/// [`provide_toaster`], read by [`use_toaster`].
#[derive(Clone, Copy, Debug)]
pub struct Toaster {
    list: RwSignal<Vec<Toast>>,
    next: StoredValue<u64>,
}

impl Toaster {
    fn new() -> Self {
        Self {
            list: RwSignal::new(Vec::new()),
            next: StoredValue::new(1),
        }
    }

    /// Shows `text` in `tone`.
    pub fn show(&self, tone: Tone, text: impl Into<String>) {
        let id = self.next.get_value();
        self.next.set_value(id + 1);
        let text = text.into();
        self.list.update(|l| l.push(Toast { id, tone, text }));
    }

    /// Removes the toast `id`.
    fn dismiss(&self, id: u64) {
        self.list.update(|l| l.retain(|t| t.id != id));
    }

    /// The toasts of one region: Danger alone, or every other tone.
    fn of(&self, assertive: bool) -> Vec<Toast> {
        self.list.with(|l| {
            l.iter()
                .filter(|t| (t.tone == Tone::Danger) == assertive)
                .cloned()
                .collect()
        })
    }
}

/// Makes the page's [`Toaster`] and provides it; call once at the app root.
pub fn provide_toaster() -> Toaster {
    let toaster = Toaster::new();
    provide_context(toaster);
    toaster
}

/// The page's [`Toaster`].
///
/// # Panics
/// When no [`provide_toaster`] is above: a wiring error the first render of
/// any screen shows, never a message silently lost.
#[must_use]
pub fn use_toaster() -> Toaster {
    use_context::<Toaster>()
        .expect("use_toaster: no Toaster provided — call provide_toaster() at the app root")
}

/// The two live regions, bottom right (bottom, full width on a phone).
#[component]
pub fn Toasts() -> impl IntoView {
    let toaster = use_toaster();
    let region = move |assertive: bool| {
        view! {
            <div
                role=if assertive { "alert" } else { "status" }
                aria-live=if assertive { "assertive" } else { "polite" }
                class="flex flex-col gap-2"
            >
                <For each=move || toaster.of(assertive) key=|t| t.id let:toast>
                    <ToastItem toast=toast toaster=toaster />
                </For>
            </div>
        }
    };
    view! {
        <div class="pointer-events-none fixed inset-x-4 bottom-4 z-50 flex flex-col gap-2 sm:left-auto sm:right-6 sm:bottom-6 sm:w-96">
            {region(false)} {region(true)}
        </div>
    }
}

/// One toast: its timer, paused while held, cancelled when it goes.
#[component]
fn ToastItem(toast: Toast, toaster: Toaster) -> impl IntoView {
    let id = toast.id;
    let timer: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let stop = move || {
        if let Some(h) = timer.try_get_value().flatten() {
            h.clear();
        }
    };
    // Timers run in the browser alone: the server renders the toast and
    // leaves (`set_timeout` reaches the browser's clock through wasm-bindgen).
    let start = move || {
        stop();
        #[cfg(feature = "browser")]
        timer.set_value(set_timeout_with_handle(move || toaster.dismiss(id), TOAST_LIFETIME).ok());
    };
    start();
    on_cleanup(stop);
    let t = strings();
    let class = format!(
        "pointer-events-auto flex items-start gap-3 rounded-xl border px-4 py-3 text-sm shadow-lg {}",
        callout_classes(toast.tone)
    );
    view! {
        <div
            class=class
            on:mouseenter=move |_| stop()
            on:mouseleave=move |_| start()
            on:focusin=move |_| stop()
            on:focusout=move |_| start()
        >
            <span class="min-w-0 flex-1">{toast.text}</span>
            <button
                type="button"
                class=classes(Variant::Icon, Size::Sm, "-my-1 -mr-2 shrink-0")
                aria-label=move || td_string!(t.locale(), callout_dismiss).to_string()
                on:click=move |_| toaster.dismiss(id)
            >
                <Icon name=IconName::Close size=14 />
            </button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_danger_toast_goes_to_the_assertive_region_alone() {
        Owner::new().with(|| {
            let t = Toaster::new();
            t.show(Tone::Success, "Saved");
            t.show(Tone::Danger, "Failed");
            assert_eq!(
                t.of(false)
                    .iter()
                    .map(|x| x.text.as_str())
                    .collect::<Vec<_>>(),
                ["Saved"]
            );
            assert_eq!(
                t.of(true)
                    .iter()
                    .map(|x| x.text.as_str())
                    .collect::<Vec<_>>(),
                ["Failed"]
            );
        });
    }

    #[test]
    fn dismissing_removes_that_toast_only() {
        Owner::new().with(|| {
            let t = Toaster::new();
            t.show(Tone::Info, "a");
            t.show(Tone::Info, "b");
            let first = t.of(false)[0].id;
            t.dismiss(first);
            assert_eq!(
                t.of(false)
                    .iter()
                    .map(|x| x.text.as_str())
                    .collect::<Vec<_>>(),
                ["b"]
            );
        });
    }

    #[test]
    #[should_panic(expected = "provide_toaster")]
    fn use_toaster_without_a_provider_names_the_fix() {
        Owner::new().with(|| {
            let _ = use_toaster();
        });
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn the_regions_are_a_polite_status_and_an_assertive_alert() {
        let out = Owner::new().with(|| {
            let t = provide_toaster();
            t.show(Tone::Success, "Saved");
            view! { <Toasts /> }.to_html()
        });
        assert!(
            out.contains(r#"aria-live="polite""#) && out.contains(r#"aria-live="assertive""#),
            "{out}"
        );
        assert!(out.contains("Saved") && out.contains("<button"), "{out}");
    }
}
