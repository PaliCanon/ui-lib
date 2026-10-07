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
    /// A box of stacked content: a counter cell, a link or a plain surface.
    Tile,
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

/// Shared by all variants: layout, focus ring, disabled. No gap: each variant
/// (or, for a tile, its size) sets its own, so one utility sets it and a
/// smaller one is not outranked by a base `gap-2`.
const BASE: &str = "inline-flex shrink-0 cursor-pointer select-none items-center \
    font-medium transition focus-visible:outline-2 focus-visible:outline-offset-2 \
    focus-visible:outline-gold disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-40";

/// The resting classes of a tile as a literal, so that [`TILE_SURFACE`] and
/// the variant's arm (`concat!` takes literals, not constants) share one text.
macro_rules! tile_surface {
    () => {
        "flex-col items-center justify-center rounded-xl border border-gold/25 bg-chip px-3 text-center"
    };
}

/// The resting look of a [`Variant::Tile`], with no state in it: a plain
/// `<div>` cell uses it alone (adding `flex`, which `BASE`'s `inline-flex`
/// stands for on a link), a link gets it through the variant. `items-center`
/// repeats `BASE` on purpose, so the plain cell aligns its content like the
/// link; its vertical padding and gap come from [`tile_spacing`], for both.
pub(crate) const TILE_SURFACE: &str = tile_surface!();

/// The tile's vertical padding and gap for `size`.
pub(crate) fn tile_spacing(size: Size) -> &'static str {
    match size {
        Size::Sm => "gap-0 py-1.5",
        Size::Md => "gap-2 py-2",
        Size::Lg => "gap-2 py-3",
    }
}

