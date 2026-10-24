//! [`ConfirmDialog`] (080): "are you sure?" before an act that costs
//! something — cutting every session, retiring a secret. A
//! [`Dialog`] (its focus trap, Escape) with Cancel and
//! a confirm in the act's tone. `require_text` makes the person type a word
//! (the application's name) before the confirm enables; `pending` disables
//! it while the act runs, and a click holds it off until the act ends or the
//! dialog closes, so a double click acts once even without `pending`;
//! `error` shows why it failed. Closing clears what was typed.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Button, Disclosed, Size, Variant, classes};
use crate::catalogue::strings;
use crate::dialog::{Dialog, DialogSize};
use crate::field::TextField;
use crate::tone::Tone;

/// Whether the confirm is enabled: nothing running, and the required word,
/// if any, typed (trimmed, exact case).
#[must_use]
pub fn may_confirm(typed: &str, required: Option<&str>, pending: bool) -> bool {
    !pending && required.is_none_or(|word| typed.trim() == word)
}

/// A confirmation dialog bound to `open`.
#[component]
pub fn ConfirmDialog(
    /// The dialog's id, unique on the page.
    id: &'static str,
    /// Whether it is open.
    open: RwSignal<bool>,
    /// The question, its title.
    #[prop(into)]
    title: String,
    /// What the confirm button says ("Cut every session").
    #[prop(into)]
    confirm_label: String,
    /// The act's tone: Danger paints the confirm as a danger button.
    #[prop(optional)]
    tone: Option<Tone>,
    /// A word to type before the confirm enables.
    #[prop(optional, into)]
    require_text: Option<String>,
    /// The act is running: the confirm is disabled.
    #[prop(optional, into)]
    pending: Option<Signal<bool>>,
    /// Why the act failed, shown above the buttons. Setting it releases the
    /// confirm for a retry; set it to `None` before acting again, so a second
    /// failure with the same message is a change too.
    #[prop(optional, into)]
    error: Option<Signal<Option<String>>>,
    /// The act.
    on_confirm: Callback<()>,
    /// What the act does, in a sentence or two.
    children: ChildrenFn,
) -> impl IntoView {
    let t = strings();
    let typed = RwSignal::new(String::new());
    let pending = pending.unwrap_or_else(|| Signal::derive(|| false));
    let error = error.unwrap_or_else(|| Signal::derive(|| None));
    let required = StoredValue::new(require_text);
    // One click acts once (review 080): `acting` holds the confirm off from
    // the click until the act ends (`pending` back to false) or the dialog
    // closes — even for a caller that passes no `pending`.
    let acting = RwSignal::new(false);
    resets(open, typed, acting, pending, error);
    let busy = Signal::derive(move || pending.get() || acting.get());
    let enabled =
        move || required.with_value(|r| may_confirm(&typed.get(), r.as_deref(), busy.get()));
    let variant = if tone == Some(Tone::Danger) {
        Variant::Danger
    } else {
        Variant::Primary
    };
    let confirm_label = StoredValue::new(confirm_label);
    let enabled = Signal::derive(enabled);
    let on_confirm = Callback::new(move |()| {
        acting.set(true);
        on_confirm.run(());
    });
    let footer = move || footer_buttons(open, variant, enabled, busy, confirm_label, on_confirm);
    let field_id = format!("{id}-type");
    view! {
        <Dialog controls=Disclosed::new(id, open) title=title size=DialogSize::Popup footer=footer>
            <div class="flex flex-col gap-4">
                <div class="text-sm text-ink">{children()}</div>
                {required
                    .get_value()
                    .map(|word| {
                        let label = td_string!(t.locale(), confirm_type, text = word.clone())
                            .to_string();
                        view! {
                            <TextField id=field_id.clone() label=label value=typed mono=true />
                        }
                    })}
                {move || {
                    error
                        .get()
                        .map(|e| {
                            view! {
                                <p role="alert" class="m-0 text-sm text-danger-text">
                                    {e}
                                </p>
                            }
                        })
                }}
            </div>
        </Dialog>
    }
}

/// Closing clears what was typed and the click's hold; the act ending
/// (`pending` back to false) or failing (`error` set) releases the hold.
/// Browser-only: effects.
fn resets(
    open: RwSignal<bool>,
    typed: RwSignal<String>,
    acting: RwSignal<bool>,
    pending: Signal<bool>,
    error: Signal<Option<String>>,
) {
    #[cfg(feature = "browser")]
    {
        Effect::new(move |_| {
            if !open.get() {
                typed.set(String::new());
                acting.set(false);
            }
        });
        Effect::new(move |was: Option<bool>| {
            let now = pending.get();
            if was == Some(true) && !now {
                acting.set(false);
            }
            now
        });
        // A failure releases the hold, so the person can try again without
        // closing — whether or not the caller wired `pending` (review 080).
        Effect::new(move |_| {
            if error.with(Option::is_some) {
                acting.set(false);
            }
        });
    }
    #[cfg(not(feature = "browser"))]
    let _ = (open, typed, acting, pending, error);
}

/// Cancel, and the confirm: inert until `enabled` — `aria-disabled`, not
/// `disabled`, so a keyboard user keeps the focus on it inside the dialog's
/// trap — and announcing "Working…" while `pending`.
fn footer_buttons(
    open: RwSignal<bool>,
    variant: Variant,
    enabled: Signal<bool>,
    pending: Signal<bool>,
    label: StoredValue<String>,
    on_confirm: Callback<()>,
) -> AnyView {
    let t = strings();
    let busy = move || {
        pending
            .get()
            .then(|| td_string!(t.locale(), confirm_busy).to_string())
    };
    view! {
        <div class="flex flex-wrap items-center justify-end gap-2">
            <span class="sr-only" aria-live="polite">
                {busy}
            </span>
            <Button variant=Variant::Ghost on_click=move |_| open.set(false)>
                {move || td_string!(t.locale(), confirm_cancel)}
            </Button>
            <button
                type="button"
                class=classes(variant, Size::Md, "")
                aria-disabled=move || (!enabled.get()).then_some("true")
                on:click=move |_| {
                    if enabled.get_untracked() {
                        on_confirm.run(());
                    }
                }
            >
                {move || busy().unwrap_or_else(|| label.get_value())}
            </button>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_confirm_waits_for_the_word_and_for_nothing_running() {
        assert!(may_confirm("", None, false), "no word asked");
        assert!(!may_confirm("", None, true), "running");
        assert!(
            !may_confirm("Studi", Some("Studio"), false),
            "not yet typed"
        );
        assert!(may_confirm("  Studio ", Some("Studio"), false), "trimmed");
        assert!(!may_confirm("studio", Some("Studio"), false), "exact case");
        assert!(
            !may_confirm("Studio", Some("Studio"), true),
            "typed but running"
        );
    }
}
