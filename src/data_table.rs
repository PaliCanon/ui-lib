//! [`DataTable`] (079): rows of one kind in columns — members, applications.
//! It shows rows and nothing else: a row's actions are the caller's cells.
//!
//! Rows render through a keyed `<For>`, so a change re-renders the rows whose
//! key changed and no other. The key is the caller's contract: unique and
//! stable for the life of a row (an id, never an index); a duplicate is a
//! `debug_assert` — it would swap rows and drop their DOM state silently.
//!
//! The empty state is decided from the same rows the server rendered, so
//! SSR and hydration agree as long as the caller hands both the same initial
//! rows. Below `sm` each row stacks as a card (083): every cell on its own
//! line, its column's header beside it (`data-label`), the actions last —
//! CSS only, so SSR and hydration render the same markup; `hide_on_phone`
//! drops a column there, for SECONDARY columns only.

use std::sync::Arc;

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::catalogue::strings;

/// Where a column's content sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Text and identities.
    #[default]
    Start,
    /// Row actions.
    End,
}

/// One column: its header, how a row fills its cell, where it sits.
pub struct Column<T> {
    header: String,
    cell: Arc<dyn Fn(&T) -> AnyView + Send + Sync>,
    align: Align,
    hide_on_phone: bool,
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            cell: Arc::clone(&self.cell),
            align: self.align,
            hide_on_phone: self.hide_on_phone,
        }
    }
}

impl<T> Column<T> {
    /// A column headed `header` whose cell `cell` renders from a row. An
    /// empty header is an actions column: its header is visually hidden.
    pub fn new(
        header: impl Into<String>,
        cell: impl Fn(&T) -> AnyView + Send + Sync + 'static,
    ) -> Self {
        Self {
            header: header.into(),
            cell: Arc::new(cell),
            align: Align::Start,
            hide_on_phone: false,
        }
    }

    /// Sits the content at the row's end (actions).
    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Drops the column below `sm`: secondary information only.
    #[must_use]
    pub fn hide_on_phone(mut self) -> Self {
        self.hide_on_phone = true;
        self
    }

    fn classes(&self, head: bool) -> String {
        let align = match self.align {
            Align::Start => "text-left",
            Align::End => "text-right",
        };
        if head {
            let phone = if self.hide_on_phone {
                "hidden sm:table-cell"
            } else {
                ""
            };
            return format!(
                "px-5 py-3 text-xs font-medium uppercase tracking-wide text-muted \
                 border-b border-line {align} {phone}"
            );
        }
        let cell = "px-5 py-3.5 border-b border-line align-middle";
        // A secondary cell leaves phones whole: no card line to conflict with.
        if self.hide_on_phone {
            return format!("{cell} {align} max-sm:hidden");
        }
        // Below sm: a line of the row's card, the header as its label.
        let line = "max-sm:flex max-sm:items-center max-sm:gap-3 max-sm:border-0 \
                    max-sm:py-1.5 max-sm:min-w-0";
        let stacked = if self.header.is_empty() {
            "max-sm:justify-end max-sm:pt-3"
        } else {
            "max-sm:justify-between max-sm:before:shrink-0 max-sm:before:text-xs \
             max-sm:before:font-medium max-sm:before:uppercase max-sm:before:tracking-wide \
             max-sm:before:text-muted max-sm:before:content-[attr(data-label)]"
        };
        format!("{cell} {align} {line} {stacked}")
    }
}

/// Whether `keys` holds a key twice.
fn has_duplicate(keys: &[String]) -> bool {
    let mut seen: Vec<&String> = keys.iter().collect();
    seen.sort_unstable();
    seen.windows(2).any(|w| w[0] == w[1])
}

/// The header row's cells; an empty header is an actions column, its
/// header visually hidden but named.
fn head_cells<T>(columns: &[Column<T>]) -> impl IntoView + use<T> {
    let t = strings();
    columns
        .iter()
        .map(|c| {
            let text = if c.header.is_empty() {
                view! { <span class="sr-only">{td_string!(t.locale(), table_actions)}</span> }
                    .into_any()
            } else {
                c.header.clone().into_any()
            };
            view! {
                <th scope="col" role="columnheader" class=c.classes(true)>
                    {text}
                </th>
            }
        })
        .collect_view()
}

/// One row's cells, each carrying its column's header as `data-label`
/// (the card's label below sm, 083).
fn row_cells<T>(columns: &[Column<T>], row: &T) -> impl IntoView + use<T> {
    columns
        .iter()
        .map(|c| {
            view! {
                <td
                    role="cell"
                    class=c.classes(false)
                    data-label=(!c.header.is_empty()).then(|| c.header.clone())
                >
                    {(c.cell)(row)}
                </td>
            }
        })
        .collect_view()
}

/// The rows, after a debug check that no two share a key.
fn checked<T>(list: Vec<T>, key: &impl Fn(&T) -> String) -> Vec<T> {
    debug_assert!(
        !has_duplicate(&list.iter().map(key).collect::<Vec<_>>()),
        "DataTable: two rows share a key"
    );
    list
}

