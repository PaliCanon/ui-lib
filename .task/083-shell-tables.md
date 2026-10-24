# 083 The rail open by default, and tables as cards on a phone

status: review · rank: 2 · level: simple · type: feat · request: anatta-console 096 · light path · number: working, checked against main at merge

## Summary
Two defects the user saw on the console (2026-10-24). On a phone a DataTable scrolled sideways inside its box: the email and the actions cut off. And the desktop rail always started collapsed, which hides the tenant switcher the console puts under the logo. Decided: below `sm` every row stacks as a card — each cell a line with its column's header beside it (`data-label`, shown by a `before:` pseudo-element), the actions last, the head row gone — in CSS only, so the server render and hydration stay one markup; a `hide_on_phone` cell gets no card styles (its `max-sm:hidden` would fight `max-sm:flex`). AppShell gains `rail_open` (off by default): the rail's state on a first visit, the person's remembered choice (`pc_rail`) winning once made. Every service renders as before unless it opts in. Rank 2: difficulty 1, impact 1.

## Log

| Step | Result |
| --- | --- |
| Build | `Column::classes` card lines below sm, `row_cells` (data-label), `thead` hidden below sm, the box `min-w-0`; `AppShell` `rail_open` into `shell_state`. Tests: below_sm_each_row_stacks_with_its_labels, rail_open_is_the_default_and_a_remembered_choice_wins (four rows: none/true/false cookies). 565 tests, clippy four targets, rustdoc, both formatters clean. Looked at through the console (headless Chrome, 390 and 1360 wide): cards readable on a phone, the rail open with the tenant on desktop |
| Review | deep #15185 (one round): FIX 2 the stacked table kept for screen readers (thead `max-sm:sr-only`, explicit table/rowgroup/row/columnheader/cell roles), FIX 4 the negative assertion replaced by positive ones on the hidden cell and the actions cell, FIX 5 no empty `data-label` on an actions cell. STRUCK 1: `persisted` parses a stored value before falling back to the default — a stored `false` wins (the test's pc_rail=false row). STRUCK 3: the truncated line is outside the table, the empty row a single block line. 6: `rail_open` is off by default on purpose (every service unchanged); the console opts in. 565 tests |

## Merge
