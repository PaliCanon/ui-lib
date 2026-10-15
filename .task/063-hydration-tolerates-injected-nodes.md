# 063 Hydration tolerates the nodes Cloudflare injects

status: building · rank: 8 · level: hard · type: bug · request: palisearch 048 (user, 2026-10-14: "Garder les deux, corriger le code") · session: ui-lib-063-hydration-tolerant · process: ../../docs/workflows/process-bug.mmd · after: []

## Summary
Rank: difficulty 5 (the hydration entry of every site, a DOM pre-pass before `hydrate_body`, a browser-level regression test, two consumers re-pinned), impact 8 (production search and learn dev are dead in a browser until it ships) → 7. Level hard: production, every Leptos site.
Cause (palisearch 048, learn-pali 039): Cloudflare injects into the served HTML a hidden `<a href="…/cdn-cgi/content?id=…">` as the FIRST child of `<body>` (an AI-crawler decoy link), plus the Web Analytics beacon `<script>` and, on challenges, a `/cdn-cgi/challenge-platform/` script. Each site calls `leptos::mount::hydrate_body(App)` (search-front app/src/lib.rs:38, learn-pali app/src/lib.rs:46): tachys walks `<body>`'s children in render order, meets the `<a>` where it expects its first node, and panics (`tachys-0.2.19/src/hydration.rs:227`, `failed_to_cast_text_node` → `unreachable!()` in release). The user keeps both Cloudflare features: the code must tolerate them.
Design to settle (consensus): a `palicanon_ui_lib::hydrate(App)` entry that, before `hydrate_body`, removes from `<body>` (and `<head>`) every node the server did not render: `a[href*="/cdn-cgi/"]`, `script[src*="cloudflareinsights.com"]`, `script[src*="/cdn-cgi/"]`, and any `script` without a `nonce`/`data-cfasync` the shell knows — a list, not a heuristic, documented; the two sites call it instead of `hydrate_body` (a bump in each, PRs as ui-lib.md asks); a regression test at the browser level (a wasm test, or the sites' rollout-check with headless Chrome asserting no panic in the console on the Worker host) since SSR tests cannot see it; the ui-lab gallery unaffected (not behind Cloudflare). Open: whether removing the decoy link breaks Cloudflare's crawler trap for real crawlers (it does not: crawlers read the HTML, not the hydrated DOM).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Reproduce | done | orchestrator, 2026-10-14, headless Chrome CDP 390×844 on https://search.palicanon.app/?q=anger: `panicked at tachys-0.2.19/src/hydration.rs:227:9`, uncaught wasm `unreachable`; the decoy `<a>` is `<body>`'s first child in the served HTML |
| Probe | done | orchestrator, 2026-10-14, CDP Fetch rewrite of the HTML response of https://search.palicanon.app/?q=anger: as served (decoy link + beacon): 1 panic; with the decoy `<a>` and the two Cloudflare scripts stripped before parsing: 0 panic, hydration completes. Cause confirmed. |
| Email Obfuscation | ruled | user, 2026-10-15: "La garder" — the zone setting stays on; the limit stands in the design (no hydration fix tolerates an address rewritten inside the root); T3 visits /privacy and fails on a panic. |
| Consensus | handed back | design v4 (root element `pc-app` + `palicanon_ui_lib::hydrate` = `hydrate_from(root)`, no node removed: only the leading decoy breaks hydration, probe decoy-last 0 panic); T1 wasm-bindgen-test in ui-lib CI, T2 missing root named, T3 headless-Chrome step in each site's rollout-check (/, /?q=, /privacy); rollout ui-lib v0.1.2 → PRs search-front and learn-pali → dev checks → tags v0.1.1 each (learn.palicanon.app is broken too). Voices: deep every round; gemini 503/ECONNRESET/429, sage 504 twice; simplify unanswered. Follow-up d0a3da22bb4254c (hashed pkg names, rank 4). Re-rank 7 → 8: a new wasm test harness in ui-lib CI, two production rollouts. User validation of v4 pending. |
| Build | done (T1 deviates, to rule) | lane:hydration (opus), 2026-10-15, branch fix/hydration-root: db87047 feat(hydration) (APP_ROOT_ID, HydrationRoot, MissingRoot, app_root, hydrate; tests/hydration.rs; README row), 348b68d ci (job test-browser: taiki-e/install-action wasm-bindgen@<Cargo.lock version> + ubuntu-latest's $CHROMEWEBDRIVER; not run on a runner yet). Reds: E0425 (native), E0432 (wasm), T1 with hydrate_body: `panicked at tachys-0.2.19/src/hydration.rs:184:9: Unrecoverable hydration error` (expected <button>, found Text). T1's markup is the ssr render in tests/fixtures/hydration-counter.html, checked by a native test (to_html of a component panics without ssr: tachys any_attribute.rs:332). Checks: fmt 0, leptosfmt 0, clippy ssr 0, clippy hydrate wasm32 0, test ssr 0 (lib 423 passed, tests/hydration 1 passed), wasm test 0 (3 passed; wasm-bindgen-test-runner 0.2.129, chromedriver 155.0.8059.40, Chrome 155.0.8059.40), rustdoc ssr 0; rustdoc hydrate wasm32 101 on pre-existing src/language.rs:3,17. Process #6501 |
| Close | done | lane:hydration, process #6503 (seven errors, @gemini:role-process) |

## Design
Viewer: ui-lib-063-hydration-tolerant/design v5 (v4 validated by the user 2026-10-15; v5 rewords T1 after the build, orchestrator ruling); consensus summary discussion #6487.
```mermaid
%% 063 Hydration tolerates the nodes Cloudflare injects — design v4
flowchart TB
  classDef cause fill:#fdecea,stroke:#c0392b,color:#5a1a12
  classDef lib fill:#eef0ff,stroke:#8b8fb5,color:#1a1a2e
  classDef site fill:#fff3cf,stroke:#c98a00,color:#3a2a00
  classDef test fill:#e8f6ee,stroke:#2e8b57,color:#0f3d24
  classDef fail fill:#fdecea,stroke:#c0392b,color:#5a1a12,stroke-width:2px
  classDef out fill:#f4f4f4,stroke:#999,color:#333,stroke-dasharray: 4 3

  subgraph CAUSE["Cause: the HTML served through Cloudflare (curl + CDP probe, 2026-10-14)"]
    direction TB
    BODY["body children, in order:<br/>1. a href=/cdn-cgi/content?id=… hidden (AI-crawler decoy, injected FIRST)<br/>2. the app: div (SettingsDialog), div (shell)<br/>3. inline challenge script (no src) + the iframe it appends<br/>4. script src=static.cloudflareinsights.com/beacon.min.js (Web Analytics)<br/>5. Leptos's own nonce scripts (__RESOLVED_RESOURCES…, after /html)"]:::cause
    WALK["hydrate_body(App): tachys Cursor starts at body, child() lands on node 1,<br/>expects the app's first node, cast fails → panic hydration.rs:227 (release: unreachable)"]:::cause
    PROBE["Probe (CDP Fetch rewrite of ?q=anger):<br/>as served: 1 panic · decoy removed, scripts kept: 0 · decoy MOVED after the app: 0<br/>→ only a node BEFORE or INSIDE the app breaks hydration; trailing nodes are never visited"]:::cause
    BODY --> WALK --> PROBE
  end

  REQ["Requirement: hydration survives ANY node added outside the app's own markup,<br/>before or after it, whoever adds it (Cloudflare today, another zone feature or a browser extension tomorrow),<br/>with no list to keep in step and nothing removed from or moved in the DOM"]:::lib
  PROBE --> REQ

  subgraph LIB["ui-lib v0.1.2 — additive (new items only), new module src/hydration.rs"]
    direction TB
    ROOTID["pub const APP_ROOT_ID = pc-app — every build<br/>the one name the server writes and the browser reads"]:::lib
    HROOT["pub component HydrationRoot(children) — every build, no cfg<br/>(the sites' shell() compiles in the hydrate build too; only the server calls it)<br/>renders div id=APP_ROOT_ID {children}; no class, no style: body is block flow, the layout is unchanged"]:::lib
    APPROOT["fn app_root(doc) → Result of HtmlElement or MissingRoot — feature hydrate<br/>MissingRoot's Display: no element #pc-app: wrap App in HydrationRoot in the server shell"]:::lib
    HYD["pub fn hydrate(f) — feature hydrate<br/>root = app_root(document) — Err → panic with its Display<br/>Ok → leptos::mount::hydrate_from(root, f).forget()<br/>then sets html data-hydrated (the marker T3 reads)"]:::lib
    EXPORTS["lib.rs: pub use hydration::{APP_ROOT_ID, HydrationRoot};<br/>cfg hydrate: pub use hydration::hydrate; web-sys features already present (Document, Element, HtmlElement)"]:::lib
    MISSING["root missing = the shell is not wired: a hard error, no fallback<br/>(fails on localhost too, so a mis-wired site never reaches a deploy)"]:::fail
    INPANIC["a node INSIDE the root that the server did not render<br/>(Email Obfuscation rewriting an address): the same tachys panic, no marker;<br/>out of reach of any hydration fix → zone question to the user, guarded by T3 on /privacy"]:::fail
    ROOTID --> HROOT
    ROOTID --> APPROOT --> HYD
    HYD --> EXPORTS
    HROOT --> EXPORTS
    ROOTID --> HROOT
    ROOTID --> HYD
    APPROOT -->|"Err MissingRoot"| MISSING
    HYD -->|"foreign node inside the root"| INPANIC
  end
  REQ --> ROOTID

  SKEW["Release boundary: the HTML and the wasm come from one image (one revision).<br/>Only skew: a browser's heuristic cache of the unhashed /pkg/*.wasm (no Cache-Control),<br/>pre-existing for every deploy → followup-pkg-cache (hashed pkg names), not this task"]:::out
  HYD -.-> SKEW

  subgraph NOT["Ruled out"]
    direction TB
    STRIP["A pre-pass REMOVING nodes before hydrate_body (a cdn-cgi / beacon list):<br/>the decoy leaves the live DOM (weaker trap for crawlers that run JS), a list behind Cloudflare,<br/>a nonce-less-script rule would hit Leptos's own scripts"]:::out
    MOVE["A pre-pass MOVING the leading body > a[href*=/cdn-cgi/] to the end (0 panic in the probe):<br/>no shell change, but still a list: the next leading node (another feature, an extension) panics again"]:::out
    LENIENT["A tolerant mode in Leptos 0.8: none exists (hydrate_body, hydrate_from, hydrate_lazy, hydrate_islands)"]:::out
    ZONE["Turning Web Analytics or the decoy links off: ruled out by the user (keep both, fix the code)"]:::out
  end
  REQ -.-> NOT

  subgraph SITES["Each site behind Cloudflare: search-front, learn-pali (one PR each, the bump commit)"]
    direction TB
    BUMP["Cargo.toml: palicanon-ui-lib tag v0.1.2 (Cargo.lock --locked)"]:::site
    SHELL["app.rs shell(): body > HydrationRoot > App<br/>(the decoy, the scripts stay body children OUTSIDE the root)"]:::site
    ENTRY["lib.rs hydrate(): palicanon_ui_lib::hydrate(app::App) instead of hydrate_body<br/>SHELL and ENTRY change in the same commit (the id written and the id read)"]:::site
    BUMP --> SHELL --> ENTRY
  end
  GALLERY["ui-lab gallery: unchanged (not behind Cloudflare, stays on hydrate_body); may adopt later"]:::out
  HROOT --> SHELL
  HYD --> ENTRY

  subgraph TESTS["Regression tests"]
    direction TB
    T1["T1 ui-lib tests/hydration.rs, wasm-bindgen-test, --features hydrate, wasm32, headless Chrome<br/>(wasm-pack test --headless --chrome, or wasm-bindgen-test-runner + chromedriver; both on ubuntu-latest) — a new CI job<br/>a test component Counter in the test file; its server markup lives in tests/fixtures/hydration-counter.html, pinned by a native test asserting the fixture == view!{<HydrationRoot><Counter/></HydrationRoot>}.to_html() under ssr (UPDATE_FIXTURES=1 rewrites it; rendering to HTML inside the hydrate build is impossible: tachys panics without ssr, and ssr drops the event handlers); the wasm test inserts decoy + fixture + beacon at the start of body (insert_adjacent_html afterbegin, never body.innerHTML: it would wipe the runner's own elements), calls hydrate(Counter), clicks, asserts the counter updated, the decoy still first, html[data-hydrated] set"]:::test
    T2["T2 ui-lib wasm test: a document with no #pc-app → app_root is Err(MissingRoot), its Display names pc-app and HydrationRoot (no should_panic needed)"]:::test
    T3["T3 check hydrate in each site's scripts/rollout-check.sh --via-worker only (Chrome cannot add the origin header),<br/>after the existing checks (a challenge has already exited 3); lands in the SAME PR as SHELL + ENTRY; runs in deploy.yml<br/>on the public host once traffic moved, exit 1 → rollback (the existing path). Driver: Chrome itself, no CDP script:<br/>google-chrome --headless=new --enable-logging=stderr --virtual-time-budget=15000 --dump-dom URL under timeout 60<br/>(proved by hand on prod 2026-10-14: stderr carries panicked at), on /, /?q=satipatthana, /privacy<br/>its red: run by hand on today's prod before the fix (fails: 1 panic)"]:::test
    T3OK["dumped DOM has html data-hydrated, stderr has no panicked at / RuntimeError → pass"]:::test
    T3PANIC["stderr has panicked at or RuntimeError → exit 1 → rollback"]:::fail
    T3TIME["no data-hydrated in the dumped DOM, or Chrome killed by timeout 60 → exit 1 → rollback"]:::fail
    T3SKIP["Cloudflare challenge (exit 3, R6) or a host behind Access → SKIP as a ::warning::, never green;<br/>the orchestrator then runs the same check by hand from its machine before any prod tag"]:::out
    T3 --> T3OK
    T3 --> T3PANIC
    T3 --> T3TIME
    T3 --> T3SKIP
  end
  LIB --> T1
  LIB --> T2
  SITES --> T3

  subgraph ROLL["Rollout — each site's prod tag waits on ITS OWN dev check"]
    direction TB
    R1["ui-lib: merge 063, release-plz tag v0.1.2"]:::lib
    R2["search-front PR: bump + shell + entry + check hydrate → main → dev deploy, T3 on search.dev"]:::site
    R3["search-front tag v0.1.1 → prod, T3 on search.palicanon.app (the bug's host)"]:::site
    R4["learn-pali PR: bump + shell + entry + check hydrate → main → dev deploy, T3 on learn.dev (not behind Access today)"]:::site
    R5["learn-pali tag v0.1.1 → prod, T3 on learn.palicanon.app (broken too: probe 2026-10-14, 1 panic)"]:::site
    RSTOP["T3 red on a site's dev → no prod tag for that site; back to the task"]:::fail
    RHAND["T3 SKIP on a site's dev → the orchestrator's manual run: green → the tag goes on; red → RSTOP"]:::out
    R1 --> R2
    R1 --> R4
    R2 -->|"green"| R3
    R4 -->|"green"| R5
    R2 -->|"red"| RSTOP
    R4 -->|"red"| RSTOP
    R2 -->|"skip"| RHAND
    R4 -->|"skip"| RHAND
    RHAND -->|"manual green"| R3
    RHAND -->|"manual green"| R5
    RHAND -->|"manual red"| RSTOP
  end
  TESTS --> ROLL```

| Validation | done | user, 2026-10-15: "Validé" (v4) |

## Lanes
| File | Tests | Code | Done |
| --- | --- | --- | --- |
| src/hydration.rs, src/lib.rs, Cargo.toml, README.md | src/hydration.rs tests (root id, bare div, MissingRoot message); red E0425 | APP_ROOT_ID, HydrationRoot, MissingRoot, app_root, hydrate | db87047 |
| tests/hydration.rs, tests/fixtures/hydration-counter.html | T1 hydrates past the decoy and beacon, T2 missing root named, root found by id, fixture = ssr render; red E0432, then the hydrate_body panic | (the tests) | db87047 |
| .github/workflows/ci.yml | job test-browser (headless Chrome) | — | 348b68d, unproven on a runner |
| Build ruling (T1) | ruled | the design's `Owner::new().with(view.to_html())` inside the hydrate build is impossible (tachys panics without ssr; ssr drops the handlers): the markup is a fixture pinned by a native test equal to the view's `to_html()` under ssr — accepted, design v5 (node T1). Trailer `Claude Opus 5.5` on the commits: left as is. Pre-existing: rustdoc with `--features hydrate` fails on `src/language.rs` links → task 064. CI job `test-browser` unproven on a runner: the branch is pushed as a PR to prove it. |
