//! A generic menu: a list of [`MenuItem`]s, rendered as a [`Menu`] (a
//! [`Popover`] holding the list, submenus nested to the right — desktop) or
//! as a [`DrillMenu`] (the list inline, a chosen submenu replacing it, with
//! a "Back" row on top — mobile, with no hover-only interaction). Both read
//! the same items; each owns its own open/level state.
//!
//! - A [`MenuItem::Action`] runs its callback, then the whole menu closes.
//! - A [`MenuItem::Choice`] runs its callback and the menu stays open: it is
//!   one of a group, marked with a check mark when selected.
//! - A [`MenuItem::Submenu`] opens a nested list: on desktop, hover
//!   (`pointerenter`), a click, or `ArrowRight`; `ArrowLeft` or Escape closes
//!   the deepest open one, an outside click (or choosing an action or a
//!   link) closes the whole menu. On mobile, choosing it drills down. Its
//!   panel opens to the entry's right, sized to its widest row, top-aligned
//!   with the entry; once visible, it is bottom-aligned and grows up when it
//!   would run past the viewport's bottom, and opens to the left when there
//!   is no room on the right.
//!
//! Settings ([`crate::shell::AppShell`]'s `settings` prop) is a `Menu`: an
//! app can pass its own items, any depth of submenus, alongside or instead
//! of [`crate::theme::theme_menu`].

use leptos::html;
use leptos::prelude::*;

use leptos_i18n::td_string;

use crate::button::{Disclosure, Size, Variant, classes};
use crate::catalogue::strings;
use crate::icon::{Icon, IconName};
use crate::popover::{PANEL, Popover};

/// The index chosen at each depth, root to the deepest open submenu; shared
/// by every level of one [`Menu`] or [`DrillMenu`].
pub(crate) type Path = RwSignal<Vec<usize>>;

/// One entry of a [`Menu`] or [`DrillMenu`].
#[derive(Clone)]
pub enum MenuItem {
    /// Goes to another page.
    Link {
        /// Shown beside the icon; its name for assistive technology.
        label: &'static str,
        /// Its icon, if any.
        icon: Option<IconName>,
        /// Where it goes.
        href: &'static str,
    },
    /// Runs `on_select`, then the whole menu closes.
    Action {
        /// Shown beside the icon.
        label: &'static str,
        /// Its icon, if any.
        icon: Option<IconName>,
        /// What choosing it does.
        on_select: Callback<()>,
    },
    /// One choice of a group: `selected` says whether it is the group's
    /// current one; choosing it runs `on_select`, and the menu stays open.
    Choice {
        /// Shown beside the icon.
        label: &'static str,
        /// Its icon, if any.
        icon: Option<IconName>,
        /// Whether this is the group's current choice.
        selected: Signal<bool>,
        /// What choosing it does, usually setting the group's value.
        on_select: Callback<()>,
    },
    /// Opens a nested list of `items`.
    Submenu {
        /// Shown beside the icon.
        label: &'static str,
        /// Its icon, if any.
        icon: Option<IconName>,
        /// The nested items.
        items: Vec<MenuItem>,
    },
    /// Anything else (an info block, a divider): rendered as given, never a
    /// button.
    Custom(ViewFn),
}

/// The items of `items` at the end of `path`, walking down one
/// [`MenuItem::Submenu`] per index; `items` itself once `path` runs out or
/// no longer names a submenu (its list changed under it).
fn items_at<'a>(items: &'a [MenuItem], path: &[usize]) -> &'a [MenuItem] {
    let mut current = items;
    for &index in path {
        match current.get(index) {
            Some(MenuItem::Submenu { items, .. }) => current = items,
            _ => break,
        }
    }
    current
}

/// The icon, label and right-aligned extra (a chevron, a check mark) of one
/// row, shared by every kind of item.
pub(crate) fn item_row(
    icon: Option<IconName>,
    label: impl IntoView,
    right: impl IntoView,
) -> impl IntoView {
    view! {
        <span class="flex flex-1 items-center gap-3">
            {icon.map(|name| view! { <Icon name=name /> })}
            <span>{label}</span>
        </span>
        {right}
    }
}

/// The check mark of a [`MenuItem::Choice`]: shown only while `shown`.
#[component]
fn CheckMark(#[prop(into)] shown: Signal<bool>) -> impl IntoView {
    view! {
        <span
            class="w-4 text-center text-gold transition-opacity duration-150"
            class:opacity-0=move || !shown.get()
            aria-hidden="true"
        >
            <Icon name=IconName::Check size=14 />
        </span>
    }
}

