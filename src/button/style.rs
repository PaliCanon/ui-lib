//! The look of every button: a variant and a size. One function gives the
//! classes for every state; components never style states themselves
//! (`docs/conventions/ui-components.md`).

/// What a button looks like. Each variant defines every state: at rest,
/// hovered, keyboard-focused, open or on, pressed, disabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// A bare icon: the lotus, a close cross, a gear. Always square.
    Icon,
    /// A rounded chip: filters, examples, "Read on SuttaCentral".
    #[default]
    Chip,
    /// The main action of a screen: the search button.
    Primary,
    /// A highlighted invitation: "Explore with AI".
    Accent,
    /// A quiet text action: "Try again".
    Ghost,
    /// An entry of a navigation list: icon then label, full width, marked
    /// when it leads to the page being shown.
    Nav,
}

/// How big a button is. `Md` is the default and the smallest comfortable
/// touch target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// Dense rows of chips.
    Sm,
    /// Everything else.
    #[default]
    Md,
    /// The search button, the rows of the mobile navigation.
    Lg,
}

/// Shared by all variants: layout, focus ring, disabled.
const BASE: &str = "inline-flex shrink-0 cursor-pointer select-none items-center gap-2 \
    font-medium transition focus-visible:outline-2 focus-visible:outline-offset-2 \
    focus-visible:outline-gold disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-40";

fn variant_classes(variant: Variant) -> &'static str {
    match variant {
        Variant::Icon => {
            "justify-center rounded-xl border border-transparent text-muted hover:border-gold/60 hover:bg-chip \
             hover:text-heading active:bg-chip-hover aria-expanded:bg-gold/15 aria-expanded:text-heading \
             aria-pressed:bg-gold/15 aria-pressed:text-heading"
        }
        Variant::Chip => {
            "justify-center rounded-full border border-line bg-chip text-muted hover:border-gold/60 hover:text-heading \
             active:bg-chip-hover aria-expanded:border-gold aria-expanded:bg-gold/10 \
             aria-expanded:text-heading aria-pressed:border-gold aria-pressed:bg-gold/10 aria-pressed:text-heading"
        }
        Variant::Primary => {
            "justify-center rounded-full bg-primary text-on-primary hover:bg-primary-hover active:brightness-95"
        }
        Variant::Accent => {
            "justify-center rounded-full border border-gold/40 bg-gold/5 text-heading hover:border-gold hover:bg-gold/10 \
             active:bg-gold/15"
        }
        Variant::Nav => {
            "w-full justify-start whitespace-nowrap rounded-xl text-muted hover:bg-chip hover:text-heading \
             active:bg-chip-hover aria-[current=page]:bg-gold/10 aria-[current=page]:text-heading \
             aria-checked:bg-gold/15 aria-checked:text-heading aria-expanded:bg-gold/15 aria-expanded:text-heading"
        }
        Variant::Ghost => {
            "justify-center rounded-sm text-gold underline underline-offset-4 hover:text-heading"
        }
    }
}

fn size_classes(variant: Variant, size: Size) -> &'static str {
    match (variant, size) {
        (Variant::Ghost, Size::Sm) => "text-[13px]",
        (Variant::Ghost, Size::Md) => "text-sm",
        (Variant::Ghost, Size::Lg) => "text-[15px]",
        (Variant::Icon, Size::Sm) => "size-8",
        (Variant::Icon, Size::Md) => "size-11",
        (Variant::Icon, Size::Lg) => "size-13",
        // The icon sits where an `Icon` button's would: centred in the first 44 px.
        (Variant::Nav, Size::Sm) => "h-8 gap-3 px-[7px] text-[13px]",
        (Variant::Nav, Size::Md) => "h-11 gap-3 px-[13px] text-sm",
        (Variant::Nav, Size::Lg) => "h-13 gap-4 px-4 text-base",
        (_, Size::Sm) => "h-8 px-3 text-[13px]",
        (_, Size::Md) => "h-11 px-4 text-sm",
        (_, Size::Lg) => "h-13 px-5 text-[15px]",
    }
}

