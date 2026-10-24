//! Fields (080): [`TextField`], one value; [`ListField`], several of one
//! kind (redirect URIs, one per line). A field is a `<label>`, its control,
//! an optional hint and an optional error: hint and error are linked to the
//! control by `aria-describedby`, and an error sets `aria-invalid`.
//!
//! A field never validates: the caller owns the rule and passes the error
//! (a console shows the server's refusal in its own words). `on_input` lets
//! the caller clear a stale error as the person types.

use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Button, Size, Variant, classes};
use crate::catalogue::strings;
use crate::icon::{Icon, IconName};

/// What a field holds: it sets the input's `type` (and so the phone
/// keyboard). `#[non_exhaustive]`: a kind added later breaks no service.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldKind {
    /// Plain text.
    #[default]
    Text,
    /// An address: a URL keyboard.
    Url,
    /// An email address.
    Email,
    /// A secret typed in: masked.
    Password,
    /// A search box.
    Search,
}

fn input_type(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::Text => "text",
        FieldKind::Url => "url",
        FieldKind::Email => "email",
        FieldKind::Password => "password",
        FieldKind::Search => "search",
    }
}

/// The control's look: one class set, its error edge a state variant.
fn control_classes(mono: bool) -> &'static str {
    if mono {
        "min-h-11 w-full min-w-0 rounded-xl border border-line bg-field px-3 font-mono text-[13px] text-ink placeholder:text-subtle aria-invalid:border-danger focus-visible:outline-2 focus-visible:outline-gold disabled:opacity-60"
    } else {
        "min-h-11 w-full min-w-0 rounded-xl border border-line bg-field px-3 text-sm text-ink placeholder:text-subtle aria-invalid:border-danger focus-visible:outline-2 focus-visible:outline-gold disabled:opacity-60"
    }
}

/// The ids `aria-describedby` lists: the hint's when there is one, the
/// error's while there is one.
fn described_by(id: &str, hint: bool, error: bool) -> Option<String> {
    let ids: Vec<String> = [(hint, "hint"), (error, "error")]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, part)| format!("{id}-{part}"))
        .collect();
    (!ids.is_empty()).then(|| ids.join(" "))
}

/// One value, labelled, with an optional hint and the caller's error.
#[component]
pub fn TextField(
    /// The control's id, unique on the page; the hint and error derive theirs.
    #[prop(into)]
    id: String,
    /// What the field asks for.
    #[prop(into)]
    label: String,
    /// The value, written as the person types.
    value: RwSignal<String>,
    /// What it holds; text by default.
    #[prop(optional)]
    kind: FieldKind,
    /// A greyed example inside the empty control.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// One line under the control.
    #[prop(optional, into)]
    hint: Option<String>,
    /// The caller's error, shown under the control while `Some`.
    #[prop(optional, into)]
    error: Option<Signal<Option<String>>>,
    /// The browser refuses to submit it empty.
    #[prop(optional)]
    required: bool,
    /// Not editable while true.
    #[prop(optional, into)]
    disabled: Option<Signal<bool>>,
    /// A monospaced value (an identifier, a URL).
    #[prop(optional)]
    mono: bool,
    /// The form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Runs with the new value on each keystroke.
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,
    /// Runs when the control loses focus.
    #[prop(optional, into)]
    on_blur: Option<Callback<()>>,
) -> impl IntoView {
    let error = error.unwrap_or_else(|| Signal::derive(|| None));
    let has_hint = hint.is_some();
    let describe = {
        let id = id.clone();
        move || described_by(&id, has_hint, error.with(Option::is_some))
    };
    let messages = field_messages(&id, hint, error);
    view! {
        <div class="flex flex-col gap-1.5">
            <label for=id.clone() class="text-sm text-ink">
                {label}
            </label>
            <input
                id=id
                type=input_type(kind)
                name=name
                placeholder=placeholder
                required=required
                class=control_classes(mono)
                prop:value=move || value.get()
                disabled=move || disabled.is_some_and(|d| d.get())
                aria-invalid=move || error.with(Option::is_some).then_some("true")
                aria-describedby=describe
                on:input=move |ev| {
                    let text = event_target_value(&ev);
                    value.set(text.clone());
                    if let Some(run) = on_input {
                        run.run(text);
                    }
                }
                on:blur=move |_| {
                    if let Some(run) = on_blur {
                        run.run(());
                    }
                }
            />
            {messages}
        </div>
    }
}