// --- Desktop: `Menu`, nested side by side -----------------------------

/// A list of [`MenuItem`]s in a [`Popover`]: the desktop rendering of a
/// menu. A [`MenuItem::Submenu`] opens nested, to the right of its entry.
#[component]
pub fn Menu<D: Disclosure>(
    /// What opens and closes the whole menu.
    controls: D,
    /// Its name for assistive technology.
    #[prop(into)]
    aria_label: String,
    /// The items, top to bottom.
    items: Vec<MenuItem>,
    /// Layout classes (position) added to the popover's look.
    #[prop(optional, into)]
    layout: String,
) -> impl IntoView {
    let path: Path = RwSignal::new(Vec::new());
    // Collapsed submenus the next time it opens: reopening a menu always
    // starts at its top level.
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if !controls.is_open() {
            path.set(Vec::new());
        }
    });
    let close_all = Callback::new(move |()| {
        path.set(Vec::new());
        controls.set_open(false);
    });
    view! {
        <Popover controls=controls aria_label=aria_label layout=layout>
            <MenuLevel items=items path=path depth=0 close_all=close_all />
        </Popover>
    }
}

/// One level of a [`Menu`]: its own `role="menu"`, so Escape and the arrow
/// keys ([`level_keydown`]) stay scoped to its own items. A submenu's panel
/// is another `MenuLevel`, nested inside its entry.
#[component]
fn MenuLevel(
    items: Vec<MenuItem>,
    path: Path,
    depth: usize,
    close_all: Callback<()>,
) -> impl IntoView {
    view! {
        <div
            role="menu"
            class="flex w-max min-w-56 flex-col gap-0.5 p-1"
            on:keydown=move |ev| level_keydown(ev, path)
        >
            {items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    view! { <DesktopItem item=item path=path depth=depth index=index close_all=close_all /> }
                })
                .collect_view()}
        </div>
    }
}

/// Escape or `ArrowLeft`, with a submenu open anywhere in this `Menu`, closes
/// just the deepest one and stops the event from also reaching the
/// `Popover`'s own Escape listener on `window` (it bubbles here first, from
/// whichever item has focus). `ArrowUp` and `ArrowDown` move focus among this
/// level's own items (hydrate only: they need the DOM).
fn level_keydown(ev: leptos::ev::KeyboardEvent, path: Path) {
    let key = ev.key();
    if (key == "Escape" || key == "ArrowLeft") && !path.get_untracked().is_empty() {
        path.update(|p| {
            p.pop();
        });
        ev.stop_propagation();
        return;
    }
    arrow_focus(&ev, &key);
}

#[cfg(feature = "hydrate")]
fn arrow_focus(ev: &leptos::ev::KeyboardEvent, key: &str) {
    use wasm_bindgen::JsCast;
    if key != "ArrowDown" && key != "ArrowUp" {
        return;
    }
    if let Some(menu) = ev
        .current_target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    {
        move_focus(&menu, key == "ArrowDown");
    }
    ev.prevent_default();
}

#[cfg(not(feature = "hydrate"))]
fn arrow_focus(_ev: &leptos::ev::KeyboardEvent, _key: &str) {}

/// Moves focus to the next (or, `!forward`, previous) item of `menu`'s own
/// level (its direct children's `[role^="menuitem"]`), wrapping around.
#[cfg(feature = "hydrate")]
fn move_focus(menu: &web_sys::Element, forward: bool) {
    use wasm_bindgen::JsCast;
    let Ok(list) = menu.query_selector_all(":scope > div > [role^='menuitem']") else {
        return;
    };
    let len = list.length();
    if len == 0 {
        return;
    }
    let active = document().active_element();
    let current = active
        .as_ref()
        .and_then(|a| (0..len).find(|&i| list.item(i).is_some_and(|n| n.is_same_node(Some(a)))));
    let next = match current {
        Some(i) if forward => (i + 1) % len,
        Some(i) => (i + len - 1) % len,
        None => 0,
    };
    if let Some(el) = list.item(next) {
        let _ = el.unchecked_ref::<web_sys::HtmlElement>().focus();
    }
}

/// How far a submenu panel stays from the viewport's edges: half a rem at
/// the default 16px root font size.
#[cfg(feature = "hydrate")]
const SUBMENU_VIEWPORT_MARGIN_PX: f64 = 8.0;

