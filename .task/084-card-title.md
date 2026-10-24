# 084 A table row's first column is its card's title on a phone

status: merged · rank: 2 · level: simple · type: feat · request: anatta-console 096 · light path · number: working, checked against main at merge

## Summary
The user, on the console's phone layout (2026-10-24, screenshot): "tu me fix le mobile aussi !" — after 083 every cell of a card was a labelled line, so the member's own name sat on the right behind a MEMBER label. Decided: below sm the first column a phone shows is the card's title — no label, at the start — and every other visible column stays a labelled line, the actions last. Rank 2: difficulty 1, impact 1.

## Log

| Step | Result |
| --- | --- |
| Build | `Column::classes_at(head, title)`; `row_cells` marks the first non-hidden column as the title (no `data-label`, `max-sm:justify-start`). Test: the title cell has no label nor `before:content`, a Role column added to the test table keeps its label. Looked at through the console (headless Chrome, 390 wide): avatar and address as the card's title, the role a line, the actions under it |
| Review | deep #15237: FIX 1 the title was the first visible column by position, so an actions column first would have been the headline — it is now the first visible column WITH a header; FIX 3 the case tested (an actions column first, a hidden column, then Member); FIX 5 a negative assertion (no `before:shrink-0`) and exactly one title per row. STRUCK 2: the head row stays for screen readers below sm (`max-sm:sr-only`, 083), so the title cell keeps its column header. 4 was already scoped to the actions cell (the test split for the line limit); 6 (a data attribute as a hook) not taken. 567 tests |

## Merge

0c844f9, released v0.4.11 (number and version checked free at merge)

## Retro

Light path: the Log line is the retro. A card where every line has a label buries the line that names the row; the review caught that "first" by position could crown the actions.