fn variant_classes(variant: Variant) -> &'static str {
    match variant {
        Variant::Tile => concat!(
            tile_surface!(),
            " hover:border-gold/60 active:bg-chip-hover aria-pressed:border-gold aria-pressed:bg-gold/10 aria-pressed:hover:border-gold"
        ),
        Variant::Icon => {
            "gap-2 justify-center rounded-xl border border-transparent text-muted hover:border-gold/60 hover:bg-chip \
             hover:text-heading active:bg-chip-hover aria-expanded:bg-gold/15 aria-expanded:text-heading \
             aria-pressed:bg-gold/15 aria-pressed:text-heading"
        }
        Variant::Chip => {
            "gap-2 justify-center rounded-full border border-line bg-chip text-muted hover:border-gold/60 hover:text-heading \
             active:bg-chip-hover aria-expanded:border-gold aria-expanded:bg-gold/10 \
             aria-expanded:text-heading aria-pressed:border-gold aria-pressed:bg-gold/10 aria-pressed:text-heading"
        }
        Variant::Primary => {
            "gap-2 justify-center rounded-full bg-primary text-on-primary hover:bg-primary-hover active:brightness-95"
        }
        Variant::Accent => {
            "gap-2 justify-center rounded-full border border-gold/40 bg-gold/5 text-heading hover:border-gold hover:bg-gold/10 \
             active:bg-gold/15 aria-expanded:border-gold aria-expanded:bg-gold/15 aria-expanded:hover:bg-gold/15 \
             aria-pressed:border-gold aria-pressed:bg-gold/15 aria-pressed:hover:bg-gold/15"
        }
        Variant::Nav => {
            "w-full justify-start whitespace-nowrap rounded-xl text-muted hover:bg-chip hover:text-heading \
             active:bg-chip-hover aria-[current=page]:bg-gold/10 aria-[current=page]:text-heading \
             aria-checked:bg-gold/15 aria-checked:text-heading aria-expanded:bg-gold/15 aria-expanded:text-heading"
        }
        Variant::Ghost => {
            "gap-2 justify-center rounded-sm text-gold-text underline underline-offset-4 hover:text-heading"
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
        // Taller than the label needs when the content stacks: a minimum, not a height.
        (Variant::Tile, Size::Sm) => "min-h-8",
        (Variant::Tile, Size::Md) => "min-h-11",
        (Variant::Tile, Size::Lg) => "min-h-13",
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
    if variant == Variant::Tile {
        out.push(' ');
        out.push_str(tile_spacing(size));
    }
    if !layout.is_empty() {
        out.push(' ');
        out.push_str(layout);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Variant; 7] = [
        Variant::Icon,
        Variant::Chip,
        Variant::Primary,
        Variant::Accent,
        Variant::Ghost,
        Variant::Nav,
        Variant::Tile,
    ];

    #[test]
    fn accent_shows_open_and_pressed_and_keeps_them_under_the_pointer() {
        let c = classes(Variant::Accent, Size::Md, "");
        for state in ["aria-expanded", "aria-pressed"] {
            for class in [
                format!("{state}:border-gold"),
                format!("{state}:bg-gold/15"),
                format!("{state}:hover:bg-gold/15"),
            ] {
                assert!(c.split(' ').any(|t| t == class), "{class} in {c}");
            }
        }
    }

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
    fn ghost_text_uses_the_gold_text_token_that_reaches_aa() {
        let c = classes(Variant::Ghost, Size::Md, "");
        assert!(c.contains("text-gold-text"), "{c}");
        assert!(!c.contains(" text-gold "), "{c}");
    }

    fn tokens(classes: &str) -> Vec<&str> {
        classes.split_whitespace().collect()
    }

    #[test]
    fn tile_defines_every_state() {
        let c = classes(Variant::Tile, Size::Md, "");
        for state in [
            "hover:border-gold/60",
            "active:bg-chip-hover",
            "disabled:opacity-40",
            "focus-visible:outline-gold",
        ] {
            assert!(c.contains(state), "{state} in {c}");
        }
    }

    #[test]
    fn tile_shows_pressed_like_a_chip_and_only_through_its_state() {
        let c = classes(Variant::Tile, Size::Md, "");
        assert!(
            tokens(&c).contains(&"aria-pressed:border-gold")
                && tokens(&c).contains(&"aria-pressed:bg-gold/10"),
            "{c}"
        );
        // Pressed wins under the pointer too: `hover:border-gold/60` sets the
        // same property, so the pressed look is repeated for the hover state.
        assert!(
            tokens(&c).contains(&"aria-pressed:hover:border-gold"),
            "{c}"
        );
        // The resting surface stays stateless: only the variant reacts.
        assert!(!TILE_SURFACE.contains("aria-pressed"), "{TILE_SURFACE}");
    }

    #[test]
    fn tile_padding_and_gap_follow_its_size() {
        for (size, spacing) in [
            (Size::Sm, ["gap-0", "py-1.5"]),
            (Size::Md, ["gap-2", "py-2"]),
            (Size::Lg, ["gap-2", "py-3"]),
        ] {
            let c = classes(Variant::Tile, size, "");
            for class in spacing {
                assert!(tokens(&c).contains(&class), "{class} in {c}");
            }
        }
        let sm = classes(Variant::Tile, Size::Sm, "");
        assert!(!tokens(&sm).contains(&"py-2"), "{sm}");
        assert!(!tokens(&sm).contains(&"gap-2"), "{sm}");
    }

    #[test]
    fn the_tile_surface_leaves_padding_and_gap_to_the_size() {
        for class in ["py-", "gap-"] {
            assert!(!TILE_SURFACE.contains(class), "{class} in {TILE_SURFACE}");
        }
        assert!(tokens(TILE_SURFACE).contains(&"px-3"), "{TILE_SURFACE}");
    }

    #[test]
    fn tile_spacing_is_the_one_the_plain_cell_shares() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let c = classes(Variant::Tile, size, "");
            for class in tile_spacing(size).split_whitespace() {
                assert!(tokens(&c).contains(&class), "{class} in {c}");
            }
        }
    }

    #[test]
    fn no_variant_sets_the_gap_or_the_vertical_padding_twice() {
        // Tailwind writes a smaller `gap-0` before `gap-2`, so a second
        // utility would silently lose (ui-components.md).
        for v in ALL {
            for size in [Size::Sm, Size::Md, Size::Lg] {
                let c = classes(v, size, "");
                for prefix in ["gap-", "py-"] {
                    let n = tokens(&c).iter().filter(|t| t.starts_with(prefix)).count();
                    assert!(n <= 1, "{prefix} {n} times in {c}");
                }
            }
        }
    }

    #[test]
    fn tile_is_a_stacked_centred_box() {
        let c = classes(Variant::Tile, Size::Md, "");
        let t = tokens(&c);
        for class in [
            "flex-col",
            "items-center",
            "justify-center",
            "text-center",
            "rounded-xl",
            "border",
            "border-gold/25",
            "bg-chip",
        ] {
            assert!(t.contains(&class), "{class} in {c}");
        }
        assert!(!t.contains(&"justify-start"), "{c}");
    }

    #[test]
    fn tile_extends_the_resting_surface() {
        assert!(TILE_SURFACE.contains("rounded-xl border border-gold/25 bg-chip"));
        assert!(variant_classes(Variant::Tile).starts_with(TILE_SURFACE));
    }

    #[test]
    fn the_resting_surface_has_no_state() {
        assert!(TILE_SURFACE.contains("flex-col"), "{TILE_SURFACE}");
        for state in [
            "hover:",
            "active:",
            "focus-visible:",
            "disabled:",
            "aria-",
            "cursor-pointer",
        ] {
            assert!(!TILE_SURFACE.contains(state), "{state} in {TILE_SURFACE}");
        }
    }

    #[test]
    fn tile_sizes_are_its_own() {
        // The wildcard arm of `size_classes` would give a Tile `h-11 px-4`
        // without a compile error: this is the only guard.
        for (size, min) in [
            (Size::Sm, "min-h-8"),
            (Size::Md, "min-h-11"),
            (Size::Lg, "min-h-13"),
        ] {
            let c = classes(Variant::Tile, size, "");
            assert!(tokens(&c).contains(&min), "{min} in {c}");
            assert!(
                !c.contains("h-11 px-4") && !c.contains(" h-8 ") && !c.contains(" h-13 "),
                "{c}"
            );
        }
    }

    #[test]
    fn tile_adds_no_second_display_to_the_base() {
        // `BASE` is `inline-flex`: a bare `flex` on the same element would be
        // two utilities for one property. A plain cell adds `flex` itself.
        let c = classes(Variant::Tile, Size::Md, "");
        assert!(tokens(&c).contains(&"inline-flex"), "{c}");
        assert!(!tokens(&c).contains(&"flex"), "{c}");
        assert!(!tokens(TILE_SURFACE).contains(&"flex"), "{TILE_SURFACE}");
    }

    #[test]
    fn defaults_are_a_medium_chip() {
        assert_eq!(Variant::default(), Variant::Chip);
        assert_eq!(Size::default(), Size::Md);
    }
}
