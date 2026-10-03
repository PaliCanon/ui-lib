//! The button family: one look (a [`Variant`] and a [`Size`]), one small
//! component per behaviour (`docs/conventions/ui-components.md`).
//!
//! | Component | Behaviour | Semantics |
//! | --- | --- | --- |
//! | [`Button`] | runs the caller's action | `<button>` |
//! | [`DisclosureButton`] | opens and closes a [`Disclosure`] | `aria-expanded`, `aria-controls` |
//! | [`ToggleButton`] | on or off | `aria-pressed` |
//! | [`LinkButton`] | goes to another page | `<a href>`, `aria-current` |
//!
//! [`Variant::Nav`] is the look of a navigation list's entries ([`crate::nav`]).

mod disclosure;
mod style;

use leptos::prelude::*;

pub use disclosure::{Disclosed, Disclosure};
pub use style::{Size, Variant, classes};

fn is_disabled(disabled: Option<Signal<bool>>) -> bool {
    disabled.is_some_and(|d| d.get())
}

/// A button that runs `on_click`. `label` names it for screen readers when
/// its content is only an icon; `layout` adds layout classes.
#[component]
pub fn Button(
    /// The look; a chip by default.
    #[prop(optional)]
    variant: Variant,
    /// The size; medium by default.
    #[prop(optional)]
    size: Size,
    /// Its name for screen readers and its tooltip; needed when the content is only an icon.
    #[prop(optional, into)]
    label: Option<String>,
    /// Layout classes added to the variant's (margin, width, position) — never colours.
    #[prop(optional, into)]
    layout: String,
    /// Disabled while this is true.
    #[prop(optional, into)]
    disabled: Option<Signal<bool>>,
    /// What a click does.
    #[prop(into)]
    on_click: Callback<leptos::ev::MouseEvent>,
    /// The content: text, an icon, or both.
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=classes(variant, size, &layout)
            aria-label=label.clone()
            title=label
            disabled=move || is_disabled(disabled)
            on:click=move |ev| on_click.run(ev)
        >
            {children()}
        </button>
    }
}

/// A button that opens and closes `controls`, and says so: `aria-expanded`,
/// `aria-controls`, and `label_closed` / `label_open` as its name.
#[component]
pub fn DisclosureButton<D: Disclosure>(
    /// What it opens and closes.
    controls: D,
    /// The look; a chip by default.
    #[prop(optional)]
    variant: Variant,
    /// The size; medium by default.
    #[prop(optional)]
    size: Size,
    /// Its name while closed, e.g. "Open navigation".
    #[prop(optional)]
    label_closed: Option<&'static str>,
    /// Its name while open, e.g. "Close navigation".
    #[prop(optional)]
    label_open: Option<&'static str>,
    /// Layout classes added to the variant's (margin, width, position) — never colours.
    #[prop(optional, into)]
    layout: String,
    /// The content: text, an icon, or both.
    children: Children,
) -> impl IntoView {
    let label = move || {
        if controls.is_open() {
            label_open
        } else {
            label_closed
        }
    };
    view! {
        <button
            type="button"
            class=classes(variant, size, &layout)
            aria-controls=controls.id()
            aria-expanded=move || controls.is_open().to_string()
            aria-label=label
            title=label
            on:click=move |_| controls.toggle()
        >
            {children()}
        </button>
    }
}

/// A button that is on or off (`aria-pressed`), flipping `pressed`.
#[component]
pub fn ToggleButton(
    /// On or off; a click flips it.
    pressed: RwSignal<bool>,
    /// The look; a chip by default.
    #[prop(optional)]
    variant: Variant,
    /// The size; medium by default.
    #[prop(optional)]
    size: Size,
    /// Its name for screen readers and its tooltip; needed when the content is only an icon.
    #[prop(optional, into)]
    label: Option<String>,
    /// Layout classes added to the variant's (margin, width, position) — never colours.
    #[prop(optional, into)]
    layout: String,
    /// Disabled while this is true.
    #[prop(optional, into)]
    disabled: Option<Signal<bool>>,
    /// The content: text, an icon, or both.
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=classes(variant, size, &layout)
            aria-pressed=move || pressed.get().to_string()
            aria-label=label.clone()
            title=label
            disabled=move || is_disabled(disabled)
            on:click=move |_| pressed.update(|on| *on = !*on)
        >
            {children()}
        </button>
    }
}

/// A link that looks like a button: a real `<a href>` (new tab, middle
/// click, crawlers). `current` marks the page you are on.
#[component]
pub fn LinkButton(
    /// Where it goes.
    #[prop(into)]
    href: String,
    /// The look; a chip by default.
    #[prop(optional)]
    variant: Variant,
    /// The size; medium by default.
    #[prop(optional)]
    size: Size,
    /// Its name for screen readers and its tooltip; needed when the content is only an icon.
    #[prop(optional, into)]
    label: Option<String>,
    /// It leads to the page being shown (`aria-current="page"`), followed live.
    #[prop(optional, into)]
    current: Option<Signal<bool>>,
    /// Layout classes added to the variant's (margin, width, position) — never colours.
    #[prop(optional, into)]
    layout: String,
    /// The content: text, an icon, or both.
    children: Children,
) -> impl IntoView {
    view! {
        <a
            href=href
            class=classes(variant, size, &layout)
            aria-current=move || current.is_some_and(|c| c.get()).then_some("page")
            aria-label=label.clone()
            title=label
        >
            {children()}
        </a>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests;
