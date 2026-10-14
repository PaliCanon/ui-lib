# 063 Hydration tolerates the nodes Cloudflare injects

status: design · rank: 7 · level: hard · type: bug · request: palisearch 048 (user, 2026-10-14: "Garder les deux, corriger le code") · session: ui-lib-063-hydration-tolerant · process: ../../docs/workflows/process-bug.mmd · after: []

## Summary
Rank: difficulty 5 (the hydration entry of every site, a DOM pre-pass before `hydrate_body`, a browser-level regression test, two consumers re-pinned), impact 8 (production search and learn dev are dead in a browser until it ships) → 7. Level hard: production, every Leptos site.
Cause (palisearch 048, learn-pali 039): Cloudflare injects into the served HTML a hidden `<a href="…/cdn-cgi/content?id=…">` as the FIRST child of `<body>` (an AI-crawler decoy link), plus the Web Analytics beacon `<script>` and, on challenges, a `/cdn-cgi/challenge-platform/` script. Each site calls `leptos::mount::hydrate_body(App)` (search-front app/src/lib.rs:38, learn-pali app/src/lib.rs:46): tachys walks `<body>`'s children in render order, meets the `<a>` where it expects its first node, and panics (`tachys-0.2.19/src/hydration.rs:227`, `failed_to_cast_text_node` → `unreachable!()` in release). The user keeps both Cloudflare features: the code must tolerate them.
Design to settle (consensus): a `palicanon_ui_lib::hydrate(App)` entry that, before `hydrate_body`, removes from `<body>` (and `<head>`) every node the server did not render: `a[href*="/cdn-cgi/"]`, `script[src*="cloudflareinsights.com"]`, `script[src*="/cdn-cgi/"]`, and any `script` without a `nonce`/`data-cfasync` the shell knows — a list, not a heuristic, documented; the two sites call it instead of `hydrate_body` (a bump in each, PRs as ui-lib.md asks); a regression test at the browser level (a wasm test, or the sites' rollout-check with headless Chrome asserting no panic in the console on the Worker host) since SSR tests cannot see it; the ui-lab gallery unaffected (not behind Cloudflare). Open: whether removing the decoy link breaks Cloudflare's crawler trap for real crawlers (it does not: crawlers read the HTML, not the hydrated DOM).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Reproduce | done | orchestrator, 2026-10-14, headless Chrome CDP 390×844 on https://search.palicanon.app/?q=anger: `panicked at tachys-0.2.19/src/hydration.rs:227:9`, uncaught wasm `unreachable`; the decoy `<a>` is `<body>`'s first child in the served HTML |
| Probe | done | orchestrator, 2026-10-14, CDP Fetch rewrite of the HTML response of https://search.palicanon.app/?q=anger: as served (decoy link + beacon): 1 panic; with the decoy `<a>` and the two Cloudflare scripts stripped before parsing: 0 panic, hydration completes. Cause confirmed. |
