//! [`Badge`]: a short status or label in a pill (079) — a role, a state,
//! "Shared", "Rotating secret". Its colour is a [`Tone`], never a colour.

use leptos::prelude::*;

use crate::tone::{Tone, badge_classes, dot_classes};

/// A pill holding a short label. `dot` leads it with a round mark of the
/// same tone (a live state: Enabled, Disabled); `outline` draws only its
/// edge (a quieter attribute beside a filled one).
#[component]
pub fn Badge(
    /// What it means; neutral by default.
    #[prop(optional)]
    tone: Tone,
    /// Leads the label with a round mark of the same tone.
    #[prop(optional)]
    dot: bool,
    /// Only the edge, no fill.
    #[prop(optional)]
    outline: bool,
    /// The label: a word or two.
    children: Children,
) -> impl IntoView {
    let class = format!(
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap {}",
        badge_classes(tone, outline)
    );
    let mark = dot.then(|| {
        let dot_class = format!("size-1.5 rounded-full {}", dot_classes(tone));
        view! { <span class=dot_class aria-hidden="true"></span> }
    });
    view! { <span class=class>{mark} {children()}</span> }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn html(view: impl FnOnce() -> AnyView) -> String {
        Owner::new().with(|| view().to_html())
    }

    #[test]
    fn a_badge_is_a_pill_in_its_tone() {
        let out = html(|| view! { <Badge tone=Tone::Success>"Enabled"</Badge> }.into_any());
        assert!(
            out.contains("rounded-full") && out.contains("text-success"),
            "{out}"
        );
        assert!(out.contains("Enabled"), "{out}");
        assert!(!out.contains("aria-hidden"), "no dot unless asked: {out}");
    }

    #[test]
    fn a_dot_leads_the_label_and_is_hidden_from_readers() {
        let out = html(|| {
            view! {
                <Badge tone=Tone::Danger dot=true>
                    "Disabled"
                </Badge>
            }
            .into_any()
        });
        assert!(
            out.contains(r#"aria-hidden="true""#) && out.contains("bg-danger"),
            "{out}"
        );
    }

    #[test]
    fn an_outline_badge_has_an_edge_and_no_fill() {
        let out = html(|| view! { <Badge outline=true>"Private"</Badge> }.into_any());
        assert!(
            out.contains("border-line") && !out.contains("bg-chip"),
            "{out}"
        );
    }
}