/// Places a submenu panel (`panel`, absolutely positioned inside its
/// entry's `relative` wrapper) once it is visible:
///
/// 1. Vertically, it starts at its entry's top and grows down (the default
///    `-top-3`, which cancels the panels' padding so the first row lines up
///    with the entry).
/// 2. If its height would run past the viewport's bottom, it is aligned on
///    its entry's bottom instead and grows up.
/// 3. If even that would run past the viewport's top, it is pinned just
///    below the top.
/// 4. Horizontally, it opens to the entry's right, or to its left when it
///    would run past the viewport's right edge.
///
/// The browser only: there is no window to measure against while the
/// server renders.
#[cfg(feature = "hydrate")]
fn position_submenu(panel: NodeRef<html::Div>) {
    request_animation_frame(move || {
        let Some(el) = panel.get_untracked() else {
            return;
        };
        let Some(entry) = el.parent_element() else {
            return;
        };
        // Fully qualified: `web_sys::HtmlElement::style` would otherwise be
        // shadowed by tachys's own `style` attribute-setter extension trait.
        let style = web_sys::HtmlElement::style(&el);
        for prop in [
            "top",
            "bottom",
            "left",
            "right",
            "margin-left",
            "margin-right",
        ] {
            let _ = style.remove_property(prop);
        }
        let (vw, vh) = crate::viewport::visible_size();
        let entry_box = entry.get_bounding_client_rect();
        let panel_box = el.get_bounding_client_rect();
        place_vertically(&style, &entry_box, panel_box.height(), vh);
        if entry_box.right() + panel_box.width() + SUBMENU_VIEWPORT_MARGIN_PX > vw {
            let _ = style.set_property("left", "auto");
            let _ = style.set_property("right", "100%");
            let _ = style.set_property("margin-left", "0");
            let _ = style.set_property("margin-right", "0.25rem");
        }
    });
}

/// Steps 1 to 3 of [`position_submenu`], for a panel `height` pixels tall
/// next to `entry`, in a viewport `vh` pixels tall.
#[cfg(feature = "hydrate")]
fn place_vertically(
    style: &web_sys::CssStyleDeclaration,
    entry: &web_sys::DomRect,
    height: f64,
    vh: f64,
) {
    let margin = SUBMENU_VIEWPORT_MARGIN_PX;
    if entry.top() + height <= vh - margin {
        return; // fits below: the default, top-aligned
    }
    if entry.bottom() - height >= margin {
        // Bottom-aligned with the entry, growing up (the same padding offset).
        let _ = style.set_property("top", "auto");
        let _ = style.set_property("bottom", "-0.75rem");
    } else {
        // Taller than the room above too: pinned just below the viewport's top.
        let _ = style.set_property("top", &format!("{}px", margin - entry.top()));
    }
}

/// One item of a [`MenuLevel`], wrapped in a `relative` container so a
/// [`MenuItem::Submenu`]'s panel can be positioned `absolute` against it.
#[component]
fn DesktopItem(
    item: MenuItem,
    path: Path,
    depth: usize,
    index: usize,
    close_all: Callback<()>,
) -> impl IntoView {
    let open = Signal::derive(move || path.get().get(depth) == Some(&index));
    view! { <div class="relative">{desktop_item_content(item, path, depth, index, open, close_all)}</div> }
}

/// The content of one [`DesktopItem`]: the row itself, and, for a
/// [`MenuItem::Submenu`], its nested [`MenuLevel`].
fn desktop_item_content(
    item: MenuItem,
    path: Path,
    depth: usize,
    index: usize,
    open: Signal<bool>,
    close_all: Callback<()>,
) -> impl IntoView {
    match item {
        MenuItem::Submenu { label, icon, items } => {
            let on_open = Callback::new(move |()| {
                path.update(|p| {
                    p.truncate(depth);
                    p.push(index);
                });
            });
            let panel_ref: NodeRef<html::Div> = NodeRef::new();
            #[cfg(feature = "hydrate")]
            Effect::new(move |_| {
                if open.get() {
                    position_submenu(panel_ref);
                }
            });
            view! {
                <SubmenuTrigger label=label icon=icon open=open on_open=on_open />
                <div
                    node_ref=panel_ref
                    class=format!("{PANEL} absolute -top-3 left-full z-10 ml-1")
                    hidden=move || !open.get()
                >
                    <MenuLevel items=items path=path depth=depth + 1 close_all=close_all />
                </div>
            }
            .into_any()
        }
        other => leaf_item(other, close_all).into_any(),
    }
}

