# 071 EnvBadge takes owned strings; an empty sha drops the separator

status: merged · rank: 2 · level: simple · type: feat · request: learn-pali 060 (the badge becomes a deploy-time value read at runtime, so the props can no longer be `&'static str`) · light path

## Summary
Rank: difficulty 2 (two props, one format rule, tests, gallery), impact 2 (every service's footer; additive) → 2.
`EnvBadge` (src/env_badge.rs) takes `env: &'static str` and `sha: &'static str` and shows `{env} · {sha}`. Change: both props `#[prop(into)] String` (a `&'static str` caller still compiles: additive, no service PR), and when `sha` is empty the badge shows `env` alone, no separator, in the text and the `title`. Render tests for both cases; the gallery section in ui-lab shows a tag-only badge (`v0.1.8`) beside today's. Release: a patch tag (v0.3.2) after the merge.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Check | done | orchestrator: diff read (label helper, #[prop(into)] String props, separator only with a sha); CI matrix: clippy ssr 0 (lane), hydrate 0, csr 0 (orchestrator); a visible change only in the gallery, opened in ui-lab after the release |
| Merge | done | 52ed069 on main; tag v0.3.2 |

## Retro
One line (light path): a lane running only the ssr clippy misses the wasm matrix; the brief now quotes the CI matrix, the orchestrator ran hydrate and csr.
