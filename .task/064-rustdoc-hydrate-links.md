# 064 rustdoc fails under the hydrate feature

status: design · rank: 1 · level: simple · type: bug · light path · request: found during 063 (2026-10-15)

## Summary
Rank: difficulty 1 (two intra-doc links), impact 1 (CI documents `ssr` only, so nothing is red today) → 1.
`RUSTDOCFLAGS=-D warnings cargo doc --no-deps --features hydrate --target wasm32-unknown-unknown` fails: `src/language.rs:3` and `:17` link to `resolve_language`, an item that exists only under `ssr`. Fix the links (a `cfg_attr(doc …)` or plain text) and add the hydrate rustdoc to CI beside the ssr one.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Found | done | 063's lane, 2026-10-15 |
