//! One tone vocabulary for every status-coloured item (079): a [`Badge`],
//! a [`Callout`], a [`Card`]'s edge. A tone is a meaning (it worked, beware,
//! it failed), never a colour: each use has its own class function over the
//! theme's tokens (`--ps-success`, `--ps-success-soft`, …), so every theme
//! paints it and no component carries a hex.
//!
//! `#[non_exhaustive]`: a tone added later is not a breaking change for a
//! service that matches on it.
//!
//! [`Badge`]: crate::badge::Badge
//! [`Callout`]: crate::callout::Callout
//! [`Card`]: crate::surface::Card

/// What a status colour means.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// No status: a label, a count, a plain role.
    #[default]
    Neutral,
    /// The identity's accent: the owner, the current thing.
    Accent,
    /// Something to know.
    Info,
    /// It worked, it is on.
    Success,
    /// Beware: it may cost something.
    Warning,
    /// It failed, or it cannot be undone.
    Danger,
}

/// A badge's fill and text, or its outline when `outline`.
pub(crate) fn badge_classes(tone: Tone, outline: bool) -> &'static str {
    match (tone, outline) {
        (Tone::Neutral, false) => "bg-chip text-muted",
        (Tone::Accent, false) => "bg-accent-soft text-gold-text",
        (Tone::Info, false) => "bg-info-soft text-info",
        (Tone::Success, false) => "bg-success-soft text-success",
        (Tone::Warning, false) => "bg-warning-soft text-warning",
        (Tone::Danger, false) => "bg-danger-soft text-danger-text",
        (Tone::Neutral, true) => "border border-line text-muted",
        (Tone::Accent, true) => "border border-gold text-gold-text",
        (Tone::Info, true) => "border border-info text-info",
        (Tone::Success, true) => "border border-success text-success",
        (Tone::Warning, true) => "border border-warning text-warning",
        (Tone::Danger, true) => "border border-danger text-danger-text",
    }
}

/// The small round mark a badge may lead with.
pub(crate) fn dot_classes(tone: Tone) -> &'static str {
    match tone {
        Tone::Neutral => "bg-muted",
        Tone::Accent => "bg-gold",
        Tone::Info => "bg-info",
        Tone::Success => "bg-success",
        Tone::Warning => "bg-warning",
        Tone::Danger => "bg-danger",
    }
}

/// A callout's box: its soft background, its edge and its text.
pub(crate) fn callout_classes(tone: Tone) -> &'static str {
    match tone {
        Tone::Neutral => "bg-chip border-line text-ink",
        Tone::Accent => "bg-mark border-gold text-ink",
        Tone::Info => "bg-info-soft border-info text-ink",
        Tone::Success => "bg-success-soft border-success text-ink",
        Tone::Warning => "bg-warning-soft border-warning text-ink",
        Tone::Danger => "bg-danger-soft border-danger text-ink",
    }
}

/// A card's edge when it carries a tone (a danger zone).
pub(crate) fn edge_classes(tone: Tone) -> &'static str {
    match tone {
        Tone::Neutral => "border-card-edge",
        Tone::Accent => "border-gold",
        Tone::Info => "border-info",
        Tone::Success => "border-success",
        Tone::Warning => "border-warning",
        Tone::Danger => "border-danger",
    }
}

/// The text colour of a tone, for an icon or a title.
pub(crate) fn text_classes(tone: Tone) -> &'static str {
    match tone {
        Tone::Neutral => "text-muted",
        Tone::Accent => "text-gold-text",
        Tone::Info => "text-info",
        Tone::Success => "text-success",
        Tone::Warning => "text-warning",
        Tone::Danger => "text-danger-text",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Tone; 6] = [
        Tone::Neutral,
        Tone::Accent,
        Tone::Info,
        Tone::Success,
        Tone::Warning,
        Tone::Danger,
    ];

    #[test]
    fn every_tone_has_its_own_classes_in_every_use() {
        for use_of in [
            (|t| badge_classes(t, false)) as fn(Tone) -> &'static str,
            |t| badge_classes(t, true),
            dot_classes,
            callout_classes,
            edge_classes,
            text_classes,
        ] {
            let mut seen: Vec<&str> = ALL.iter().map(|t| use_of(*t)).collect();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), ALL.len(), "two tones share a look: {seen:?}");
        }
    }

    #[test]
    fn a_tone_is_painted_by_theme_tokens_never_a_hex() {
        for tone in ALL {
            for classes in [
                badge_classes(tone, false),
                badge_classes(tone, true),
                callout_classes(tone),
                edge_classes(tone),
            ] {
                assert!(
                    !classes.contains('#') && !classes.contains('['),
                    "{classes}"
                );
            }
        }
    }
}