/// The button that opens a [`MenuItem::Submenu`]: hover, a click, or
/// ArrowRight opens it; a chevron shows on the right.
#[component]
fn SubmenuTrigger(
    label: &'static str,
    icon: Option<IconName>,
    #[prop(into)] open: Signal<bool>,
    on_open: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="menuitem"
            aria-haspopup="true"
            aria-expanded=move || open.get().to_string()
            class=classes(Variant::Nav, Size::Md, "w-full")
            on:click=move |_| on_open.run(())
            on:pointerenter=move |_| on_open.run(())
            on:keydown=move |ev| {
                if ev.key() == "ArrowRight" {
                    on_open.run(());
                    ev.stop_propagation();
                }
            }
        >
            {item_row(icon, label, view! { <Icon name=IconName::ChevronRight size=14 /> })}
        </button>
    }
}

// --- Shared: Link, Action and Choice rows ------------------------------

/// A [`MenuItem::Link`], [`MenuItem::Action`], [`MenuItem::Choice`] or
/// [`MenuItem::Custom`] as one row; used by both the desktop and the mobile
/// rendering (a [`MenuItem::Submenu`] is handled by its own caller — it
/// behaves differently on each).
pub(crate) fn leaf_item(item: MenuItem, close_all: Callback<()>) -> impl IntoView {
    match item {
        MenuItem::Link { label, icon, href } => view! {
            <a href=href role="menuitem" class=classes(Variant::Nav, Size::Md, "w-full")>
                {item_row(icon, label, ())}
            </a>
        }
        .into_any(),
        MenuItem::Action {
            label,
            icon,
            on_select,
        } => view! {
            <button
                type="button"
                role="menuitem"
                class=classes(Variant::Nav, Size::Md, "w-full")
                on:click=move |_| {
                    on_select.run(());
                    close_all.run(());
                }
            >
                {item_row(icon, label, ())}
            </button>
        }
        .into_any(),
        MenuItem::Choice {
            label,
            icon,
            selected,
            on_select,
        } => view! {
            <button
                type="button"
                role="menuitemradio"
                aria-checked=move || selected.get().to_string()
                class=classes(Variant::Nav, Size::Md, "w-full")
                on:click=move |_| on_select.run(())
            >
                {item_row(icon, label, view! { <CheckMark shown=selected /> })}
            </button>
        }
        .into_any(),
        MenuItem::Custom(render) => {
            view! { <div class="px-2 py-1.5">{render.run()}</div> }.into_any()
        }
        MenuItem::Submenu { .. } => {
            unreachable!("a Submenu is rendered by its caller, never through leaf_item")
        }
    }
}

// --- Mobile: `DrillMenu`, one level at a time --------------------------

/// The items of a menu, inline: choosing a [`MenuItem::Submenu`] replaces
/// the list with its items, with a "Back" row above them — the mobile
/// rendering (no hover, no side-by-side submenus).
#[component]
pub fn DrillMenu(
    /// The items, top to bottom.
    items: Vec<MenuItem>,
) -> impl IntoView {
    let path: Path = RwSignal::new(Vec::new());
    let close_all = Callback::new(move |()| path.set(Vec::new()));
    view! {
        <div role="menu" class="flex flex-col gap-0.5">
            <BackRow path=path />
            {move || {
                items_at(&items, &path.get())
                    .to_vec()
                    .into_iter()
                    .enumerate()
                    .map(|(index, item)| {
                        view! { <DrillItem item=item path=path index=index close_all=close_all /> }
                    })
                    .collect_view()
            }}
        </div>
    }
}

/// The "Back" row of a [`DrillMenu`]: shown only once drilled into a
/// submenu, it drops the deepest level of `path`. Its label is the
/// library's own string, in the current language.
#[component]
pub(crate) fn BackRow(path: Path) -> impl IntoView {
    let t = strings();
    view! {
        <button
            type="button"
            class=classes(Variant::Nav, Size::Md, "w-full")
            hidden=move || path.get().is_empty()
            on:click=move |_| {
                path.update(|p| {
                    p.pop();
                });
            }
        >
            {item_row(Some(IconName::ArrowLeft), move || td_string!(t.locale(), back), ())}
        </button>
    }
}

