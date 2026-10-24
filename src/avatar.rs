//! [`Avatar`] (079): who someone is, at a glance — their picture, else their
//! initials; a dashed ring with an envelope while they are only invited.
//!
//! Precedence: `pending` wins (no account yet, so no picture); then the
//! picture, which falls back to the initials if it fails to load; then the
//! initials.

use leptos::prelude::*;

use crate::button::Size;
use crate::icon::{Icon, IconName};

/// The first letters of the first two words of `name`, uppercased; `?` for
/// a name with no letter. By character, not grapheme: an accented letter
/// written precomposed (é) keeps its accent, which covers the names seen.
#[must_use]
pub fn initials(name: &str) -> String {
    let letters: String = name
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .flat_map(char::to_uppercase)
        .collect();
    if letters.is_empty() {
        "?".to_string()
    } else {
        letters
    }
}

fn box_size(size: Size) -> (&'static str, u16) {
    match size {
        Size::Sm => ("size-7 text-[11px]", 12),
        Size::Md => ("size-9 text-xs", 14),
        Size::Lg => ("size-12 text-sm", 18),
    }
}

/// A round mark for a person: picture, initials, or the pending ring.
#[component]
pub fn Avatar(
    /// Their name (or address): the initials and the accessible name.
    #[prop(into)]
    name: String,
    /// Their picture, if any.
    #[prop(optional, into)]
    image: Option<String>,
    /// Invited, no account yet: a dashed ring with an envelope.
    #[prop(optional)]
    pending: bool,
    /// Its size; medium by default.
    #[prop(optional)]
    size: Size,
) -> impl IntoView {
    let (dims, icon) = box_size(size);
    let base = format!(
        "inline-flex shrink-0 items-center justify-center rounded-full font-semibold {dims}"
    );
    if pending {
        let class = format!("{base} border border-dashed border-line text-muted");
        return view! {
            <span class=class role="img" aria-label=name>
                <Icon name=IconName::Mail size=icon />
            </span>
        }
        .into_any();
    }
    let letters = initials(&name);
    match image {
        Some(src) => picture(name, letters, src, &base).into_any(),
        None => initials_mark(name, letters, &base).into_any(),
    }
}

/// The initials, named for screen readers.
fn initials_mark(name: String, letters: String, base: &str) -> impl IntoView + use<> {
    let class = format!("{base} bg-chip text-ink");
    view! {
        <span class=class role="img" aria-label=name>
            {letters}
        </span>
    }
}

/// The picture, falling back to the initials when it fails to load.
fn picture(name: String, letters: String, src: String, base: &str) -> impl IntoView + use<> {
    let failed = RwSignal::new(false);
    let img = NodeRef::<leptos::html::Img>::new();
    // A picture that failed before the page was hydrated fired its `error`
    // with no handler attached yet: read its state once mounted. Gated on
    // `browser`, which both client modes include (`hydrate` and `csr` in
    // Cargo.toml); a loaded picture has a natural width, so 0 is a failure.
    #[cfg(feature = "browser")]
    Effect::new(move |_| {
        if img
            .get()
            .is_some_and(|el| el.complete() && el.natural_width() == 0)
        {
            failed.set(true);
        }
    });
    let picture_class = format!("{base} object-cover");
    let fallback = {
        let (name, letters, base) = (name.clone(), letters, base.to_string());
        move || initials_mark(name.clone(), letters.clone(), &base)
    };
    view! {
        <Show when=move || !failed.get() fallback=fallback>
            <img
                node_ref=img
                class=picture_class.clone()
                src=src.clone()
                alt=name.clone()
                on:error=move |_| failed.set(true)
            />
        </Show>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn html(view: impl FnOnce() -> AnyView) -> String {
        Owner::new().with(|| view().to_html())
    }

    #[test]
    fn initials_are_the_first_letters_of_two_words() {
        assert_eq!(initials("Thibault Rivière"), "TR");
        assert_eq!(initials("élodie ângelo"), "ÉÂ");
        assert_eq!(initials("mr.riviere.thibault@gmail.com"), "M");
        assert_eq!(initials("山田 太郎"), "山太");
        assert_eq!(initials("  "), "?");
        assert_eq!(initials("Anne Marie Claire"), "AM");
    }

    #[test]
    fn without_a_picture_the_initials_are_named_for_readers() {
        let out = html(|| view! { <Avatar name="Thibault Rivière" /> }.into_any());
        assert!(
            out.contains(">TR<") && out.contains(r#"role="img""#),
            "{out}"
        );
        assert!(out.contains(r#"aria-label="Thibault Rivière""#), "{out}");
    }

    #[test]
    fn a_picture_carries_its_alt() {
        let out = html(|| view! { <Avatar name="Ann" image="/a.png" /> }.into_any());
        assert!(
            out.contains(r#"src="/a.png""#) && out.contains(r#"alt="Ann""#),
            "{out}"
        );
    }

    #[test]
    fn pending_wins_over_a_picture() {
        let out = html(|| {
            view! { <Avatar name="x@example.org" image="/a.png" pending=true /> }.into_any()
        });
        assert!(
            out.contains("border-dashed") && !out.contains("<img"),
            "{out}"
        );
    }
}
