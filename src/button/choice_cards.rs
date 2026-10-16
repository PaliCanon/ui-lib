//! [`ChoiceCards`]: a row of picture cards for one of a few choices,
//! `role="radiogroup"` — the classic Light/Dark/System style picker
//! ([`crate::theme::ThemePicker`] is built on it). Unlike
//! [`Segmented`](crate::button::Segmented)'s plain chips, each option shows
//! a picture (CSS-drawn, or an image — whatever the caller puts in the
//! slot) with its label underneath; the chosen one gets the gold ring and a
//! small check. A `<button>` already answers Space and Enter natively; this
//! component only adds the arrow keys, which move the choice — and the
//! focus with it — to the next or previous option, wrapping around: the
//! same "one tab stop" roving behaviour a native `<input type="radio">`
//! group has (only the chosen option's button is itself tabbable).

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// One option of a [`ChoiceCards`] group.
pub struct ChoiceCardOption<T> {
    /// Shown above the label: whatever the caller draws for this option
    /// (CSS, an image, anything) — the "picture slot".
    pub picture: ViewFn,
    /// Shown under the picture.
    pub label: &'static str,
    /// The value this option chooses.
    pub value: T,
}

/// A row of picture cards for one value out of a few, `role="radiogroup"`:
/// small fixed thumbnails (4.5rem wide, a 3rem-tall picture, the label
/// under it) that sit in the middle of the group's width and wrap on a
/// narrow phone instead of overflowing. They never grow with the room
/// around them, so three of them fit on one row even in a 320px-wide dialog.
#[component]
pub fn ChoiceCards<T>(
    /// Its name for assistive technology.
    #[prop(into)]
    aria_label: String,
    /// The options, in order.
    options: Vec<ChoiceCardOption<T>>,
    /// The group's current value.
    #[prop(into)]
    value: Signal<T>,
    /// Runs with the chosen value when an option is picked (a click, or an
    /// arrow key).
    #[prop(into)]
    on_change: Callback<T>,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    let values: Vec<T> = options.iter().map(|option| option.value).collect();
    view! {
        <div
            role="radiogroup"
            aria-label=aria_label
            class="flex flex-wrap justify-center gap-3"
            on:keydown=move |ev| choice_cards_keydown(&ev, &values, on_change)
        >
            {options
                .into_iter()
                .map(|option| {
                    view! { <ChoiceCard option=option value=value on_change=on_change /> }
                })
                .collect_view()}
        </div>
    }
}

/// One card of a [`ChoiceCards`] group: the picture (overlaid with a check
/// mark once chosen), then the label underneath.
#[component]
fn ChoiceCard<T>(
    option: ChoiceCardOption<T>,
    value: Signal<T>,
    on_change: Callback<T>,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    let ChoiceCardOption {
        picture,
        label,
        value: option_value,
    } = option;
    let checked = Signal::derive(move || value.get() == option_value);
    view! {
        <button
            type="button"
            role="radio"
            class="flex w-18 shrink-0 flex-col items-center gap-1.5 rounded-xl p-1 text-center \
            focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold"
            aria-checked=move || checked.get().to_string()
            tabindex=move || if checked.get() { "0" } else { "-1" }
            on:click=move |_| on_change.run(option_value)
        >
            <span class=move || frame_class(
                checked.get(),
            )>
                {picture.run()}
                <span
                    class="absolute top-1 right-1 flex size-4 items-center justify-center rounded-full bg-gold text-on-gold"
                    class:hidden=move || !checked.get()
                    aria-hidden="true"
                >
                    <Icon name=IconName::Check size=11 />
                </span>
            </span>
            <span class=move || label_class(checked.get())>{label}</span>
        </button>
    }
}

/// The picture frame's classes: the gold border and ring once chosen, the
/// plain line otherwise. One set or the other, never both: Tailwind orders
/// same-specificity utilities itself, so a class added on top of the base
/// one may lose to it.
fn frame_class(checked: bool) -> &'static str {
    if checked {
        "relative block h-12 w-full overflow-hidden rounded-lg border-2 border-gold \
         outline-2 outline-offset-2 outline-gold"
    } else {
        "relative block h-12 w-full overflow-hidden rounded-lg border-2 border-line"
    }
}

/// The label's classes: the heading colour once chosen, muted otherwise
/// (one or the other, as in [`frame_class`]).
fn label_class(checked: bool) -> &'static str {
    if checked {
        "text-xs text-heading"
    } else {
        "text-xs text-muted"
    }
}

