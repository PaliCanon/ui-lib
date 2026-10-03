//! The icons of the identity: inline SVG drawn with `currentColor`, so an
//! icon takes the colour of the text around it (and of its button's states).
//!
//! The set grows one icon at a time, when something needs it: a new icon is
//! a new [`IconName`] and its drawing in `paths`.

use leptos::prelude::*;

/// Every icon of the set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconName {
    /// A button under a pointer: the gallery's buttons page.
    Button,
}

/// The drawing of `name`, on a 24 × 24 grid, as SVG path data.
fn paths(name: IconName) -> &'static [&'static str] {
    match name {
        IconName::Button => &[
            // The button: a pill, open at the bottom right where the pointer sits.
            "M13 16H7a4 4 0 0 1 0-8h10a4 4 0 0 1 3.9 3.1",
            // The pointer.
            "M14 13l7 2.6-3 1.2-1.3 3z",
        ],
    }
}

/// An icon, 18 px by default. Decorative (`aria-hidden`): the button or link
/// around it carries the name.
#[component]
pub fn Icon(
    /// Which icon.
    name: IconName,
    /// Its width and height in pixels.
    #[prop(default = 18)]
    size: u16,
) -> impl IntoView {
    view! {
        <svg
            width=size
            height=size
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            class="shrink-0"
        >
            {paths(name).iter().map(|d| view! { <path d=*d /> }).collect_view()}
        </svg>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn an_icon_is_decorative_and_follows_the_text_colour() {
        let out = Owner::new().with(|| view! { <Icon name=IconName::Button size=20 /> }.to_html());
        assert!(out.contains(r#"aria-hidden="true""#), "{out}");
        assert!(out.contains(r#"stroke="currentColor""#), "{out}");
        assert!(out.contains(r#"width="20""#), "{out}");
        assert_eq!(out.matches("<path").count(), paths(IconName::Button).len());
    }
}
