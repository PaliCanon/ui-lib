# 077 A Users icon — a team, a workspace's members

status: built — awaiting merge and release v0.4.4 · rank: 1 · level: simple · type: feat · request: the user, 2026-10-23 (anatta-studio's rail) · light path

## Summary
anatta-studio's rail drew its Workspace entry with `IconName::Menu` (☰), for want of a better icon; with the rail closed, the user read it
as a second "open the menu" button under the logo, which is the real toggle. The user chose a "team" icon in ui-lib over a clickable
workspace name. Additive: `IconName::Users`, two people, one behind the other, on the 24 × 24 grid and the stroke of the others. No
service changes until it opts in; IconName is not matched exhaustively anywhere outside the library.

## Log

| Step | Result |
| --- | --- |
| Build | IconName::Users, its paths, the every-icon test; v0.4.4 |