/// `ArrowLeft`/`ArrowRight` (and `ArrowUp`/`ArrowDown`) pick the previous or
/// next option and move focus there with it, wrapping around — hydrate
/// only: it needs the DOM to find which option has focus and to move focus
/// to the next one. Space and Enter need nothing here: a `<button>` answers
/// both natively.
#[cfg(feature = "browser")]
fn choice_cards_keydown<T: Copy>(
    ev: &leptos::ev::KeyboardEvent,
    values: &[T],
    on_change: Callback<T>,
) {
    use wasm_bindgen::JsCast;
    let forward = match ev.key().as_str() {
        "ArrowRight" | "ArrowDown" => true,
        "ArrowLeft" | "ArrowUp" => false,
        _ => return,
    };
    let Some(group) = ev
        .current_target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Ok(list) = group.query_selector_all(":scope > [role='radio']") else {
        return;
    };
    let len = list.length();
    if len == 0 || len as usize != values.len() {
        return;
    }
    let active = document().active_element();
    let current = active
        .as_ref()
        .and_then(|a| (0..len).find(|&i| list.item(i).is_some_and(|n| n.is_same_node(Some(a)))));
    let index = current.unwrap_or(0);
    let next = if forward {
        (index + 1) % len
    } else {
        (index + len - 1) % len
    };
    on_change.run(values[next as usize]);
    if let Some(el) = list.item(next) {
        let _ = el.unchecked_ref::<web_sys::HtmlElement>().focus();
    }
    ev.prevent_default();
}

#[cfg(not(feature = "browser"))]
fn choice_cards_keydown<T: Copy>(
    _ev: &leptos::ev::KeyboardEvent,
    _values: &[T],
    _on_change: Callback<T>,
) {
}

#[cfg(all(test, feature = "ssr"))]
pub(crate) mod tests {
    use super::*;

    fn picture(text: &'static str) -> ViewFn {
        (move || view! { <span>{text}</span> }.into_any()).into()
    }

    fn options() -> Vec<ChoiceCardOption<u8>> {
        vec![
            ChoiceCardOption {
                picture: picture("system-pic"),
                label: "System",
                value: 0,
            },
            ChoiceCardOption {
                picture: picture("light-pic"),
                label: "Light",
                value: 1,
            },
            ChoiceCardOption {
                picture: picture("dark-pic"),
                label: "Dark",
                value: 2,
            },
        ]
    }

