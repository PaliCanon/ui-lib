# 076 PaliText: every word tappable with a lookup

status: merged · rank: 2 · level: simple · type: feat · request: learn-pali 065 (the user, 2026-10-23: «pourquoi je ne peux pas avoir la définition des mots du sutta ?», «mot tapable partout») · light path

## Summary
Rank: difficulty 2 (two optional props, one branch in the segmenting, one line in the panel), impact 3 (every passage of Learn) → 2.
Additive: `PaliText` takes `lookup: Option<Callback<String>>` (every word becomes a button; opening a word with no entry calls it with the form, once per opening, so the caller fetches and fills the `entries` signal) and `empty_label: Option<Signal<String>>` (what the panel of a word with no entry says, read live). Without them nothing changes. Released as v0.4.3.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | b5775c1; 496 ssr tests (3 new), clippy ssr and hydrate 0, fmt 0, doc 0 (orchestrator, no agents) |
| Check | done | orchestrator: consumed by learn-pali 065 through the local patch; seen on learn.dev with it |
| Merge | done | 2d95230 on main (squash); v0.4.3 at the bump commit |

## Retro
Light path by the orchestrator alone. One lesson: an optional view (`Option`) in a component's markup leaves a `<!>` marker in SSR and breaks a golden fixture; a plain `Vec` of rows does not. Retro 076 validated.

## Merge
2d95230
