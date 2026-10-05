use super::*;

fn html(view: impl FnOnce() -> AnyView) -> String {
    Owner::new().with(|| view().to_html())
}

#[test]
fn a_button_runs_an_action_and_is_named() {
    let out = html(|| {
        view! {
            <Button variant=Variant::Icon label="Close" on_click=|_| ()>
                "×"
            </Button>
        }
        .into_any()
    });
    assert!(out.contains(r#"type="button""#), "{out}");
    assert!(out.contains(r#"aria-label="Close""#), "{out}");
    assert!(out.contains("size-11"), "medium icon: {out}");
    assert!(!out.contains("disabled=\"\""), "enabled by default: {out}");
}

#[test]
fn a_disabled_button_says_so() {
    let out = html(|| {
        view! {
            <Button disabled=Signal::derive(|| true) on_click=|_| ()>
                "Send"
            </Button>
        }
        .into_any()
    });
    assert!(out.contains("disabled"), "{out}");
}

#[test]
fn a_disclosure_button_names_and_follows_what_it_controls() {
    let out = html(|| {
        let d = Disclosed::new("pc-rail", RwSignal::new(true));
        view! {
            <DisclosureButton controls=d label_closed="Open" label_open="Close">
                "x"
            </DisclosureButton>
        }
        .into_any()
    });
    assert!(out.contains(r#"aria-controls="pc-rail""#), "{out}");
    assert!(out.contains(r#"aria-expanded="true""#), "{out}");
    assert!(out.contains(r#"aria-label="Close""#), "{out}");
}

#[test]
fn a_toggle_button_is_pressed_or_not() {
    let out = html(|| {
        view! { <ToggleButton pressed=RwSignal::new(true)>"Sepia"</ToggleButton> }.into_any()
    });
    assert!(out.contains(r#"aria-pressed="true""#), "{out}");
}

#[test]
fn a_radio_button_says_checked_and_is_named() {
    let out = html(|| {
        view! {
            <RadioButton checked=true label="Light" on_select=|_| ()>
                "Light"
            </RadioButton>
        }
        .into_any()
    });
    assert!(out.contains(r#"role="radio""#), "{out}");
    assert!(out.contains(r#"aria-checked="true""#), "{out}");
    assert!(out.contains(r#"aria-label="Light""#), "{out}");
    let unchecked = html(|| {
        view! {
            <RadioButton checked=false on_select=|_| ()>
                "Dark"
            </RadioButton>
        }
        .into_any()
    });
    assert!(unchecked.contains(r#"aria-checked="false""#), "{unchecked}");
}

#[test]
fn a_link_button_is_a_real_link_marked_current() {
    let current = html(|| {
        view! {
            <LinkButton href="https://palicanon.app" current=true>
                "Search"
            </LinkButton>
        }
        .into_any()
    });
    assert!(current.starts_with("<a"), "{current}");
    assert!(
        current.contains(r#"href="https://palicanon.app""#),
        "{current}"
    );
    assert!(current.contains(r#"aria-current="page""#), "{current}");
    let other = html(|| view! { <LinkButton href="/x">"X"</LinkButton> }.into_any());
    assert!(!other.contains("aria-current"), "{other}");
}
