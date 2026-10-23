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
    /// A chevron pointing left: a carousel's Previous button.
    ChevronLeft,
    /// A check mark: the selected choice of a [`crate::menu::MenuItem::Choice`].
    Check,
    /// Three stacked lines: the gallery's menu page.
    Menu,
    /// A speech-bubble panel: the gallery's popover page.
    Popover,
    /// A 2x2 grid of squares: the gallery's icons page.
    Grid,
    /// An ✕: closes a [`crate::dialog::Dialog`] or the mobile navigation.
    Close,
    /// A window with a header bar: the gallery's dialog page.
    Dialog,
    /// A globe: the language entry of the rail and of the mobile navigation.
    Globe,
    /// Three shrinking lines, trailing a motion: the gallery's animations
    /// page.
    Motion,
    /// A magnifier: a search entry.
    Search,
    /// A plus: what adds (learn-pali's + that adds terms).
    Plus,
    /// A door with an arrow leaving it: the shell's sign-out entry, under
    /// Settings at the bottom of the rail.
    SignOut,
    /// Two people, one behind the other: a team, a workspace's members
    /// (anatta-studio's Workspace entry, where ☰ read as "open the menu").
    Users,
}

/// The drawing of `name`, on a 24 × 24 grid, as SVG path data.
fn paths(name: IconName) -> &'static [&'static str] {
    match name {
        IconName::ArrowLeft => &["M19 12H5", "M11 18l-6-6 6-6"],
        IconName::ChevronRight => &["M9 18l6-6-6-6"],
        IconName::ChevronLeft => &["M15 18l-6-6 6-6"],
        IconName::Check => &["M20 6L9 17l-5-5"],
        IconName::Menu => &["M3 12h18", "M3 6h18", "M3 18h18"],
        IconName::Popover => &[
            // A rounded panel with a small tail, pointing down-left.
            "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z",
        ],
        IconName::Grid => &[
            "M3 3h7v7H3z",
            "M14 3h7v7h-7z",
            "M14 14h7v7h-7z",
            "M3 14h7v7H3z",
        ],
        IconName::Close => &["M4 4l16 16", "M20 4L4 20"],
        IconName::Plus => &["M12 5v14", "M5 12h14"],
        IconName::SignOut => &[
            // The door frame, open on the right.
            "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4",
            // The arrow, leaving through it.
            "M16 17l5-5-5-5",
            "M21 12H9",
        ],
        IconName::Users => &[
            // The person in front: shoulders, then head.
            "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2",
            "M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z",
            // The one behind: a shoulder, and the edge of a head.
            "M22 21v-2a4 4 0 0 0-3-3.87",
            "M16 3.13a4 4 0 0 1 0 7.75",
        ],
        IconName::Dialog => &[
            // The window frame.
            "M3 4h18a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z",
            // The header bar's divider.
            "M2 9h20",
        ],
        IconName::Globe => &[
            // The sphere, its equator, and one meridian.
            "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z",
            "M2 12h20",
            "M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z",
        ],
        IconName::Motion => &[
            // Three trailing lines, shrinking towards the moving shape.
            "M3 7h10", "M3 12h15", "M3 17h7",
        ],
        IconName::Search => &[
            // The lens.
            "M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0z",
            // The handle.
            "M15 15l6 6",
        ],
        other => core_paths(other),
    }
}

/// [`paths`]'s first icons — the oldest, and the only ones with fiddly
/// multi-line drawings (`Gear`, `Sun`) — split out so neither function
/// grows past the line limit.
fn core_paths(name: IconName) -> &'static [&'static str] {
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
        _ => unreachable!("paths dispatches every other IconName to core_paths"),
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

    #[test]
    fn every_icon_draws_at_least_one_path() {
        for name in [
            IconName::Button,
            IconName::Gear,
            IconName::Sun,
            IconName::Moon,
            IconName::Monitor,
            IconName::ArrowLeft,
            IconName::ChevronRight,
            IconName::ChevronLeft,
            IconName::Check,
            IconName::Menu,
            IconName::Popover,
            IconName::Grid,
            IconName::Close,
            IconName::Dialog,
            IconName::Globe,
            IconName::Motion,
            IconName::Search,
            IconName::Plus,
            IconName::SignOut,
            IconName::Users,
        ] {
            assert!(!paths(name).is_empty(), "{name:?}");
        }
    }

    #[test]
    fn chevron_left_mirrors_chevron_right() {
        let out = Owner::new().with(|| view! { <Icon name=IconName::ChevronLeft /> }.to_html());
        assert!(out.contains(r#"d="M15 18l-6-6 6-6""#), "{out}");
        assert!(out.contains(r#"aria-hidden="true""#), "{out}");
    }
}
