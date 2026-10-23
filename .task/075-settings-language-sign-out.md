# 075 The rail's bottom: the language in Settings, a sign-out under it

status: merged · rank: 2 · level: simple · type: feat · request: the user, 2026-10-22 (anatta-studio, looking at its rail)

## Summary
The user, on anatta-studio's rail: "mets les langues dans settings", "mets le signout dessous les settings". anatta-studio has accounts
(search and learn-pali do not), so it needs a sign-out; it also wants the language choice inside the Settings dialog rather than as its own
entry. Additive and opt-in, so every service builds and renders unchanged:
- `AppShell` `language_in_settings: bool` (default false): the globe entry of the rail and the language row of the phone menu are not
  rendered; the Settings dialog gets a Language section after Appearance, `LanguagePicker` (a `Segmented` of `Language::ALL` in their own
  names, bound to `language_signal`, so a choice applies at once and is kept in `pc_lang`).
- `AppShell` `sign_out: Option<&'static str>` (none by default): under the Settings entry, at the rail's bottom and in the phone menu, an
  entry that opens a CONFIRMATION (the user: "sign out devrait avoir un pop up de confirmation") — a `Popup` dialog, "Sign out?", Cancel and
  the one `<form method="post">` to the app's path with the new `IconName::SignOut` and the label `sign_out` (en "Sign out", fr "Se déconnecter",
  vi "Đăng xuất"), fading with the rail like the entries above it. The app's route keeps its own Origin check.

## Log
| Step | Result |
| --- | --- |
| Build | shell.rs (ShellOptions context, RailSignOut, MobileSignOut, LanguagePicker), icon.rs (SignOut), locales ×3, lib.rs export; tests `sign_out_is_a_post_form_under_settings_in_the_rail_and_the_menu`, `language_in_settings_moves_the_choice_into_the_dialog` |
| Verify | fmt, leptosfmt 0; clippy ssr, hydrate, csr, browser 0; cargo test --features ssr: 507 passed |
| Check (in anatta-studio, headless Chrome over DevTools) | rail opened: Settings then Sign out at the bottom; Settings: Appearance then Language — but no language looked chosen: the Chip variant styled aria-pressed and aria-expanded, never aria-checked, the state Segmented's radios carry. Fixed in the same branch (style.rs, test `a_checked_chip_is_drawn_as_chosen`, red before); recaptured: English drawn chosen |
| Build 2 | the confirmation (SignOutDialog, `sign_out_open` in ShellState, Escape closes it first), the Language choice centred like the theme cards (the user); strings sign_out_title, sign_out_body, cancel ×3; tests `sign_out_opens_a_confirmation_holding_the_one_post_form`, `escape_closes_the_sign_out_confirmation_only`; 497 lib tests, checks 0; captured in anatta-studio: Settings (Language centred, English chosen), the confirmation |

## Retro
Light path by the orchestrator alone (no agents, the user's order). Two things the browser caught that the HTML tests did not: Segmented's chosen option was never drawn (aria-checked unstyled by the Chip variant), and the language choice sat left under centred theme cards. Backward compatibility proved by compiling learn-pali and search-front (tests included) against the branch. The user asked for the sign-out confirmation after seeing the first build.

## Merge
4374e67, released v0.4.2
