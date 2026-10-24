//! [`Callout`] (079): a boxed message in a [`Tone`] — "this secret will not
//! be shown again", "rotating replaces the staged secret". Its icon follows
//! the tone; when dismissible, its close control is the library's
//! [`Button`] (variant `Icon`), named by the library's own string.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Button, Size, Variant};
use crate::catalogue::strings;
use crate::icon::{Icon, IconName};
use crate::tone::{Tone, callout_classes, text_classes};

fn icon_of(tone: Tone) -> IconName {
    match tone {
        Tone::Success => IconName::Check,
        Tone::Warning | Tone::Danger => IconName::Warning,
        _ => IconName::Info,
    }
}

/// A message in a box of its tone, with an optional title and close.
#[component]
pub fn Callout(
    /// What the message means; informational by default.
    #[prop(optional)]
    tone: Option<Tone>,
    /// A bold first line.
    #[prop(optional, into)]
    title: Option<String>,
    /// Shows a close control: a click hides the callout, then runs this. A
    /// closed callout stays closed: a new message is a new callout (key it
    /// by the message where one instance would be reused).
    #[prop(optional, into)]
    on_dismiss: Option<Callback<()>>,
    /// The message.
    children: Children,
) -> impl IntoView {
    let tone = tone.unwrap_or(Tone::Info);
    let role = if matches!(tone, Tone::Warning | Tone::Danger) {
        "alert"
    } else {
        "status"
    };
    let class = format!(
        "flex items-start gap-3 rounded-xl border px-4 py-3 text-sm {}",
        callout_classes(tone)
    );
    let icon_class = format!("mt-0.5 shrink-0 {}", text_classes(tone));
    let title = title.map(|t| view! { <strong class="font-semibold text-heading">{t}</strong> });
    // Closing hides the box itself (review 079: a close that only ran the
    // caller's callback left a dead button wherever the caller did not
    // hide it); the callback is told after.
    let open = RwSignal::new(true);
    let dismiss = on_dismiss.map(|run| {
        let t = strings();
        view! {
            <Button
                variant=Variant::Icon
                size=Size::Sm
                label=td_string!(t.locale(), callout_dismiss).to_string()
                layout="-my-1 -mr-2 shrink-0"
                on_click=move |_| {
                    open.set(false);
                    run.run(());
                }
            >
                <Icon name=IconName::Close size=14 />
            </Button>
        }
    });
    view! {
        <div class=class role=role hidden=move || !open.get()>
            <span class=icon_class aria-hidden="true">
                <Icon name=icon_of(tone) size=18 />
            </span>
            <div class="flex min-w-0 flex-1 flex-col gap-0.5">{title} <div>{children()}</div></div>
            {dismiss}
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn html(view: impl FnOnce() -> AnyView) -> String {
        Owner::new().with(|| view().to_html())
    }

    #[test]
    fn a_warning_is_an_alert_in_its_tone() {
        let out = html(|| {
            view! {
                <Callout tone=Tone::Warning title="Once">
                    "Copy it now"
                </Callout>
            }
            .into_any()
        });
        assert!(
            out.contains(r#"role="alert""#) && out.contains("bg-warning-soft"),
            "{out}"
        );
        assert!(out.contains("Once") && out.contains("Copy it now"), "{out}");
    }

    #[test]
    fn an_info_is_a_status_and_has_no_close_unless_asked() {
        let out = html(|| view! { <Callout>"FYI"</Callout> }.into_any());
        assert!(
            out.contains(r#"role="status""#) && !out.contains("<button"),
            "{out}"
        );
    }

    #[test]
    fn a_dismissible_callout_closes_through_a_named_library_button() {
        let out =
            html(|| view! { <Callout on_dismiss=Callback::new(|()| ())>"x"</Callout> }.into_any());
        assert!(
            out.contains("<button") && out.contains("aria-label="),
            "{out}"
        );
    }
}
