//! [`Tabs`] and [`TabPanel`] (079): one view of a few, chosen by a row of
//! tabs — "Owned · 2" / "Reached · 3". The WAI-ARIA tabs pattern: a
//! `tablist` of `tab`s with a roving tabindex (only the selected tab is in
//! the Tab order), Arrow Left/Right move and select, Home/End go to the ends;
//! each [`TabPanel`] is labelled by its tab.
//!
//! Both take one [`TabSet`], made once by the caller: it holds the ids'
//! prefix, the options and the value, so a tab and its panel cannot be wired
//! to two different signals. The selection shown is DERIVED — a value naming
//! no option shows the first one — and nothing is written during render.
//! A panel's content is built while it is shown, and dropped when it is not.

use leptos::prelude::*;

/// One tab: its value, its label and an optional count shown after it.
/// Built with [`TabOption::new`] and [`TabOption::with_count`]; fields may
/// be added, so it is not built literally outside this crate.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabOption {
    /// What the value holds when this tab is selected.
    pub value: String,
    /// What the tab says.
    pub label: String,
    /// A count shown after the label (`Owned · 2`).
    pub count: Option<usize>,
}

impl TabOption {
    /// A tab with no count.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            count: None,
        }
    }

    /// The same tab with a count.
    #[must_use]
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
}

/// The id of the tab for `value` in the tab set `id`.
#[must_use]
pub fn tab_id(id: &str, value: &str) -> String {
    format!("{id}-tab-{value}")
}

/// The id of the panel for `value` in the tab set `id`.
#[must_use]
pub fn panel_id(id: &str, value: &str) -> String {
    format!("{id}-panel-{value}")
}

/// The value shown: `value`, or the first option's when it names none.
fn shown_value(options: &[TabOption], value: &str) -> String {
    if options.iter().any(|o| o.value == value) {
        value.to_string()
    } else {
        options.first().map(|o| o.value.clone()).unwrap_or_default()
    }
}

/// The value a key moves to from `current`, if the key moves at all.
fn moved(options: &[TabOption], current: &str, key: &str) -> Option<String> {
    let at = options.iter().position(|o| o.value == current)?;
    let last = options.len().checked_sub(1)?;
    let to = match key {
        "ArrowRight" => {
            if at == last {
                0
            } else {
                at + 1
            }
        }
        "ArrowLeft" => {
            if at == 0 {
                last
            } else {
                at - 1
            }
        }
        "Home" => 0,
        "End" => last,
        _ => return None,
    };
    Some(options[to].value.clone())
}

/// One set of tabs and their panels: made once, given to [`Tabs`] and to
/// every [`TabPanel`]. `Copy`, so it moves into closures freely.
#[derive(Clone, Copy, Debug)]
pub struct TabSet {
    id: StoredValue<String>,
    options: StoredValue<Vec<TabOption>>,
    value: RwSignal<String>,
    shown: Memo<String>,
}

impl TabSet {
    /// A set whose ids start with `id` (unique on the page), holding
    /// `options` and selecting through `value`.
    ///
    /// `value` is the caller's input: a click or a key writes it, but a value
    /// naming no option is left as it is while the first tab is SHOWN — read
    /// the selection back through [`TabSet::shown`], never `value` (a filter
    /// or a `?tab=` mirror on `value` would disagree with the screen). Make
    /// the set inside the component that renders it: it lives in that
    /// component's reactive owner and must not outlive it.
    #[must_use]
    pub fn new(id: impl Into<String>, options: Vec<TabOption>, value: RwSignal<String>) -> Self {
        let options = StoredValue::new(options);
        let shown = Memo::new(move |_| options.with_value(|o| shown_value(o, &value.get())));
        Self {
            id: StoredValue::new(id.into()),
            options,
            value,
            shown,
        }
    }

    /// The value shown now (tracked).
    #[must_use]
    pub fn shown(&self) -> String {
        self.shown.get()
    }

    /// Selects `value`.
    pub fn select(&self, value: String) {
        self.value.set(value);
    }
}

fn tab_classes(on: bool) -> &'static str {
    if on {
        "min-h-10 px-3.5 -mb-px border-0 border-b-2 border-gold bg-transparent text-sm font-medium text-heading cursor-pointer focus-visible:outline-2 focus-visible:outline-gold"
    } else {
        "min-h-10 px-3.5 -mb-px border-0 border-b-2 border-transparent bg-transparent text-sm text-muted cursor-pointer hover:text-ink focus-visible:outline-2 focus-visible:outline-gold"
    }
}

/// The row of tabs of `set`; put one [`TabPanel`] per option beside it.
#[component]
pub fn Tabs(
    /// The set, shared with its panels.
    set: TabSet,
    /// The list's name for screen readers.
    #[prop(into)]
    label: String,
) -> impl IntoView {
    let tabs = set
        .options
        .get_value()
        .into_iter()
        .map(|option| tab(set, option))
        .collect_view();
    view! {
        <div role="tablist" aria-label=label class="flex flex-wrap gap-1 border-b border-line">
            {tabs}
        </div>
    }
}