/// The classes of a button of `variant` and `size`, plus the caller's
/// `layout` classes (margin, width, position — never colours).
#[must_use]
pub fn classes(variant: Variant, size: Size, layout: &str) -> String {
    let mut out = format!(
        "{BASE} {} {}",
        variant_classes(variant),
        size_classes(variant, size)
    );
    if !layout.is_empty() {
        out.push(' ');
        out.push_str(layout);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Variant; 6] = [
        Variant::Icon,
        Variant::Chip,
        Variant::Primary,
        Variant::Accent,
        Variant::Ghost,
        Variant::Nav,
    ];

    #[test]
    fn primary_fills_from_its_own_tokens_not_the_gold_accent() {
        let c = classes(Variant::Primary, Size::Md, "");
        assert!(
            c.contains("bg-primary text-on-primary hover:bg-primary-hover"),
            "{c}"
        );
        assert!(!c.contains("bg-gold "), "{c}");
    }

    #[test]
    fn every_theme_block_defines_the_rail_and_primary_tokens() {
        let css = include_str!("../../style/theme.css");
        for token in [
            "--ps-rail:",
            "--ps-primary:",
            "--ps-primary-hover:",
            "--ps-on-primary:",
        ] {
            // dark :root, explicit light, system light
            assert_eq!(css.matches(token).count(), 3, "{token}");
        }
        // and Tailwind knows them: without the @theme mapping, bg-primary emits nothing
        for mapping in [
            "--color-rail: var(--ps-rail);",
            "--color-primary: var(--ps-primary);",
            "--color-primary-hover: var(--ps-primary-hover);",
            "--color-on-primary: var(--ps-on-primary);",
        ] {
            assert!(css.contains(mapping), "{mapping}");
        }
    }

    #[test]
    fn every_variant_has_focus_and_disabled_states() {
        for v in ALL {
            let c = classes(v, Size::Md, "");
            assert!(c.contains("focus-visible:outline-gold"), "{v:?}");
            assert!(c.contains("disabled:opacity-40"), "{v:?}");
            assert!(c.contains("hover:"), "{v:?} reacts to hover");
        }
    }

    #[test]
    fn togglable_variants_show_open_and_on() {
        for v in [Variant::Icon, Variant::Chip] {
            let c = classes(v, Size::Md, "");
            assert!(
                c.contains("aria-expanded:") && c.contains("aria-pressed:"),
                "{v:?}"
            );
        }
    }

    #[test]
    fn icons_are_square_and_layout_is_appended() {
        assert!(classes(Variant::Icon, Size::Sm, "").contains("size-8"));
        assert!(classes(Variant::Icon, Size::Lg, "").contains("size-13"));
        assert!(classes(Variant::Chip, Size::Lg, "").contains("h-13 px-5"));
        assert!(classes(Variant::Chip, Size::Md, "m-3 w-full").ends_with(" m-3 w-full"));
        assert!(!classes(Variant::Chip, Size::Md, "").ends_with(' '));
    }

    #[test]
    fn nav_entries_start_left_and_mark_the_current_page() {
        let c = classes(Variant::Nav, Size::Md, "");
        assert!(
            c.contains("justify-start") && !c.contains("justify-center"),
            "{c}"
        );
        assert!(c.contains("aria-[current=page]:"), "{c}");
        for v in [Variant::Icon, Variant::Chip, Variant::Primary] {
            assert!(classes(v, Size::Md, "").contains("justify-center"), "{v:?}");
        }
    }

    #[test]
    fn nav_entries_also_mark_a_checked_radio_or_an_open_disclosure() {
        let c = classes(Variant::Nav, Size::Md, "");
        assert!(c.contains("aria-checked:bg-gold/15"), "{c}");
        assert!(c.contains("aria-expanded:bg-gold/15"), "{c}");
    }

    #[test]
    fn defaults_are_a_medium_chip() {
        assert_eq!(Variant::default(), Variant::Chip);
        assert_eq!(Size::default(), Size::Md);
    }
}
