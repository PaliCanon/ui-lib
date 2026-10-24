//! [`Switch`] (080): on or off, applied at once — "Enabled", "Shared with
//! other tenants". A `role="switch"` button with `aria-checked`, its label
//! and description beside it; the state variants paint it.

use leptos::prelude::*;

/// The track and the knob: one class set each, the "on" look from the
/// `aria-checked` state variant.
const TRACK: &str = "relative inline-flex h-7 w-12 shrink-0 cursor-pointer items-center rounded-full border border-line bg-chip transition-colors aria-checked:border-gold aria-checked:bg-gold aria-disabled:cursor-wait aria-disabled:opacity-60 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold disabled:cursor-not-allowed disabled:opacity-50";
const KNOB: &str = "pointer-events-none ml-0.5 size-5 rounded-full bg-muted transition-transform group-aria-checked:translate-x-5 group-aria-checked:bg-on-gold";

/// A labelled on/off control. `on_change` runs with the new state after it
/// is set.
#[component]
pub fn Switch(
    /// The control's id, unique on the page; the label's derives from it.
    #[prop(into)]
    id: String,
    /// What it turns on and off.
    #[prop(into)]
    label: String,
    /// One line under the label.
    #[prop(optional, into)]
    description: Option<String>,
    /// The state.
    checked: RwSignal<bool>,
    /// Not operable while true.
    #[prop(optional, into)]
    disabled: Option<Signal<bool>>,
    /// Saving: not operable meanwhile, so two fast toggles cannot race.
    #[prop(optional, into)]
    pending: Option<Signal<bool>>,
    /// Runs with the new state, after `checked` is set: a save that fails
    /// sets `checked` back (the switch shows what the caller says is true).
    #[prop(optional, into)]
    on_change: Option<Callback<bool>>,
) -> impl IntoView {
    // Saving is aria-disabled, not disabled: the focus stays on the switch
    // (as on ConfirmDialog's confirm); clicks are ignored meanwhile.
    let saving = move || pending.is_some_and(|p| p.get());
    let saving_now = move || pending.is_some_and(|p| p.get_untracked());
    let label_id = format!("{id}-label");
    let description_id = description.as_ref().map(|_| format!("{id}-description"));
    let description = description.map(|d| {
        view! {
            <span id=format!("{id}-description") class="text-sm text-muted">
                {d}
            </span>
        }
    });
    view! {
        <div class="flex items-center justify-between gap-4 py-1">
            <div class="flex min-w-0 flex-col gap-0.5">
                <span id=label_id.clone() class="text-sm font-medium text-heading">
                    {label}
                </span>
                {description}
            </div>
            <button
                id=id
                type="button"
                role="switch"
                class=format!("group {TRACK}")
                aria-checked=move || checked.get().to_string()
                aria-labelledby=label_id
                aria-describedby=description_id
                disabled=move || disabled.is_some_and(|d| d.get())
                aria-disabled=move || saving().then_some("true")
                aria-busy=move || saving().then_some("true")
                on:click=move |_| {
                    if saving_now() {
                        return;
                    }
                    let on = !checked.get_untracked();
                    checked.set(on);
                    if let Some(run) = on_change {
                        run.run(on);
                    }
                }
            >
                <span class=KNOB aria-hidden="true"></span>
            </button>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn html(on: bool, description: bool) -> String {
        Owner::new().with(move || {
            let checked = RwSignal::new(on);
            if description {
                view! { <Switch id="en" label="Enabled" description="Obtains tokens" checked=checked /> }.to_html()
            } else {
                view! { <Switch id="en" label="Enabled" checked=checked /> }.to_html()
            }
        })
    }

    #[test]
    fn a_switch_is_a_named_switch_that_says_its_state() {
        let on = html(true, true);
        assert!(
            on.contains(r#"role="switch""#) && on.contains(r#"aria-checked="true""#),
            "{on}"
        );
        assert!(
            on.contains(r#"aria-labelledby="en-label""#) && on.contains(r#"id="en-label""#),
            "{on}"
        );
        assert!(on.contains(r#"aria-describedby="en-description""#), "{on}");
        assert!(html(false, false).contains(r#"aria-checked="false""#));
    }

    #[test]
    fn a_saving_switch_is_inert_and_busy() {
        let out = Owner::new().with(|| {
            let checked = RwSignal::new(true);
            view! { <Switch id="s" label="Shared" checked=checked pending=Signal::derive(|| true) /> }.to_html()
        });
        assert!(
            out.contains("disabled") && out.contains(r#"aria-busy="true""#),
            "{out}"
        );
    }

    #[test]
    fn without_a_description_nothing_is_described() {
        assert!(!html(true, false).contains("aria-describedby="));
    }
}