/// One tab of `set`: selected when shown, moving the selection with keys.
fn tab(set: TabSet, option: TabOption) -> impl IntoView {
    let id = set.id.get_value();
    let v = option.value.clone();
    let on = {
        let v = v.clone();
        move || set.shown.get() == v
    };
    let (on2, on3) = (on.clone(), on.clone());
    let text = match option.count {
        Some(n) => format!("{} · {n}", option.label),
        None => option.label,
    };
    let pick = v.clone();
    view! {
        <button
            type="button"
            role="tab"
            id=tab_id(&id, &v)
            aria-controls=panel_id(&id, &v)
            aria-selected=move || on().to_string()
            tabindex=move || if on2() { "0" } else { "-1" }
            class=move || tab_classes(on3())
            on:click=move |_| set.select(pick.clone())
            on:keydown=move |ev| {
                let current = set.shown.get_untracked();
                if let Some(to) = set.options.with_value(|o| moved(o, &current, &ev.key())) {
                    ev.prevent_default();
                    focus_tab(&tab_id(&set.id.get_value(), &to));
                    set.select(to);
                }
            }
        >
            {text}
        </button>
    }
}

/// Moves the keyboard focus to the tab `id` (a roving tabindex follows the
/// selection); nothing on the server.
fn focus_tab(id: &str) {
    #[cfg(feature = "browser")]
    {
        use wasm_bindgen::JsCast;
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = el.focus();
        }
    }
    #[cfg(not(feature = "browser"))]
    let _ = id;
}

/// The view of `value` in `set`: labelled by its tab, its content built only
/// while it is shown.
#[component]
pub fn TabPanel(
    /// The set, the same as the [`Tabs`]'.
    set: TabSet,
    /// The value this panel belongs to.
    #[prop(into)]
    value: String,
    /// The panel's content.
    children: ChildrenFn,
) -> impl IntoView {
    let id = set.id.get_value();
    let mine = value.clone();
    let shown = move || set.shown.get() == mine;
    view! {
        <div
            role="tabpanel"
            id=panel_id(&id, &value)
            aria-labelledby=tab_id(&id, &value)
            hidden=move || !shown()
        >
            <Show when=shown.clone()>{children()}</Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> Vec<TabOption> {
        vec![
            TabOption::new("owned", "Owned").with_count(2),
            TabOption::new("reached", "Reached"),
            TabOption::new("all", "All"),
        ]
    }

    #[test]
    fn keys_move_round_and_to_the_ends() {
        let o = opts();
        assert_eq!(moved(&o, "owned", "ArrowRight").as_deref(), Some("reached"));
        assert_eq!(
            moved(&o, "all", "ArrowRight").as_deref(),
            Some("owned"),
            "wraps"
        );
        assert_eq!(
            moved(&o, "owned", "ArrowLeft").as_deref(),
            Some("all"),
            "wraps back"
        );
        assert_eq!(moved(&o, "reached", "Home").as_deref(), Some("owned"));
        assert_eq!(moved(&o, "owned", "End").as_deref(), Some("all"));
        assert_eq!(moved(&o, "owned", "Enter"), None);
    }

    #[test]
    fn an_unknown_value_shows_the_first_tab() {
        assert_eq!(shown_value(&opts(), "gone"), "owned");
        assert_eq!(shown_value(&[], "gone"), "");
    }

    #[cfg(feature = "ssr")]
    fn render(start: &str) -> (String, String) {
        let start = start.to_string();
        Owner::new().with(move || {
            let value = RwSignal::new(start);
            let set = TabSet::new("apps", opts(), value);
            let out = view! {
                <Tabs set=set label="Applications" />
                <TabPanel set=set value="owned">
                    "OWNED"
                </TabPanel>
                <TabPanel set=set value="reached">
                    "REACHED"
                </TabPanel>
            }
            .to_html();
            (out, value.get_untracked())
        })
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn the_selected_tab_alone_is_in_the_tab_order_and_labels_its_panel() {
        let (out, _) = render("reached");
        assert!(
            out.contains(r#"role="tablist""#) && out.contains(r#"aria-label="Applications""#),
            "{out}"
        );
        assert_eq!(out.matches(r#"tabindex="0""#).count(), 1, "{out}");
        assert!(
            out.contains(r#"id="apps-tab-reached""#) && out.contains(r#"aria-selected="true""#),
            "{out}"
        );
        assert!(
            out.contains(r#"aria-labelledby="apps-tab-reached""#),
            "{out}"
        );
        assert!(out.contains("Owned · 2"), "{out}");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn only_the_shown_panel_builds_its_content() {
        let (out, _) = render("reached");
        assert!(out.contains("REACHED") && !out.contains("OWNED"), "{out}");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn an_unknown_value_shows_the_first_panel_and_writes_nothing() {
        let (out, value) = render("gone");
        assert!(out.contains("OWNED") && !out.contains("REACHED"), "{out}");
        assert_eq!(value, "gone", "the caller's value is left as it was");
    }
}
