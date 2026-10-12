# ui-lib

PaliCanon's UI library: the Leptos (SSR + hydrate) shell, components,
theme, languages and motion every service builds on. Items arrive by
promotion from `PaliCanon/ui-lab`, once validated there (same code, same
tests); `ui-lab`'s gallery shows every one of them. Services depend on
`ui-lib` only. Rules: `docs/conventions/ui-lib.md`.

It never holds a project's content (copy, data, routes, screens).

## Items

| Item | What | Used by |
| --- | --- | --- |
| `persisted`, `persisted_with` | A value kept in a cookie shared by every `*.palicanon.app` service, read by the server for the first render; `persisted_with` starts from a value the caller resolved itself and only writes the cookie | `AppShell`, `theme_attribute`, `provide_language` |
| `Language`, `provide_language`, `language_signal`, `language_attribute`, `resolve_language` | The interface language (English, Français, Tiếng Việt): `resolve_language` is the one owner of the server's choice (cookie `pc_lang`, then `Accept-Language`, then English), `provide_language` puts the page's one signal at the app root (the server's shell and the hydrated root; `AppShell` calls it too), `language_signal` is that signal, which every catalogue reads — from the request on the server, from `<html lang>` when hydrating; `language_attribute` is the server shell's `<html lang>`; a choice is kept in `pc_lang` and applies live. Each crate keeps its strings in its own typed `leptos_i18n` catalogue (`build.rs` from `locales/<code>.json`, a missing key or a placeholder not in every locale fails the build) whose `Locale` has an exhaustive `From<Language>`; the library's own strings follow it | `AppShell`, gallery (ui-lab) |
| `Motion`, `Presence` | Motion tokens (`--dur-*`, `--ease-*`, `--spring`) and the `pc-motion-*` classes a `Motion` (Fade, Scale, Slide) builds on them; `Presence` keeps its content mounted through its exit, playing one | gallery (ui-lab) |
| `Button`, `DisclosureButton`, `ToggleButton`, `LinkButton`, `RadioButton` | The button family: appearance is a `Variant` (Icon, Chip, Primary, Accent, Ghost, Nav, Tile — a box of stacked content: a counter cell, a link, a selectable cell shown pressed) and a `Size` (Sm, Md, Lg); behaviour is the component (click, open/close something, on/off, navigate, one choice of a group) | `AppShell`, gallery (ui-lab) |
| `Disclosure`, `Disclosed` | Anything that opens and closes (an id, open or not, toggle), so a `DisclosureButton` can control it and name it | `AppShell`, `Popover`, `Dialog` |
| `Icon`, `IconName` | The icons of the identity, inline SVG on `currentColor` | `nav`, `AppShell`, `theme` |
| `Popover` | A floating panel tied to a `Disclosure`; closes itself on Escape and on an outside click once hydrated | `Menu`, gallery (ui-lab) |
| `Menu`, `MenuItem`, `DrillMenu` | A list of items (`Link`, `Action`, `Choice`, `Submenu`, `Custom`), with submenus: `Menu` is a `Popover` with submenus nested to the right (desktop: hover, click or ArrowRight; ArrowLeft/Escape closes one level); `DrillMenu` is the same items inline, a chosen submenu replacing the list with a "Back" row (mobile) | gallery (ui-lab) |
| `Theme`, `ThemePicker`, `theme_signal`, `theme_menu`, `theme_attribute` | Light, Dark or System (the device's own), persisted in `pc_theme`; `ThemePicker` chooses it with picture cards (a radio group, each card a preview of the page in its palette), what `AppShell`'s Settings use; `theme_menu` is a helper `Submenu` of `MenuItem::Choice`s that chooses it, for an app that wants a theme menu instead, `theme_attribute` is what the server puts on `<html>` for the first render | `AppShell`, gallery (ui-lab) |
| `Dialog`, `DialogSection` | A modal dialog tied to a `Disclosure`: a centred panel over a dimmed, blurred backdrop (a full-screen sheet on mobile), pinned header (its title a string or a signal, followed in place) and optional footer, only the body scrolls, its content titled `DialogSection`s; at most one open at a time — a second open replaces it, unless it is not `dismissable` (a required confirmation), which refuses the second | `AppShell` (Settings), gallery (ui-lab) |
| `AppShell` | The shell: one column at least a screen tall — the top bar (mobile), then the rail first, at the page's full height (desktop), beside the page column, which ends with the optional `footer` slot (at the bottom of a short page); navigation rail on desktop (the lotus opens and closes it, remembered) with a Settings gear pinned at its bottom, full-screen navigation on mobile with a Settings row; above Settings, the language (the globe and the current language's name: a `Menu` of the languages on desktop, a row drilling down to them on mobile, back at the row whenever the navigation reopens); the env badge, when `env_badge` is given, below Settings (desktop; hidden while the rail is collapsed) or in the top bar (mobile); the gear and the row open the same Settings `Dialog`, whose content is its `settings` prop (by default one Appearance section: the theme as a `ThemePicker`, picture cards of Light, Dark and System); Escape closes the language menu or Settings alone when one is open, backs out of the mobile languages one level when drilled into them, and closes the rail and the mobile navigation otherwise | gallery (ui-lab) |
| `EnvBadge`, `EnvBadgeInfo` | Which build is running: a small mono badge, `env · sha7`, muted colours, never interactive; `AppShell`'s `env_badge` prop shows it at the rail bottom (desktop) and in the top bar (mobile) | `AppShell`, gallery (ui-lab) |
| `Footer`, `FooterLink` | A generic page footer: the caller's own `links` (a real `<a>` each), `credits` (one line each — texts, translators, a dictionary's licence…), and optional `leading` content (a build badge, a version); the footer owns only the layout and its own nav label, translated: on a phone stacked and centred (leading, then links, then credits); from `sm` a 1fr/auto/1fr grid — the leading at the left edge, links centred on the full width, credits at the footer's right edge, their lines centred on each other; no credits renders no credits block, no links no nav, no leading nothing (all optional, credits empty by default) | gallery (ui-lab) |
| `SearchCard`, `SearchRow` | The two search forms, plain `<form role="search" method="get">`s the browser submits itself (no JavaScript needed): `SearchCard` is the home's large card — a big field, the caller's controls (filters, dropdowns) in its bottom row at the left, a round gold submit with an arrow at the right, a soft halo; `SearchRow` is a results page's one-line bar — a search icon, the field, a gold submit flush at the right ("Search" on a desktop, an arrow on a phone), whose form `id` (`form_id`, required: no default) lets fields elsewhere on the page join the same GET through the `form` attribute. The caller gives every string (label, placeholder, submit name); `name` (default `q`) is the field's name and id. Ids are unique per page: a second form passes a distinct `name` (and `form_id`). The submit dims while the field is empty, in CSS (`group/card` and `group/row` are page-global group names the dim classes depend on) | search-front (after task 005/the switch), gallery (ui-lab) |
| `Pali`, `PaliSize` | A Pāli text: a `<span lang="pi">` in the serif face, sized Body, Lemma or Display with a line height that never clips ṭ ḍ ṃ ñ; no colour of its own, no state | learn-pali (arrived by a need), gallery (ui-lab) |
| `PaliTerm` | A Pāli term on two lines, generic: the `lemma` in bold (a `Pali` at body size) with an optional `tag` after it in parentheses (small, muted, never truncated: a long lemma truncates first), the `gloss` under it on one truncated line (`lemma`, `gloss` and `tag` are strings or signals, so a translation updates in place), and an optional end slot at the right of each line (`top_end`, `bottom_end`: small, muted, flush right, never wrapping; the slot runs once, so a reactive end holds a `move ||` inside; absent, nothing is rendered and line 2 still starts in column 1); a `w-full` two-column grid (it fills a centring parent such as a `Tile` button) whose left column shrinks, so the lemma and the gloss truncate first on a phone; not interactive, the caller wraps it in a button or a link; no PaliCanon data | learn-pali (progress rows, task 003), gallery (ui-lab) |
| `PaliText`, `WordEntry`, `words`, `Token` | A Pāli passage whose words open their meaning, generic (no dictionary): the caller gives `entries`, a map (or a signal of one) from each word as written, lower-cased, to its `WordEntry`s (headword, optional tag, meaning); a word with entries is an inline `<button>` (`aria-expanded`, `aria-controls` naming the passage's panel `id`; no mark at rest, a dotted gold underline when hovered, focused or open), any other word, punctuation and numbers stay plain; pressing a word opens a `Popover` under it from `sm` (flipped above when it would overflow), a bottom sheet over a dimmed backdrop with a close button on a phone: the form in bold, at most three entries then `+ n more` (`more_label` translates it); one word open at a time, Escape, an outside click or a tap on the backdrop closes it; `words` is the tokeniser (letters with Pāli diacritics, precomposed or combining, and the sandhi apostrophe between letters: `sampādethā'ti` is one word) | gallery (ui-lab); learn-pali (task 026, planned) |
| `RevealCard` | A card with a `front`, a `reveal` shown in place (focus moves into it, no live region) and the caller's `actions` pinned at the bottom above the safe-area inset; Space and Enter reveal it while unrevealed; the reveal is derived from a `key` that is unique per presentation, so a new key reads unrevealed in the same tick | learn-pali (arrived by a need), gallery (ui-lab) |
| `BucketStrip`, `Bucket` | n labelled counts in a row (three columns on a phone, one flex row from `sm`), under an optional heading; a cell with `to` is a real `<a>` on the `Tile` variant, without it a plain box; no cells, nothing rendered; `size` (Size::Md by default) sets the cells' size — Size::Sm is compact (smaller count and label); with `on_select` (a `Callback<usize>`) every cell is a `<button aria-pressed>` instead, the `selected` one (`Signal<Option<usize>>`) pressed, for filtering a list in place | learn-pali (arrived by a need), gallery (ui-lab) |
| `Slider` | A range slider over `[min, max]` that reads and writes an `RwSignal<u32>`, its value shown at the right (`show_value`, on by default; `tabular-nums`): a native `<input type="range">` (keyboard, touch and screen readers for free) named by `label` (`aria-label`), `step` 1 by default, `max` a signal; the value is always held inside the range, also when `max` shrinks below it; disabled when `max <= min`; `accent-gold` paints the filled part and the thumb with the theme's gold, the focus ring is the buttons'; the track fills the width it is given and the thumb is 24px tall on a phone | gallery (ui-lab), learn-pali (task 009, the Practice batch) |
| `Carousel` | A row of slides, one or more in view, scrolled by swipe, the arrow keys (the scroller is the focusable labelled region) or two Previous / Next buttons: native scroll-snap, so it works with no JavaScript; the component wraps each child in a slide (`role="group"`, named "n of N" in the current language) and knows how many there are, none rendering nothing; a slide's phone width is the caller's `phone_layout` (85% by default, the next peeks) and its `slide_layout` width from `sm`; the buttons, below the scroller, are invisible until hydrated and while nothing overflows, and a press scrolls to the next slide edge and announces "n of N" politely; smooth scrolling is `pc-carousel` in `theme.css`, off under reduced motion; no autoplay, no loop, no visible position. The slides are the children, one expression each, or one list as the only child; `id` is the caller's and unique on the page | gallery (ui-lab), search-front (task 010, planned) |
| `SessionHeader` | A counter (`3 / 10 cards`, no live region) and a 44 px close button; it holds no dialog, the caller composes the confirmation | learn-pali (arrived by a need), gallery (ui-lab) |

## Use it

```toml
palicanon-ui-lib = { git = "ssh://git@github.com/PaliCanon/ui-lib.git", branch = "main" }
# features: "ssr" on the server, "hydrate" in the browser (wasm32)
```

Besides the crate, a consumer reads three things from the repository:

- `style/theme.css` — the palettes, the tokens and the `pc-motion-*`
  classes; a consumer's Tailwind input `@import`s it, and `@source`s
  `src/` so the classes the components use are generated.
- `build/placeholders.rs` — the placeholder-parity check, which a
  consumer's own `build.rs` includes with `#[path]` for its catalogue.
- `locales/<code>.json` — the library's own strings (en, fr, vi).

Today `ui-lab` reaches them through a sibling checkout (`../ui-lib`); a
service consuming the library by git only will need them shipped
another way (`ui-lab`'s task 032, "Forks for later").

## Checks

```sh
# Toolchain with the wasm32 target, isolated from the system Rust:
export RUSTUP_HOME=$HOME/.rustup-pc/rustup CARGO_HOME=$HOME/.rustup-pc/cargo PATH=$HOME/.rustup-pc/cargo/bin:$PATH RUSTC_WRAPPER=sccache
cargo fmt --check
leptosfmt --check .    # the view! macros (cargo install leptosfmt --locked)
cargo clippy --features ssr --all-targets -- -D warnings
cargo clippy --features hydrate --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test --features ssr
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features ssr
```

The same run in CI (`.github/workflows/ci.yml`). Functions stay under 50
lines (`clippy.toml`).
