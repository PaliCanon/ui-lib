//! A generic modal dialog over a dimmed, blurred backdrop: opened by any
//! [`Disclosure`], like [`crate::popover::Popover`] — a centred panel with a
//! pinned header (title, close ✕) and an optional pinned footer, the body
//! between them the only part that scrolls. Below `sm`, the panel becomes a
//! full-screen sheet (no margins; nothing of the backdrop shows behind it
//! anyway). At most one `Dialog` is open at a time: opening a second one
//! closes the first immediately, with a single focus move into the new one
//! — unless the first is not `dismissable`, which refuses the second (it
//! stays closed) until its own action closes it (see the internal
//! `focus_trap` module).
//!
//! First used by [`crate::shell::AppShell`]'s Settings.

use leptos::prelude::*;
use leptos_i18n::td_string;

use crate::button::{Button, Disclosure, Variant};
use crate::catalogue::strings;
use crate::focus_trap;
use crate::icon::{Icon, IconName};

/// The panel's own look: card colour, a full-bleed sheet below `sm`,
/// centred and bounded above it.
const PANEL: &str = "flex h-full w-full flex-col overflow-hidden bg-card \
    sm:h-auto sm:max-h-[85vh] sm:w-full sm:max-w-[40rem] sm:rounded-2xl \
    sm:border sm:border-line sm:shadow-lg";

/// A modal dialog controlled by a [`Disclosure`]: a centred panel (a
/// full-screen sheet below `sm`) over a dimmed, blurred backdrop.
#[component]
pub fn Dialog<D: Disclosure>(
    /// What opens and closes it.
    controls: D,
    /// Its header title; also its accessible name (`aria-labelledby`).
    /// A plain string, or a signal to follow: a title in the current
    /// language changes in place, the dialog stays mounted.
    #[prop(into)]
    title: Signal<String>,
    /// Backdrop click and Escape close it, the header shows a close
    /// button, and another `Dialog` opening replaces it, while this is true
    /// (the default). Set to `false` for a required confirmation: only the
    /// caller's own action (in `footer`) can close it, and another `Dialog`
    /// asking to open meanwhile is refused (it stays closed).
    #[prop(default = true)]
    dismissable: bool,
    /// Pinned under the body, e.g. Primary and Ghost [`Button`]s.
    #[prop(optional, into)]
    footer: Option<ViewFn>,
    /// Layout classes added to the panel (e.g. a narrower `max-w`) — never
    /// colours.
    #[prop(optional, into)]
    layout: String,
    /// The body: the only part that scrolls, e.g. one or more
    /// [`DialogSection`]s.
    children: Children,
) -> impl IntoView {
    focus_trap::manage(controls, dismissable);
    close_on_escape(controls, dismissable);
    let title_id = format!("{}-title", controls.id());
    let header_title_id = title_id.clone();
    let panel_class = format!("{PANEL} {layout}");
    view! {
        <div class="fixed inset-0 z-50" hidden=move || !controls.is_open()>
            <Backdrop controls=controls dismissable=dismissable />
            <div class="relative z-10 flex h-full items-center justify-center sm:p-4">
                <div
                    id=controls.id()
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby=title_id
                    class=panel_class
                >
                    <Header
                        title=title
                        title_id=header_title_id
                        controls=controls
                        dismissable=dismissable
                    />
                    <div class="min-h-0 flex-1 overflow-y-auto px-5 py-4">{children()}</div>
                    {footer.map(|footer| view! { <Footer>{footer.run()}</Footer> })}
                </div>
            </div>
        </div>
    }
}

/// The dimmed, blurred scrim behind the panel; a click closes the dialog
/// while `dismissable`.
#[component]
fn Backdrop<D: Disclosure>(controls: D, dismissable: bool) -> impl IntoView {
    view! {
        <div
            class="fixed inset-0 bg-black/50 backdrop-blur-sm"
            aria-hidden="true"
            on:click=move |_| {
                if dismissable {
                    controls.set_open(false);
                }
            }
        ></div>
    }
}

/// The pinned header: the title, and a close ✕ ([`Variant::Icon`]) unless
/// `dismissable` is false — a required confirmation has no shortcut out.
#[component]
fn Header<D: Disclosure>(
    title: Signal<String>,
    title_id: String,
    controls: D,
    dismissable: bool,
) -> impl IntoView {
    view! {
        <div class="flex shrink-0 items-center justify-between gap-3 border-b border-line px-5 py-4">
            <h2 id=title_id class="font-serif text-lg text-heading">
                {move || title.get()}
            </h2>
            {dismissable.then(|| view! { <CloseButton controls=controls /> })}
        </div>
    }
}

/// The header's close ✕, named `close` in the current language: re-rendered
/// whole on a switch, since [`Button`]'s `label` is a plain string.
#[component]
fn CloseButton<D: Disclosure>(controls: D) -> impl IntoView {
    let t = strings();
    move || {
        view! {
            <Button
                variant=Variant::Icon
                label=td_string!(t.locale(), close)
                on_click=move |_| controls.set_open(false)
            >
                <Icon name=IconName::Close />
            </Button>
        }
    }
}