/// Rows of `T` in `columns`, keyed by `key`; `empty` when there are none.
#[component]
pub fn DataTable<T, K>(
    /// The columns, in order.
    columns: Vec<Column<T>>,
    /// The rows.
    #[prop(into)]
    rows: Signal<Vec<T>>,
    /// A row's key: unique and stable.
    key: K,
    /// The table's name for screen readers (its caption, visually hidden).
    #[prop(into)]
    label: String,
    /// What shows when there is no row; the library's line otherwise.
    #[prop(optional)]
    empty: Option<ViewFn>,
    /// Says the list was cut at a limit, under the rows.
    #[prop(optional, into)]
    truncated: Option<Signal<bool>>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    K: Fn(&T) -> String + Clone + Send + Sync + 'static,
{
    let t = strings();
    let heads = head_cells(&columns);
    let cols = StoredValue::new(columns);
    let span = cols.with_value(Vec::len);
    let row_key = key.clone();
    let body = move || checked(rows.get(), &row_key);
    let empty_line = move || match &empty {
        Some(view) => view.run(),
        None => td_string!(t.locale(), table_empty).into_any(),
    };
    let truncated = move || truncated.is_some_and(|s| s.get());
    view! {
        <div class="min-w-0 overflow-x-auto rounded-2xl border border-card-edge bg-card">
            // The roles are explicit: below sm the table's parts are blocks,
            // and some screen readers stop announcing a table then (083).
            <table role="table" class="w-full border-collapse text-sm text-ink max-sm:block">
                <caption class="sr-only">{label}</caption>
                // Below sm the cards show each header as a label; the head
                // row stays for screen readers, out of sight.
                <thead role="rowgroup" class="max-sm:sr-only">
                    <tr role="row">{heads}</tr>
                </thead>
                <tbody role="rowgroup" class="max-sm:block">
                    <For each=body key=key let:row>
                        <tr
                            role="row"
                            class="max-sm:block max-sm:border-b max-sm:border-line max-sm:px-0 max-sm:py-3 max-sm:last:border-b-0"
                        >
                            {cols.with_value(|cs| row_cells(cs, &row))}
                        </tr>
                    </For>
                    <Show when=move || rows.with(Vec::is_empty)>
                        <tr>
                            <td colspan=span class="px-5 py-10 text-center text-muted">
                                {empty_line.clone()}
                            </td>
                        </tr>
                    </Show>
                </tbody>
            </table>
            <Show when=truncated>
                <p class="m-0 px-5 py-3 text-xs text-muted border-t border-line">
                    {td_string!(t.locale(), table_truncated)}
                </p>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_duplicate_key_is_seen() {
        assert!(has_duplicate(&["a".into(), "b".into(), "a".into()]));
        assert!(!has_duplicate(&["a".into(), "b".into()]));
    }

    #[cfg(feature = "ssr")]
    mod render {
        use super::super::*;

        #[derive(Clone)]
        struct Member {
            sub: &'static str,
            email: &'static str,
        }

        fn columns() -> Vec<Column<Member>> {
            vec![
                Column::new("Member", |m: &Member| m.email.into_any()),
                Column::new("Joined", |_: &Member| "today".into_any()).hide_on_phone(),
                Column::new("", |m: &Member| format!("act {}", m.sub).into_any()).align(Align::End),
            ]
        }

        fn table(rows: Vec<Member>, truncated: bool) -> String {
            Owner::new().with(|| {
                view! {
                    <DataTable
                        columns=columns()
                        rows=Signal::derive(move || rows.clone())
                        key=|m: &Member| m.sub.to_string()
                        label="Members"
                        truncated=Signal::derive(move || truncated)
                    />
                }
                .to_html()
            })
        }

        #[test]
        fn rows_fill_their_columns_under_a_hidden_caption() {
            let out = table(
                vec![Member {
                    sub: "u1",
                    email: "ann@example.org",
                }],
                false,
            );
            assert!(out.contains("<caption") && out.contains("Members"), "{out}");
            assert!(
                out.contains("ann@example.org") && out.contains("act u1"),
                "{out}"
            );
            assert!(
                out.contains(r#"scope="col""#) && out.contains("sr-only"),
                "an actions header is hidden: {out}"
            );
            assert!(
                out.contains("hidden sm:table-cell"),
                "a secondary column leaves phones: {out}"
            );
            assert!(out.contains("overflow-x-auto"), "the box scrolls: {out}");
        }

        /// 083: below sm a row is a card — the head row leaves, every cell
        /// carries its column's header as `data-label` shown before it, and
        /// the actions cell sits at the end.
        #[test]
        fn below_sm_each_row_stacks_with_its_labels() {
            let out = table(
                vec![Member {
                    sub: "u1",
                    email: "ann@example.org",
                }],
                false,
            );
            assert!(
                out.contains(r#"<thead role="rowgroup" class="max-sm:sr-only">"#),
                "the head stays for screen readers: {out}"
            );
            assert!(
                out.contains(r#"<tr role="row" class="max-sm:block"#),
                "{out}"
            );
            assert!(
                out.contains(r#"<table role="table""#) && out.contains(r#"role="cell""#),
                "{out}"
            );
            assert!(out.contains(r#"data-label="Member""#), "{out}");
            // The secondary column's cell leaves phones and is no card line.
            let joined = out
                .split("<td")
                .find(|c| c.contains("today"))
                .expect("the Joined cell");
            assert!(joined.contains("max-sm:hidden"), "{joined}");
            assert!(
                !joined.contains("max-sm:flex") && !joined.contains("before:content"),
                "{joined}"
            );
            // The actions cell carries no empty label.
            let actions = out
                .split("<td")
                .find(|c| c.contains("act u1"))
                .expect("the actions cell");
            assert!(!actions.contains("data-label"), "{actions}");
            assert!(
                out.contains("max-sm:before:content-[attr(data-label)]"),
                "{out}"
            );
            assert!(
                out.contains("max-sm:justify-end"),
                "the actions last: {out}"
            );
            assert!(
                out.contains("min-w-0 overflow-x-auto"),
                "the box never widens the page: {out}"
            );
        }

        #[test]
        fn no_rows_says_so_and_a_cut_list_says_it_was_cut() {
            let out = table(vec![], true);
            assert!(out.contains("colspan=\"3\""), "{out}");
            assert!(out.contains("border-t"), "the truncated line: {out}");
        }
    }
}
