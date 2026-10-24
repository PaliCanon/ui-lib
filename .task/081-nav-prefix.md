# 081 A prefix for NavList's hrefs — a console's /t/{tenant}

status: review · rank: 2 · level: simple · type: feat · request: anatta-console 096 · light path · number: working, checked against main at merge

## Summary
anatta-console's SPA (096) keeps the tenant in its URLs (`/t/{tenant}/members`), so two tabs can hold two tenants. `NavEntry.href` is `&'static str` and anatta-studio builds `NavEntry` literally, so the type cannot change; a per-tenant string would leak once per render on the server. Additive instead: `NavList` gains a live `prefix` (empty by default) and `with_prefix`; each entry renders `prefix + href` and is current when the path equals the whole. No service builds `NavList` literally (grepped: every one calls `NavList::new`). Rank 2: difficulty 1, impact 1.

## Log

| Step | Result |
| --- | --- |
| Build | `NavList.prefix` + `with_prefix`; an entry rebuilt when the prefix changes (a tenant switched in the URL). Tests: a prefixed rail links `/t/42/…` and marks the current entry on the whole path; without a prefix the hrefs are as written. README: the NavList row, and AppShell's rail_top corrected (kept mounted since 080). 560 tests, clippy four targets, rustdoc, both formatters clean |
| Review | deep #15086: `current` must include the prefix (now said on with_prefix), no other reader of href/sections (grepped: footer.rs reads FooterLink's), a trailing slash would double (joined(): one slash), the no-prefix path paid a closure (prefix is now an Option: None renders as before), give the prefix synchronously at SSR (said). Fixed in the second commit |

## Merge