/// The pinned footer: the caller's actions, right-aligned.
#[component]
fn Footer(children: Children) -> impl IntoView {
    view! {
        <div class="flex shrink-0 items-center justify-end gap-2 border-t border-line px-5 py-4">
            {children()}
        </div>
    }
}

/// The browser only: Escape closes `controls` while `dismissable`. There is
/// no window while the server renders.
fn close_on_escape<D: Disclosure>(controls: D, dismissable: bool) {
    #[cfg(feature = "hydrate")]
    {
        let handle = window_event_listener(leptos::ev::keydown, move |ev| {
            if dismissable && controls.is_open() && ev.key() == "Escape" {
                controls.set_open(false);
            }
        });
        // Ends with the dialog's owner: an unmounted dialog must not keep
        // reading its (disposed) `controls` on every Escape.
        on_cleanup(move || handle.remove());
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (controls, dismissable);
}

/// A titled block of a [`Dialog`]'s body: a divider above it (skipped for
/// the first section) then its content.
#[component]
pub fn DialogSection(
    /// The section's own title.
    #[prop(into)]
    title: String,
    /// The section's content.
    children: Children,
) -> impl IntoView {
    view! {
        <section class="mt-4 border-t border-line pt-4 first:mt-0 first:border-t-0 first:pt-0">
            <h3 class="font-serif text-base text-heading">{title}</h3>
            <div class="mt-3">{children()}</div>
        </section>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::button::Disclosed;
    use crate::language::{Language, language_signal};

    fn rendered(id: &'static str, open: bool) -> String {
        Owner::new().with(|| {
            let controls = Disclosed::new(id, RwSignal::new(open));
            view! {
                <Dialog controls=controls title="Settings">
                    <p>"body"</p>
                </Dialog>
            }
            .to_html()
        })
    }

    #[test]
    fn hidden_while_closed_shown_while_open() {
        // `aria-hidden="true"` (the backdrop) and `overflow-hidden` (the
        // panel) both contain "hidden" too, so the wrapper's own boolean
        // `hidden` attribute (always its outermost, first tag) is checked
        // on its own, not as a loose substring of the whole page.
        let closed = rendered("pc-dialog-closed", false);
        assert!(closed.starts_with("<div hidden"), "{closed}");
        let open = rendered("pc-dialog-open", true);
        assert!(open.starts_with("<div class="), "{open}");
        assert!(open.contains("<p>body</p>"), "{open}");
    }

    #[test]
    fn the_panel_is_a_labelled_modal_dialog() {
        let html = rendered("pc-dialog-open", true);
        assert!(html.contains(r#"role="dialog""#), "{html}");
        assert!(html.contains(r#"aria-modal="true""#), "{html}");
        assert!(
            html.contains(r#"aria-labelledby="pc-dialog-open-title""#),
            "{html}"
        );
        assert!(html.contains(r#"id="pc-dialog-open-title""#), "{html}");
        assert!(html.contains("Settings"), "{html}");
    }

    #[test]
    fn the_close_button_shows_by_default() {
        let html = rendered("pc-dialog-open", true);
        assert!(html.contains(r#"aria-label="Close""#), "{html}");
    }

    #[test]
    fn the_close_button_is_named_in_the_current_language() {
        for (language, name) in [(Language::Fr, "Fermer"), (Language::Vi, "Đóng")] {
            let html = Owner::new().with(|| {
                language_signal().set(language);
                let controls = Disclosed::new("pc-dialog-lang", RwSignal::new(true));
                view! {
                    <Dialog controls=controls title="T">
                        <p>"body"</p>
                    </Dialog>
                }
                .to_html()
            });
            let expected = format!(r#"aria-label="{name}""#);
            assert!(html.contains(&expected), "{language:?}: {html}");
        }
    }

    #[test]
    fn a_reactive_title_follows_its_signal_once_built() {
        let html = Owner::new().with(|| {
            let controls = Disclosed::new("pc-dialog-title", RwSignal::new(true));
            let french = RwSignal::new(false);
            let title = Signal::derive(move || {
                if french.get() {
                    "Paramètres"
                } else {
                    "Settings"
                }
                .to_string()
            });
            // Built with one title, rendered after it changed: the same
            // dialog shows the new one, no rebuild needed.
            let dialog = view! {
                <Dialog controls=controls title=title>
                    <p>"body"</p>
                </Dialog>
            };
            french.set(true);
            dialog.to_html()
        });
        assert!(html.contains(">Paramètres</h2>"), "{html}");
        assert!(!html.contains("Settings"), "{html}");
    }

    #[test]
    fn switching_the_language_re_renders_the_close_buttons_name() {
        let html = Owner::new().with(|| {
            let controls = Disclosed::new("pc-dialog-live", RwSignal::new(true));
            // Built in English, rendered after the switch.
            let dialog = view! {
                <Dialog controls=controls title="T">
                    <p>"body"</p>
                </Dialog>
            };
            language_signal().set(Language::Fr);
            dialog.to_html()
        });
        assert!(html.contains(r#"aria-label="Fermer""#), "{html}");
        assert!(!html.contains(r#"aria-label="Close""#), "{html}");
    }

    #[test]
    fn a_non_dismissable_dialog_has_no_close_button() {
        let controls = Disclosed::new("pc-confirm", RwSignal::new(true));
        let html = Owner::new().with(|| {
            view! {
                <Dialog controls=controls title="Confirm" dismissable=false>
                    <p>"Are you sure?"</p>
                </Dialog>
            }
            .to_html()
        });
        assert!(!html.contains(r#"aria-label="Close""#), "{html}");
    }

    #[test]
    fn a_footer_renders_only_when_given() {
        let without = rendered("pc-dialog-open", true);
        assert!(
            !without.contains("border-t border-line px-5 py-4\">"),
            "{without}"
        );

        let controls = Disclosed::new("pc-dialog-footer", RwSignal::new(true));
        let with_footer = Owner::new().with(|| {
            view! {
                <Dialog
                    controls=controls
                    title="Settings"
                    footer=|| view! { <button>"Save"</button> }.into_any()
                >
                    <p>"body"</p>
                </Dialog>
            }
            .to_html()
        });
        assert!(with_footer.contains("Save"), "{with_footer}");
    }

    #[test]
    fn the_body_is_the_only_scroll_region() {
        let html = rendered("pc-dialog-open", true);
        assert_eq!(
            html.matches("overflow-y-auto").count(),
            1,
            "only the body scrolls: {html}"
        );
    }

    #[test]
    fn mobile_is_a_full_screen_sheet_bounded_from_sm_up() {
        let html = rendered("pc-dialog-open", true);
        assert!(html.contains("h-full w-full"), "{html}");
        assert!(html.contains("sm:max-w-[40rem]"), "{html}");
        assert!(html.contains("sm:max-h-[85vh]"), "{html}");
        assert!(html.contains("sm:rounded-2xl"), "{html}");
    }

    #[test]
    fn layout_classes_join_the_panel_look() {
        let controls = Disclosed::new("pc-dialog-layout", RwSignal::new(true));
        let html = Owner::new().with(|| {
            view! {
                <Dialog controls=controls title="Settings" layout="sm:max-w-[28rem]">
                    <p>"x"</p>
                </Dialog>
            }
            .to_html()
        });
        assert!(html.contains("sm:max-w-[28rem]"), "{html}");
    }

    #[test]
    fn a_dialog_section_titles_its_content_and_skips_the_first_divider() {
        let html = Owner::new().with(|| {
            view! {
                <>
                    <DialogSection title="Appearance">
                        <p>"theme"</p>
                    </DialogSection>
                    <DialogSection title="Language">
                        <p>"lang"</p>
                    </DialogSection>
                </>
            }
            .into_any()
            .to_html()
        });
        assert!(html.contains("Appearance"), "{html}");
        assert!(html.contains("Language"), "{html}");
        assert_eq!(html.matches("first:border-t-0").count(), 2, "{html}");
    }
}

/// `Dialog`'s wiring to `focus_trap`, run natively on its recording page:
/// the `dismissable` prop reaches the one-open rule.
#[cfg(all(test, feature = "ssr"))]
mod one_open_tests {
    use super::*;
    use crate::button::Disclosed;
    use crate::focus_trap::fake::{self, settle};

    /// A `Dialog` `id`, closed; its open flag.
    fn dialog(id: &'static str, dismissable: bool) -> RwSignal<bool> {
        let open = RwSignal::new(false);
        let controls = Disclosed::new(id, open);
        let _ = view! {
            <Dialog controls=controls title="T" dismissable=dismissable>
                <p>"body"</p>
            </Dialog>
        }
        .to_html();
        open
    }

    #[test]
    fn a_non_dismissable_dialog_refuses_another_dialogs_open() {
        fake::run(async || {
            let confirm = dialog("pc-confirm", false);
            let settings = dialog("pc-settings", true);
            settle().await;
            confirm.set(true);
            settle().await;
            settings.set(true);
            settle().await;
            assert!(confirm.get_untracked(), "the confirmation stays open");
            assert!(!settings.get_untracked(), "Settings is refused");
        });
    }

    #[test]
    fn a_dismissable_dialog_is_replaced_by_another_dialogs_open() {
        fake::run(async || {
            let first = dialog("pc-first", true);
            let second = dialog("pc-second", true);
            settle().await;
            first.set(true);
            settle().await;
            second.set(true);
            settle().await;
            assert!(!first.get_untracked(), "the first is replaced");
            assert!(second.get_untracked(), "the second opens");
        });
    }

    #[test]
    fn a_disposed_non_dismissable_dialog_no_longer_refuses_another() {
        fake::run(async || {
            let owner = Owner::new();
            let confirm = owner.with(|| dialog("pc-confirm", false));
            let settings = dialog("pc-settings", true);
            settle().await;
            confirm.set(true);
            settle().await;
            owner.cleanup();
            settle().await;
            settings.set(true);
            settle().await;
            assert!(settings.get_untracked(), "Settings opens: no stale entry");
        });
    }
}