/// The hint and the error under a control, each with the id
/// `aria-describedby` names.
fn field_messages(
    id: &str,
    hint: Option<String>,
    error: Signal<Option<String>>,
) -> impl IntoView + use<> {
    let hint_id = format!("{id}-hint");
    let error_id = format!("{id}-error");
    let hint = hint.map(|h| {
        view! {
            <p id=hint_id class="m-0 text-xs text-muted">
                {h}
            </p>
        }
    });
    let error_line = move || {
        error.get().map(|e| {
            view! {
                <p id=error_id.clone() class="m-0 text-xs text-danger-text">
                    {e}
                </p>
            }
        })
    };
    view! {
        {hint}
        {error_line}
    }
}

static NEXT_ROW: AtomicU64 = AtomicU64::new(1);

/// One line of a [`ListField`]: its value and an id drawn once, which keys
/// it — so replacing the list wholesale, or removing a line, keeps every
/// other line's DOM and focus.
///
/// The id is drawn from a counter of the process: a server render and the
/// browser draw different ones, so a list rendered on the server is
/// rebuilt, not reused, when the page hydrates. Build the rows where the
/// form is used (a console's SPA builds them in the browser alone).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListRow {
    id: u64,
    /// What the line holds.
    pub value: String,
}

impl ListRow {
    /// A line holding `value`, with a new id.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            id: NEXT_ROW.fetch_add(1, Ordering::Relaxed),
            value: value.into(),
        }
    }

    /// The line's id: what an error names it by.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Lines from values.
    #[must_use]
    pub fn from_values<S: Into<String>>(values: impl IntoIterator<Item = S>) -> Vec<Self> {
        values.into_iter().map(Self::new).collect()
    }
}

/// Where the focus goes once the line `removed` is gone from `ids` (the ids
/// before the removal): the next line, else the previous, else Add.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusAfter {
    /// The line with this id.
    Row(u64),
    /// The Add button.
    Add,
}

/// See [`FocusAfter`].
#[must_use]
pub fn focus_after_remove(ids: &[u64], removed: u64) -> FocusAfter {
    let Some(at) = ids.iter().position(|id| *id == removed) else {
        return FocusAfter::Add;
    };
    ids.get(at + 1)
        .or_else(|| at.checked_sub(1).and_then(|p| ids.get(p)))
        .map_or(FocusAfter::Add, |id| FocusAfter::Row(*id))
}

/// The DOM id of the line at `index` in the list `id`: by position, so the
/// server and the browser agree whatever ids they drew.
fn row_dom_id(id: &str, index: usize) -> String {
    format!("{id}-{index}")
}

/// Moves the keyboard focus to the element `id` once the DOM has the change;
/// nothing on the server.
fn focus_soon(id: String) {
    #[cfg(feature = "browser")]
    request_animation_frame(move || {
        use wasm_bindgen::JsCast;
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = el.focus();
        }
    });
    #[cfg(not(feature = "browser"))]
    let _ = id;
}

/// Several values of one kind, one per line, each removable, with an Add.
/// At least one line stays: the last one has no Remove.
#[component]
pub fn ListField(
    /// The prefix of every id in the field, unique on the page.
    #[prop(into)]
    id: String,
    /// What the field asks for.
    #[prop(into)]
    label: String,
    /// The lines, written as the person types, adds and removes.
    rows: RwSignal<Vec<ListRow>>,
    /// What each line holds; text by default.
    #[prop(optional)]
    kind: FieldKind,
    /// One line under the list.
    #[prop(optional, into)]
    hint: Option<String>,
    /// The caller's errors, by line id ([`ListRow::id`]).
    #[prop(optional, into)]
    errors: Option<Signal<Vec<(u64, String)>>>,
    /// Runs with a line's id as the person types in it: the caller clears
    /// that line's error there.
    #[prop(optional, into)]
    on_input: Option<Callback<u64>>,
    /// Monospaced values (URLs, identifiers).
    #[prop(optional)]
    mono: bool,
) -> impl IntoView {
    let errors = errors.unwrap_or_else(|| Signal::derive(Vec::new));
    let ids = StoredValue::new(id.clone());
    let add_id = format!("{id}-add");
    let t = strings();
    let hint = hint.map(|h| view! { <p class="m-0 text-xs text-muted">{h}</p> });
    let add = move |_| {
        rows.update(|r| r.push(ListRow::new("")));
        let last = rows.with_untracked(Vec::len) - 1;
        focus_soon(row_dom_id(&ids.get_value(), last));
    };
    view! {
        <fieldset class="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
            <legend class="mb-1.5 p-0 text-sm text-ink">{label}</legend>
            <For each=move || rows.get() key=ListRow::id let:row>
                <ListLine
                    row=row
                    rows=rows
                    ids=ids
                    kind=kind
                    mono=mono
                    errors=errors
                    on_input=on_input
                />
            </For>
            {hint}
            <Button variant=Variant::Ghost size=Size::Sm layout="self-start" on_click=add>
                <span id=add_id.clone()>{move || td_string!(t.locale(), field_add)}</span>
            </Button>
        </fieldset>
    }
}

