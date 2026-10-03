//! Motion tokens, and the handful of CSS classes [`crate::presence::Presence`]
//! builds on them: durations (`--dur-fast/base/slow`), easings
//! (`--ease-out/in-out`), and one bouncier curve (`--spring`), declared once
//! in `theme.css` (not theme-scoped: the same under light, dark and
//! system). A [`Motion`] names which of the library's own classes
//! (`pc-motion-*`) a [`crate::presence::Presence`] wears — each animates
//! `opacity`, and every one but [`Motion::Fade`] also `transform`: never a
//! layout property (the rail width transition in `shell.rs` is the one
//! exception elsewhere in the app, left as its own single CSS transition).
//!
//! Under `prefers-reduced-motion: reduce`, or inside an ancestor carrying
//! `data-motion="reduce"` (the gallery's live toggle), every `pc-motion-*`
//! class drops its `transform` half and keeps only a quick opacity
//! crossfade (`theme.css`'s reduced-motion block).

/// Which edge a [`Motion::Slide`] enters from (and exits back towards).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// Slides down from above.
    Top,
    /// Slides up from below.
    Bottom,
    /// Slides in from the left.
    Left,
    /// Slides in from the right.
    Right,
}

/// An animation a [`crate::presence::Presence`] plays between its closed and
/// open `data-state`: a crossfade, a crossfade with a small scale, or a
/// crossfade with a small slide from an [`Edge`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// Opacity only.
    Fade,
    /// Opacity and a small scale, grown in with [`crate::motion`]'s `--spring`.
    Scale,
    /// Opacity and a small slide from an [`Edge`].
    Slide(Edge),
}

impl Motion {
    /// The library-owned classes a [`crate::presence::Presence`] wrapper
    /// wears for this motion: the shared `pc-motion` (duration, easing,
    /// the reduced-motion carve-out) and this variant's own (which
    /// properties animate, and the closed/open values of each).
    pub(crate) fn classes(self) -> &'static str {
        match self {
            Motion::Fade => "pc-motion pc-motion-fade",
            Motion::Scale => "pc-motion pc-motion-scale",
            Motion::Slide(Edge::Top) => "pc-motion pc-motion-slide-top",
            Motion::Slide(Edge::Bottom) => "pc-motion pc-motion-slide-bottom",
            Motion::Slide(Edge::Left) => "pc-motion pc-motion-slide-left",
            Motion::Slide(Edge::Right) => "pc-motion pc-motion-slide-right",
        }
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    const CSS: &str = include_str!("../style/theme.css");

    #[test]
    fn each_motion_wears_the_shared_class_and_its_own() {
        assert_eq!(Motion::Fade.classes(), "pc-motion pc-motion-fade");
        assert_eq!(Motion::Scale.classes(), "pc-motion pc-motion-scale");
        assert_eq!(
            Motion::Slide(Edge::Top).classes(),
            "pc-motion pc-motion-slide-top"
        );
        assert_eq!(
            Motion::Slide(Edge::Bottom).classes(),
            "pc-motion pc-motion-slide-bottom"
        );
        assert_eq!(
            Motion::Slide(Edge::Left).classes(),
            "pc-motion pc-motion-slide-left"
        );
        assert_eq!(
            Motion::Slide(Edge::Right).classes(),
            "pc-motion pc-motion-slide-right"
        );
    }

    #[test]
    fn theme_declares_every_motion_token_exactly_once() {
        for token in [
            "--dur-fast",
            "--dur-base",
            "--dur-slow",
            "--ease-out",
            "--ease-in-out",
            "--spring",
        ] {
            assert_eq!(
                CSS.matches(&format!("{token}:")).count(),
                1,
                "{token} declared once: {CSS}"
            );
        }
    }

    #[test]
    fn every_motion_class_exists_and_only_touches_opacity_or_transform() {
        for class in [
            ".pc-motion-fade",
            ".pc-motion-scale",
            ".pc-motion-slide-top",
            ".pc-motion-slide-bottom",
            ".pc-motion-slide-left",
            ".pc-motion-slide-right",
        ] {
            assert!(CSS.contains(class), "{class} missing: {CSS}");
        }
        for banned in ["width", "height", "padding", "margin", "left", "top"] {
            assert!(
                !CSS.contains(&format!("transition-property: {banned}")),
                "a Motion class animates {banned}, a layout property: {CSS}"
            );
        }
    }

    #[test]
    fn reduced_motion_drops_transform_and_keeps_only_opacity() {
        // Both the OS preference and the gallery's live toggle carve the
        // same opacity-only exception out of `.pc-motion`.
        assert!(CSS.contains("prefers-reduced-motion: reduce"));
        assert!(CSS.contains(r#"[data-motion="reduce"]"#));
        assert_eq!(
            CSS.matches("transition-property: opacity !important")
                .count(),
            2,
            "one carve-out for the media query, one for the ancestor toggle: {CSS}"
        );
    }
}
