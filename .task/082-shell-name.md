# 082 AppShell's name — the wordmark and the navigation's accessible name

status: review · rank: 2 · level: simple · type: feat · request: anatta-console 096 · light path · number: working, checked against main at merge

## Summary
AppShell wrote "PaliCanon" as its wordmark and as both navigations' accessible name: every service in the shell said PaliCanon, anatta-studio included. An optional `name` (default "PaliCanon", so every service is unchanged), provided as a context the wordmark and the two `aria-label`s read. Rank 2: difficulty 1, impact 1.

## Log

| Step | Result |
| --- | --- |
| Build | `AppShell` `name`; `AppName` context; `app_name()` ("PaliCanon" outside a shell). Test: the default is PaliCanon; given "anatta", the wordmark and the rail's name say anatta and nothing says PaliCanon. 562 tests, clippy four targets, rustdoc, both formatters clean |
| Review | deep #15090: one default in two literals (now `DEFAULT_APP_NAME`, exported), a blank name left the navigation nameless (it falls back now; tested), the phone menu's name asserted positively; `&'static str` and an unlocalised wordmark are deliberate (a service's own brand, like the logo URL). Fixed in the second commit; 563 tests |

## Merge
