# 074 Pāli text one size bigger; a Plus icon

status: review · rank: 1 · level: simple · type: feat · request: learn-pali, the user 2026-10-22 («ce serait bien d'avoir le pali en un peu plus gros»; «centrer un peu mieux la croix» on the + button) · light path

## Summary
Rank: difficulty 1 (three class strings, one icon), impact 2 (every Pāli text in Learn; search-front takes it at its next bump) → 1.
`PaliSize` grows one step each: Body `text-[19px] leading-8` (was 17px/7), Lemma `text-3xl` (was 2xl), Display `text-5xl` (was 4xl); the docs and tests follow. `IconName::Plus` (two strokes, centred in the 24 box) so a + button holds a drawn icon, not a glyph whose baseline sits off centre. Additive: no service breaks. Released as v0.4.1.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | 6de6fe2 d5fb1ee ; 493 ssr tests + fixtures, clippy ssr and hydrate 0, fmt 0 (orchestrator, no agents) |
| Check | done | orchestrator: the sizes and the icon are seen on learn.dev with learn-pali 063b (the user asked for both there) |