/// One line of a [`ListField`]: its input, its Remove, its error.
#[component]
fn ListLine(
    row: ListRow,
    rows: RwSignal<Vec<ListRow>>,
    ids: StoredValue<String>,
    kind: FieldKind,
    mono: bool,
    errors: Signal<Vec<(u64, String)>>,
    on_input: Option<Callback<u64>>,
) -> impl IntoView {
    let key = row.id;
    // `None` for the frame a removed line is still on screen: it then has no
    // id at all, never the first line's (review 080).
    let index = move || rows.with(|r| r.iter().position(|x| x.id == key));
    let dom_id = move || index().map(|at| row_dom_id(&ids.get_value(), at));
    let error = move || errors.with(|e| e.iter().find(|(k, _)| *k == key).map(|(_, m)| m.clone()));
    let error_id = move || dom_id().map(|d| format!("{d}-error"));
    let t = strings();
    let remove_label = move || {
        let value = rows.with(|r| {
            r.iter()
                .find(|x| x.id == key)
                .map(|x| x.value.clone())
                .unwrap_or_default()
        });
        if value.trim().is_empty() {
            td_string!(t.locale(), field_remove_empty).to_string()
        } else {
            td_string!(t.locale(), field_remove, value = value).to_string()
        }
    };
    let remove = move |_| {
        let before: Vec<u64> = rows.with_untracked(|r| r.iter().map(|x| x.id).collect());
        let target = focus_after_remove(&before, key);
        rows.update(|r| r.retain(|x| x.id != key));
        let id = ids.get_value();
        match target {
            FocusAfter::Row(next) => {
                let at = rows.with_untracked(|r| r.iter().position(|x| x.id == next).unwrap_or(0));
                focus_soon(row_dom_id(&id, at));
            }
            FocusAfter::Add => focus_soon(format!("{id}-add")),
        }
    };
    let alone = move || rows.with(Vec::len) <= 1;
    view! {
        <div class="flex flex-col gap-1">
            <div class="flex items-center gap-2">
                <input
                    id=dom_id
                    type=input_type(kind)
                    class=control_classes(mono)
                    prop:value=row.value.clone()
                    aria-label=move || {
                        index().map(|at| td_string!(t.locale(), field_line, n = at + 1).to_string())
                    }
                    aria-invalid=move || error().is_some().then_some("true")
                    aria-describedby=move || error().and(error_id())
                    on:input=move |ev| {
                        let text = event_target_value(&ev);
                        rows.update(|r| {
                            if let Some(x) = r.iter_mut().find(|x| x.id == key) {
                                x.value = text;
                            }
                        });
                        if let Some(run) = on_input {
                            run.run(key);
                        }
                    }
                />
                // The button family's look, with a name that follows the line's
                // content (`Button` takes a fixed label).
                <Show when=move || !alone()>
                    <button
                        type="button"
                        class=classes(Variant::Icon, Size::Md, "shrink-0")
                        aria-label=remove_label
                        title=remove_label
                        on:click=remove
                    >
                        <Icon name=IconName::Close size=16 />
                    </button>
                </Show>
            </div>
            {move || {
                error()
                    .map(|e| {
                        view! {
                            <p id=error_id() class="m-0 text-xs text-danger-text">
                                {e}
                            </p>
                        }
                    })
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn described_by_lists_only_what_is_there() {
        assert_eq!(described_by("f", false, false), None);
        assert_eq!(described_by("f", true, false).as_deref(), Some("f-hint"));
        assert_eq!(
            described_by("f", true, true).as_deref(),
            Some("f-hint f-error")
        );
        assert_eq!(described_by("f", false, true).as_deref(), Some("f-error"));
    }

    #[test]
    fn the_focus_goes_next_else_previous_else_to_add() {
        assert_eq!(focus_after_remove(&[1, 2, 3], 2), FocusAfter::Row(3));
        assert_eq!(focus_after_remove(&[1, 2, 3], 3), FocusAfter::Row(2));
        assert_eq!(focus_after_remove(&[7], 7), FocusAfter::Add);
        assert_eq!(
            focus_after_remove(&[1, 2], 9),
            FocusAfter::Add,
            "an unknown line"
        );
    }

    #[test]
    fn rows_draw_distinct_ids_and_keep_them() {
        let rows = ListRow::from_values(["a", "b"]);
        assert_ne!(rows[0].id(), rows[1].id());
        let mut kept = rows.clone();
        kept.retain(|r| r.value != "a");
        assert_eq!(kept[0].id(), rows[1].id(), "a removal keeps the other ids");
    }

    #[cfg(feature = "ssr")]
    mod render {
        use super::super::*;

        fn html(view: impl FnOnce() -> AnyView) -> String {
            Owner::new().with(|| view().to_html())
        }

        #[test]
        fn a_text_field_is_labelled_and_describes_its_hint_and_error() {
            let out = html(|| {
                let value = RwSignal::new("x".to_string());
                let error = Signal::derive(|| Some("Must be https".to_string()));
                view! {
                    <TextField
                        id="start"
                        label="Start URL"
                        value=value
                        kind=FieldKind::Url
                        hint="Where the tile leads"
                        error=error
                    />
                }
                .into_any()
            });
            assert!(
                out.contains(r#"for="start""#) && out.contains(r#"id="start""#),
                "{out}"
            );
            assert!(out.contains(r#"type="url""#), "{out}");
            assert!(
                out.contains(r#"aria-describedby="start-hint start-error""#),
                "{out}"
            );
            assert!(
                out.contains(r#"aria-invalid="true""#) && out.contains("Must be https"),
                "{out}"
            );
        }

        #[test]
        fn a_field_without_error_is_not_invalid() {
            let out = html(|| {
                let value = RwSignal::new(String::new());
                view! { <TextField id="n" label="Name" value=value /> }.into_any()
            });
            assert!(
                !out.contains("aria-invalid=") && !out.contains("aria-describedby="),
                "{out}"
            );
        }

        #[test]
        fn a_list_field_has_a_legend_one_input_per_line_and_no_remove_when_alone() {
            let two = html(|| {
                let rows = RwSignal::new(ListRow::from_values([
                    "https://a.example/cb",
                    "https://b.example/cb",
                ]));
                view! { <ListField id="r" label="Redirect URIs" rows=rows kind=FieldKind::Url /> }
                    .into_any()
            });
            assert!(
                two.contains("<legend") && two.contains("Redirect URIs"),
                "{two}"
            );
            assert_eq!(two.matches(r#"type="url""#).count(), 2, "{two}");
            assert!(
                two.contains(r#"id="r-0""#) && two.contains(r#"id="r-1""#),
                "{two}"
            );
            assert_eq!(
                two.matches("aria-label=\"Remove https://").count(),
                2,
                "{two}"
            );
            let one = html(|| {
                let rows = RwSignal::new(ListRow::from_values(["x"]));
                view! { <ListField id="r" label="L" rows=rows /> }.into_any()
            });
            assert!(!one.contains("Remove"), "the last line stays: {one}");
        }

        #[test]
        fn a_line_error_is_linked_to_its_line_alone() {
            let out = html(|| {
                let rows = RwSignal::new(ListRow::from_values(["http://a", "https://b"]));
                let bad = rows.get_untracked()[0].id();
                let errors = Signal::derive(move || vec![(bad, "Must be https".to_string())]);
                view! { <ListField id="r" label="L" rows=rows errors=errors /> }.into_any()
            });
            assert_eq!(out.matches(r#"aria-invalid="true""#).count(), 1, "{out}");
            assert!(
                out.contains(r#"aria-describedby="r-0-error""#)
                    && out.contains(r#"id="r-0-error""#),
                "{out}"
            );
        }
    }
}
