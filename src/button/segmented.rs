//! An inline row of [`RadioButton`]s, `role="radiogroup"`: a few closed
//! choices shown side by side, e.g. the reading theme (Light · Dark ·
//! System).

use leptos::prelude::*;

use crate::button::{RadioButton, Size, Variant};

/// One option of a [`Segmented`] control: what it shows, and the value it
/// chooses.
pub struct SegmentedOption<T> {
    /// Shown on the option's own button.
    pub label: &'static str,
    /// The value this option chooses.
    pub value: T,
}

/// A row of [`RadioButton`]s for one value out of a few, `role="radiogroup"`.
#[component]
pub fn Segmented<T>(
    /// Its name for assistive technology.
    #[prop(into)]
    aria_label: String,
    /// The options, in order.
    options: Vec<SegmentedOption<T>>,
    /// The group's current value.
    #[prop(into)]
    value: Signal<T>,
    /// Runs with the chosen value when an option is picked.
    #[prop(into)]
    on_change: Callback<T>,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    view! {
        <div role="radiogroup" aria-label=aria_label class="inline-flex gap-1 rounded-full border border-line bg-chip p-1">
            {options
                .into_iter()
                .map(|SegmentedOption { label, value: option }| {
                    view! {
                        <RadioButton
                            checked=Signal::derive(move || value.get() == option)
                            on_select=move |_| on_change.run(option)
                            variant=Variant::Chip
                            size=Size::Sm
                        >
                            {label}
                        </RadioButton>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn options() -> Vec<SegmentedOption<u8>> {
        vec![
            SegmentedOption {
                label: "System",
                value: 0,
            },
            SegmentedOption {
                label: "Light",
                value: 1,
            },
            SegmentedOption {
                label: "Dark",
                value: 2,
            },
        ]
    }

    #[test]
    fn it_is_a_radiogroup_of_radios_named_and_marked() {
        let html = Owner::new().with(|| {
            view! {
                <Segmented
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
        assert!(html.contains("System"), "{html}");
        assert!(html.contains("Light"), "{html}");
        assert!(html.contains("Dark"), "{html}");
    }

    #[test]
    fn exactly_the_current_value_is_checked() {
        let html = Owner::new().with(|| {
            view! {
                <Segmented
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
        let dark_button_start = html[..html.find("Dark").unwrap()].rfind("<button").unwrap();
        let dark_button_end = html[dark_button_start..].find('>').unwrap() + dark_button_start;
        assert!(
            html[dark_button_start..dark_button_end].contains(r#"aria-checked="true""#),
            "Dark is the checked one: {html}"
        );
    }
}
