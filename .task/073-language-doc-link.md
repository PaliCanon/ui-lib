# 073 language doc link

status: building · rank: 1 · level: simple · type: docs · request: found by lane 072 (2026-10-22): a broken intra-doc link in src/language.rs under non-ssr feature sets · light path

## Summary
Difficulty 1, impact 1: the `resolve_language` function is cfg-gated to `#[cfg(feature = "ssr")]` only, but the module-level documentation references it without qualification, causing the intra-doc link to break when compiled with other feature sets (csr, hydrate). Fix by using fully qualified paths in the links.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | Red: exit 101 on `--features csr`: unresolved link to `resolve_language` on lines 3 and 17 (module docs); Green: 6c79bf5 (fix merged); exit 0 on `--features csr`, `--features hydrate`, `--features ssr`; 505 tests; fmt clean |

| Check | done | orchestrator: a doc-only change (three links to a cfg-gated item written as plain code); doc exit 0 under csr, hydrate and ssr |
