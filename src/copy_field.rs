//! [`CopyField`] (080): a value to take away — a client id, a secret shown
//! once — read-only, with a Copy button. Copying selects the text too, so
//! where the Clipboard API is refused (an insecure origin, a denied
//! permission) the person can still copy it by hand; the button says
//! "Copied" only once the clipboard's promise resolved, "Selected" otherwise.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Size, Variant, classes};
use crate::catalogue::strings;
use crate::icon::{Icon, IconName};
use crate::tone::{Tone, edge_classes};

/// What the button says after a click, for 2 s.
/// Copied and Selected are told by the browser's clipboard alone.
#[cfg_attr(not(feature = "browser"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Said {
    /// Nothing yet, or the 2 s are over.
    Idle,
    /// The Clipboard API took it.
    Copied,
    /// No Clipboard API: the text is selected.
    Selected,
}

/// Selects the field `id`'s text, then asks the clipboard for it and sets
/// `state` from the answer: Copied only once the promise resolved, Selected
/// when it was refused (an insecure origin, a denied permission) or the API
/// is absent — the selection is there either way, to copy by hand.
fn copy_from(
    id: &str,
    text: &str,
    state: RwSignal<Said>,
    timer: StoredValue<Option<TimeoutHandle>>,
) {
    #[cfg(feature = "browser")]
    {
        use wasm_bindgen::JsCast;
        let window = web_sys::window();
        if let Some(input) = window
            .as_ref()
            .and_then(web_sys::Window::document)
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            input.select();
        }
        // The Clipboard API exists only on a secure origin; elsewhere calling
        // it throws, so the selection is the answer at once (review 080).
        let Some(window) = window.filter(web_sys::Window::is_secure_context) else {
            settle(state, timer, Said::Selected);
            return;
        };
        let promise = window.navigator().clipboard().write_text(text);
        leptos::task::spawn_local(async move {
            let said = if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
                Said::Copied
            } else {
                Said::Selected
            };
            settle(state, timer, said);
        });
    }
    #[cfg(not(feature = "browser"))]
    let _ = (id, text, state, timer);
}

/// Says `said`, then back to Copy 2 s later — the timer armed once the
/// clipboard answered, so a slow permission prompt is not cut short.
#[cfg(feature = "browser")]
fn settle(state: RwSignal<Said>, timer: StoredValue<Option<TimeoutHandle>>, said: Said) {
    // The clipboard may answer after the field is gone (a route change during
    // a permission prompt): every write is a `try_`, a no-op then.
    if let Some(h) = timer.try_get_value().flatten() {
        h.clear();
    }
    if state.try_set(said).is_some() {
        return;
    }
    let back = move || {
        let _ = state.try_set(Said::Idle);
    };
    let _ =
        timer.try_set_value(set_timeout_with_handle(back, std::time::Duration::from_secs(2)).ok());
}

/// A read-only value with a Copy button. `tone` colours its edge (a secret
/// shown once: `Warning`).
#[component]
pub fn CopyField(
    /// The control's id, unique on the page.
    #[prop(into)]
    id: String,
    /// What the value is.
    #[prop(into)]
    label: String,
    /// The value.
    #[prop(into)]
    value: String,
    /// Colours the box's edge.
    #[prop(optional)]
    tone: Option<Tone>,
) -> impl IntoView {
    let state = RwSignal::new(Said::Idle);
    let timer: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    on_cleanup(move || {
        if let Some(h) = timer.try_get_value().flatten() {
            h.clear();
        }
    });
    let edge = tone.map_or("border-line", edge_classes);
    let box_class =
        format!("flex items-center gap-2 rounded-xl border bg-field px-3 py-1.5 {edge}");
    let (input_id, copy_id, text) = (id.clone(), id.clone(), value.clone());
    view! {
        <div class="flex flex-col gap-1.5">
            <label for=id class="text-xs uppercase tracking-wide text-muted">
                {label}
            </label>
            <div class=box_class>
                <input
                    id=input_id
                    type="text"
                    readonly=true
                    value=value
                    class="min-w-0 flex-1 border-0 bg-transparent font-mono text-[13px] text-ink focus-visible:outline-none"
                />
                {copy_button(copy_id, text, state, timer)}
            </div>
        </div>
    }
}

/// The Copy button: copies (or selects), says so for 2 s.
fn copy_button(
    id: String,
    text: String,
    state: RwSignal<Said>,
    timer: StoredValue<Option<TimeoutHandle>>,
) -> impl IntoView {
    let t = strings();
    let click = move |_| copy_from(&id, &text, state, timer);
    let says = move || match state.get() {
        Said::Idle => td_string!(t.locale(), copy).to_string(),
        Said::Copied => td_string!(t.locale(), copied).to_string(),
        Said::Selected => td_string!(t.locale(), copy_selected).to_string(),
    };
    view! {
        <button
            type="button"
            class=classes(Variant::Chip, Size::Sm, "shrink-0")
            on:click=click
            aria-live="polite"
        >
            <Icon name=IconName::Copy size=14 />
            {says}
        </button>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn a_copy_field_is_a_labelled_read_only_value_with_a_copy() {
        let out = Owner::new().with(|| {
            view! { <CopyField id="cid" label="Client ID" value="app_7KQ2" tone=Tone::Warning /> }.to_html()
        });
        assert!(
            out.contains(r#"for="cid""#) && out.contains(r#"id="cid""#),
            "{out}"
        );
        assert!(
            out.contains("readonly") && out.contains(r#"value="app_7KQ2""#),
            "{out}"
        );
        assert!(
            out.contains("border-warning") && out.contains("<button"),
            "{out}"
        );
    }
}