/// One item of a [`DrillMenu`]'s current level. A [`MenuItem::Submenu`]
/// drills in (pushes `index`) instead of opening a nested panel.
#[component]
fn DrillItem(item: MenuItem, path: Path, index: usize, close_all: Callback<()>) -> impl IntoView {
    match item {
        MenuItem::Submenu { label, icon, .. } => view! {
            <button
                type="button"
                role="menuitem"
                aria-haspopup="true"
                class=classes(Variant::Nav, Size::Md, "w-full")
                on:click=move |_| {
                    path.update(|p| p.push(index));
                }
            >
                {item_row(icon, label, view! { <Icon name=IconName::ChevronRight size=14 /> })}
            </button>
        }
        .into_any(),
        other => leaf_item(other, close_all).into_any(),
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::button::Disclosed;
    use crate::language::{Language, language_signal};

    fn theme_items() -> Vec<MenuItem> {
        let selected = RwSignal::new(0_u8);
        vec![
            MenuItem::Submenu {
                label: "Theme",
                icon: Some(IconName::Sun),
                items: [(0_u8, "System"), (1, "Light"), (2, "Dark")]
                    .into_iter()
                    .map(|(value, label)| MenuItem::Choice {
                        label,
                        icon: None,
                        selected: Signal::derive(move || selected.get() == value),
                        on_select: Callback::new(move |()| selected.set(value)),
                    })
                    .collect(),
            },
            MenuItem::Action {
                label: "Say hello",
                icon: None,
                on_select: Callback::new(|()| {}),
            },
        ]
    }

    #[test]
    fn items_at_walks_submenus_and_falls_back_on_a_bad_path() {
        let items = theme_items();
        assert_eq!(items_at(&items, &[]).len(), 2);
        assert_eq!(items_at(&items, &[0]).len(), 3);
        assert_eq!(
            items_at(&items, &[0, 9]).len(),
            3,
            "an out-of-range index stays at its level"
        );
        assert_eq!(
            items_at(&items, &[1]).len(),
            2,
            "an Action is not a Submenu: no deeper level"
        );
    }

    #[test]
    fn the_menu_is_a_popover_of_menu_roles() {
        let html = Owner::new().with(|| {
            let controls = Disclosed::new("pc-settings", RwSignal::new(true));
            view! { <Menu controls=controls aria_label="Settings" items=theme_items() /> }.to_html()
        });
        assert!(html.contains(r#"role="dialog""#), "{html}");
        assert_eq!(
            html.matches(r#"role="menu""#).count(),
            2,
            "the top level and the Theme submenu's, the latter hidden: {html}"
        );
        assert!(html.contains(r#"role="menuitem""#), "{html}");
        assert!(html.contains(r#"aria-haspopup="true""#), "{html}");
    }

    #[test]
    fn a_submenu_panel_opens_right_aligned_and_sized_to_its_widest_row() {
        let html = Owner::new().with(|| {
            let controls = Disclosed::new("pc-settings", RwSignal::new(true));
            view! { <Menu controls=controls aria_label="Settings" items=theme_items() /> }.to_html()
        });
        assert!(
            html.contains("absolute -top-3 left-full"),
            "opens to the right, aligned with its entry by default: {html}"
        );
        assert!(
            html.contains("w-max min-w-56"),
            "sized to its widest row, never a fixed width rows can overflow: {html}"
        );
    }

    #[test]
    fn a_choice_reports_aria_checked_from_its_signal() {
        let html = Owner::new().with(|| {
            let selected = Signal::derive(|| true);
            let item = MenuItem::Choice {
                label: "Dark",
                icon: None,
                selected,
                on_select: Callback::new(|()| {}),
            };
            leaf_item(item, Callback::new(|()| {})).into_any().to_html()
        });
        assert!(html.contains(r#"role="menuitemradio""#), "{html}");
        assert!(html.contains(r#"aria-checked="true""#), "{html}");
    }

    #[test]
    fn the_back_row_speaks_the_current_language() {
        for (language, back) in [
            (Language::En, "Back"),
            (Language::Fr, "Retour"),
            (Language::Vi, "Quay lại"),
        ] {
            let html = Owner::new().with(|| {
                language_signal().set(language);
                view! { <DrillMenu items=theme_items() /> }.to_html()
            });
            assert!(
                html.contains(&format!("<span>{back}</span>")),
                "{language}: {html}"
            );
        }
    }

    #[test]
    fn the_drill_menu_starts_at_the_top_with_back_hidden() {
        let html = Owner::new().with(|| view! { <DrillMenu items=theme_items() /> }.to_html());
        assert!(html.contains("Theme"), "{html}");
        assert!(html.contains("Say hello"), "{html}");
        let back = &html[..html.find("Back").unwrap()];
        assert!(
            back[back.rfind("<button").unwrap()..].contains("hidden"),
            "{html}"
        );
    }
}