    #[test]
    fn it_is_a_radiogroup_of_cards_named_and_marked() {
        let html = Owner::new().with(|| {
            view! {
                <ChoiceCards
                    aria_label="Theme"
                    options=options()
                    value=Signal::derive(|| 1_u8)
                    on_change=|_| ()
                />
            }
            .to_html()
        });
        assert!(html.contains(r#"role="radiogroup""#), "{html}");
        assert!(html.contains(r#"aria-label="Theme""#), "{html}");
        assert_eq!(html.matches(r#"role="radio""#).count(), 3, "{html}");
        assert!(html.contains("system-pic"), "{html}");
        assert!(html.contains("light-pic"), "{html}");
        assert!(html.contains("dark-pic"), "{html}");
        assert!(html.contains("System"), "{html}");
        assert!(html.contains("Light"), "{html}");
        assert!(html.contains("Dark"), "{html}");
    }

    #[test]
    fn exactly_the_current_value_is_checked() {
        let html = Owner::new().with(|| {
            view! {
                <ChoiceCards
                    aria_label="Theme"
                    options=options()
                    value=Signal::derive(|| 2_u8)
                    on_change=|_| ()
                />
            }
            .to_html()
        });
        assert_eq!(html.matches(r#"aria-checked="true""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"aria-checked="false""#).count(), 2, "{html}");
        let dark_start = html[..html.find("dark-pic").unwrap()]
            .rfind("<button")
            .unwrap();
        let dark_end = html[dark_start..].find('>').unwrap() + dark_start;
        assert!(
            html[dark_start..dark_end].contains(r#"aria-checked="true""#),
            "Dark is the checked one: {html}"
        );
    }

    #[test]
    fn only_the_checked_card_is_a_tab_stop() {
        let html = Owner::new().with(|| {
            view! {
                <ChoiceCards
                    aria_label="Theme"
                    options=options()
                    value=Signal::derive(|| 0_u8)
                    on_change=|_| ()
                />
            }
            .to_html()
        });
        assert_eq!(html.matches(r#"tabindex="0""#).count(), 1, "{html}");
        assert_eq!(html.matches(r#"tabindex="-1""#).count(), 2, "{html}");
    }

    #[test]
    fn only_the_checked_card_shows_its_check_mark() {
        let html = Owner::new().with(|| {
            view! {
                <ChoiceCards
                    aria_label="Theme"
                    options=options()
                    value=Signal::derive(|| 1_u8)
                    on_change=|_| ()
                />
            }
            .to_html()
        });
        // The check mark's own span (not the decorative icon's `aria-hidden`)
        // carries the `hidden` class for every option but the checked one.
        assert_eq!(
            html.matches("text-on-gold\">").count(),
            1,
            "one check mark shown: {html}"
        );
        assert_eq!(
            html.matches("text-on-gold hidden\">").count(),
            2,
            "hidden on the two unchecked ones: {html}"
        );
    }

    #[test]
    fn only_the_checked_card_has_the_gold_border_ring_and_heading_label() {
        let html = Owner::new().with(|| {
            view! {
                <ChoiceCards
                    aria_label="Theme"
                    options=options()
                    value=Signal::derive(|| 1_u8)
                    on_change=|_| ()
                />
            }
            .to_html()
        });
        // One class or the other, never both: Tailwind orders same-specificity
        // utilities itself, so a second class on top of the base one may lose.
        let checked = class_tokens(card(&html, "light-pic"));
        for class in ["border-gold", "outline-gold", "text-heading"] {
            assert!(checked.contains(&class), "checked has {class}: {html}");
        }
        for class in ["border-line", "text-muted"] {
            assert!(!checked.contains(&class), "checked lacks {class}: {html}");
        }
        for marker in ["system-pic", "dark-pic"] {
            let unchecked = class_tokens(card(&html, marker));
            for class in ["border-line", "text-muted"] {
                assert!(unchecked.contains(&class), "{marker} has {class}: {html}");
            }
            for class in ["border-gold", "outline-gold", "text-heading"] {
                assert!(
                    !unchecked.contains(&class),
                    "{marker} lacks {class}: {html}"
                );
            }
        }
    }

    #[test]
    fn cards_are_small_fixed_thumbnails_that_never_grow() {
        let html = Owner::new().with(|| {
            view! {
                <ChoiceCards
                    aria_label="Theme"
                    options=options()
                    value=Signal::derive(|| 1_u8)
                    on_change=|_| ()
                />
            }
            .to_html()
        });
        // 4.5rem wide, a 3rem-tall picture: three cards and two gaps make
        // 15rem, inside the Settings dialog's 280px content box at 320px.
        for marker in ["system-pic", "light-pic", "dark-pic"] {
            let tokens = class_tokens(card(&html, marker));
            for class in ["w-18", "shrink-0", "h-12"] {
                assert!(tokens.contains(&class), "{marker} has {class}: {html}");
            }
            for class in ["flex-1", "aspect-[3/4]", "min-w-[5.5rem]"] {
                assert!(!tokens.contains(&class), "{marker} lacks {class}: {html}");
            }
        }
    }

    /// 042 (user, 2026-10-05): the cards sit in the middle of the group's
    /// width (the Settings dialog's section, the gallery's Theme page)
    /// and wrap on a narrow phone instead of overflowing.
    #[test]
    fn the_cards_are_centred_in_their_group_and_wrap() {
        let html = Owner::new().with(|| {
            view! {
                <ChoiceCards
                    aria_label="Theme"
                    options=options()
                    value=Signal::derive(|| 1_u8)
                    on_change=|_| ()
                />
            }
            .to_html()
        });
        let start = html.find(r#"role="radiogroup""#).expect(&html);
        let tag = &html[start..start + html[start..].find('>').expect(&html)];
        assert!(tag.contains("justify-center"), "{tag}");
        assert!(tag.contains("flex-wrap"), "{tag}");
    }

    /// The rendered `<button>…</button>` of the card holding `marker`.
    pub(crate) fn card<'a>(html: &'a str, marker: &str) -> &'a str {
        let at = html
            .find(marker)
            .unwrap_or_else(|| panic!("{marker}: {html}"));
        let start = html[..at].rfind("<button").expect("inside a button");
        let end = at + html[at..].find("</button>").expect("closed button");
        &html[start..end]
    }

    /// Every class token of every `class="…"` attribute in `html`.
    pub(crate) fn class_tokens(html: &str) -> Vec<&str> {
        html.split(r#"class=""#)
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .collect()
    }
}
