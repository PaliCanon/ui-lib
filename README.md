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
| `Button`, `DisclosureButton`, `ToggleButton`, `LinkButton`, `RadioButton` | The button family: appearance is a `Variant` (Icon, Chip, Primary, Accent, Ghost, Nav) and a `Size` (Sm, Md, Lg); behaviour is the component (click, open/close something, on/off, navigate, one choice of a group) | `AppShell`, gallery (ui-lab) |
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
cargo clippy --features ssr --all-targets -- -D warnings
cargo clippy --features hydrate --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test --features ssr
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features ssr
```

The same run in CI (`.github/workflows/ci.yml`). Functions stay under 50
lines (`clippy.toml`).
