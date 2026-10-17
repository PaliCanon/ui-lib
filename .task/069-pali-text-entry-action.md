# 069 PaliText's popover takes an action per entry

status: merged · rank: 3 · level: standard · type: feature · request: learn-pali 054 (user, 2026-10-17: add a passage's word to the practice from its popover) · session: learn-pali-054-add-word (the design is 054's v3, its `slot` node) · additive (a minor release; the services build unchanged)

## Summary
Rank: difficulty 3 (an optional prop on PaliText rendered under each entry, desktop popover and phone bottom sheet, a gallery section), impact 3 (every service using PaliText; additive) → 3.
PaliText's word popover lists the DPD entries of the pressed form. learn-pali 054 needs a button under an entry ("Add to practice") and an "Added" line after. PaliText gains an optional prop `entry_action: Callback<(String /* the form pressed */, WordEntry), AnyView>` (or the closest Leptos 0.8 shape: a `Callback` or `ViewFn`), rendered under each entry inside the popover and the phone's bottom sheet; absent = today's look exactly. ui-lib owns no string and no rule: the service decides what it shows. A gallery section shows it with a button and with an "Added" line. Released as ui-lib v0.3.1 (tag), learn-pali bumps in its own commit.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | design learn-pali 054 v3 (consensus deep and sage, 2026-10-17), the `slot` node; validated by the user's answers |
| Build | done | b89f008: `entry_action` (type `EntryAction`) in src/pali_text.rs, 6 new SSR tests + 3 golden fixtures (markup without the prop unchanged), version 0.3.1, README. Red: the tests did not compile (no `EntryAction`, no `Ctx.entry_action`, no prop). No browser test exists for PaliText. |
| Verify (lane) | done | fmt, leptosfmt, clippy ssr/hydrate/csr/browser, `cargo test --features ssr --locked` (451 lib tests), hydration and csr wasm tests (chromedriver 155 / Chrome 155), doc: all exit 0. Gallery section: to do in ui-lab (gallery/src/cards.rs), not in this branch. |
| Review fix 1 | done | 4dad070: Z1 `new_ctx` + test through it (mutation: `entry_action: None` in new_ctx fails 2 tests), Z2 recorded calls (mutation: headword as form fails 2), Z4 sheet scroll assertion (mutation: drop `max-sm:overflow-y-auto` fails 2), Z3 not testable in SSR (one render, no reactive re-run): documented on the prop instead. 455 lib tests; fmt, leptosfmt, clippy x4, doc, hydration and csr wasm: exit 0. |
| Review 1 | done | reviewer d820c0bc359a14e; deep 11552, gemini 11571 (sage thin): Z1 the prop through the component's wiring, Z2 calls per visible entry and the cap, Z4 the phone sheet's scroll taken in (4dad070); Z3 skipped (SSR renders once; the doc says how a reactive slot reads its signal); the gallery done in ui-lab 6551045 |
| Check | done | orchestrator, 2026-10-17: no visible change without the prop (golden fixtures); the gallery example checked by its SSR test; all checks exit 0 (455 lib tests) |
| Merge | done | 522011c on main (squash), tag v0.3.1; gallery merged in ui-lab |

## Retro
One line (standard, no retro agent run: the user asked for speed): a slot whose path through the component cannot be opened in SSR needs a seam (new_ctx) to be tested at all; a test that builds the context by hand proves nothing about the wiring.
