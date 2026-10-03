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
    /// A gear: the Settings entry at the bottom of the rail.
    Gear,
    /// A sun: the light theme, in the theme picker.
    Sun,
    /// A crescent moon: the dark theme, in the theme picker.
    Moon,
    /// A monitor: the system theme (the device's own), in the theme picker.
    Monitor,
    /// An arrow pointing left: the "Back" row of a [`crate::menu::DrillMenu`].
    ArrowLeft,
    /// A chevron pointing right: a [`crate::menu::MenuItem::Submenu`]'s entry,
    /// on the right.
    ChevronRight,
    /// A check mark: the selected choice of a [`crate::menu::MenuItem::Choice`].
    Check,
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
        IconName::Gear => &[
            // The toothed ring.
            "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 \
             1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 \
             1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 \
             4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 \
             0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 \
             2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z",
            // The centre hole.
            "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z",
        ],
        IconName::Sun => &[
            // The disc.
            "M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10z",
            // The eight rays.
            "M12 1v2",
            "M12 21v2",
            "M4.22 4.22l1.42 1.42",
            "M18.36 18.36l1.42 1.42",
            "M1 12h2",
            "M21 12h2",
            "M4.22 19.78l1.42-1.42",
            "M18.36 5.64l1.42-1.42",
        ],
        IconName::Moon => &[
            // A disc with a smaller one overlapping it, left as a crescent.
            "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z",
        ],
        IconName::Monitor => &[
            // The screen.
            "M4 3h16a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
            // The stand.
            "M8 21h8",
            "M12 17v4",
        ],
        IconName::ArrowLeft => &["M19 12H5", "M11 18l-6-6 6-6"],
        IconName::ChevronRight => &["M9 18l6-6-6-6"],
        IconName::Check => &["M20 6L9 17l-5-5"],
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
