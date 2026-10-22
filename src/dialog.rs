//! A generic modal dialog over a dimmed, blurred backdrop: opened by any
//! [`Disclosure`], like [`crate::popover::Popover`] — a centred panel with a
//! pinned header (title, close ✕) and an optional pinned footer, the body
//! between them the only part that scrolls. By default
//! ([`DialogSize::Sheet`]) the panel becomes a full-screen sheet below `sm`
//! (no margins; nothing of the backdrop shows behind it anyway);
//! [`DialogSize::Popup`] keeps a small centred card, inset from the screen
//! edge, at every width. At most one `Dialog` is open at a time: opening a second one
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

/// The [`DialogSize::Sheet`] panel: card colour, a full-bleed sheet below
/// `sm`, centred and bounded above it.
const SHEET_PANEL: &str = "flex h-full w-full flex-col overflow-hidden bg-card \
    sm:h-auto sm:max-h-[85vh] sm:w-full sm:max-w-[40rem] sm:rounded-2xl \
    sm:border sm:border-line sm:shadow-lg";

/// The [`DialogSize::Sheet`] wrapper: no inset below `sm`.
const SHEET_WRAPPER: &str = "relative z-10 flex h-full items-center justify-center sm:p-4";

/// The [`DialogSize::Popup`] panel: a small card, the same at every width.
const POPUP_PANEL: &str = "flex w-full max-w-md max-h-[85dvh] flex-col overflow-hidden \
    rounded-2xl border border-line bg-card shadow-lg";

/// The [`DialogSize::Popup`] wrapper: the only owner of the inset, at every
/// width.
const POPUP_WRAPPER: &str = "relative z-10 flex h-full items-center justify-center p-4";

/// How a [`Dialog`] sizes itself. Each size is a complete set of classes
/// for the wrapper and the panel, never a patch on the other's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    /// The default: a full-screen sheet below `sm`, a centred panel (at most
    /// `40rem` wide, `85vh` high) from `sm` up.
    #[default]
    Sheet,
    /// A small centred card at every width: at most `28rem` wide, `85dvh`
    /// high, inset `1rem` from the screen edge, rounded and bordered. For a
    /// short confirmation or a form; the phone shows it as a card, not a
    /// full screen.
    Popup,
}

impl DialogSize {
    /// The wrapper's and the panel's classes for this size.
    const fn classes(self) -> (&'static str, &'static str) {
        match self {
            Self::Sheet => (SHEET_WRAPPER, SHEET_PANEL),
            Self::Popup => (POPUP_WRAPPER, POPUP_PANEL),
        }
    }
}

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
    /// Layout classes added to the panel (e.g. a margin, or a narrower
    /// `max-w` on a [`DialogSize::Sheet`]) — never colours. Never a `max-w-`
    /// with [`DialogSize::Popup`]: the width belongs to the size.
    #[prop(optional, into)]
    layout: String,
    /// Its size: [`DialogSize::Sheet`] (the default, a full-screen sheet on
    /// a phone) or [`DialogSize::Popup`] (a small centred card at every
    /// width). The width belongs to the size, so a caller never passes a
    /// `max-w-` in `layout` with a popup.
    #[prop(optional)]
    size: DialogSize,
    /// The body: the only part that scrolls, e.g. one or more
    /// [`DialogSection`]s.
    children: Children,
) -> impl IntoView {
    focus_trap::manage(controls, dismissable);
    close_on_escape(controls, dismissable);
    let title_id = format!("{}-title", controls.id());
    let header_title_id = title_id.clone();
    let (wrapper_class, panel) = size.classes();
    let panel_class = format!("{panel} {layout}");
    view! {
        <div class="fixed inset-0 z-50" hidden=move || !controls.is_open()>
            <Backdrop controls=controls dismissable=dismissable />
            <div class=wrapper_class>
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
    #[cfg(feature = "browser")]
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
    #[cfg(not(feature = "browser"))]
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

    /// Today's `rendered("pc-dialog-open", true)`, captured before `size`
    /// existed (the Sheet look must stay byte for byte).
    const SHEET_GOLDEN: &str = include_str!("../tests/fixtures/dialog_sheet.html");

    /// A `Dialog` with `size`, the optional `layout`, `dismissable` and a
    /// footer, rendered open.
    fn rendered_with(
        id: &'static str,
        size: DialogSize,
        layout: &'static str,
        dismissable: bool,
        footer: bool,
    ) -> String {
        Owner::new().with(|| {
            let controls = Disclosed::new(id, RwSignal::new(true));
            if footer {
                view! {
                    <Dialog
                        controls=controls
                        title="Settings"
                        size=size
                        layout=layout
                        dismissable=dismissable
                        footer=|| view! { <button>"Save"</button> }.into_any()
                    >
                        <p>"body"</p>
                    </Dialog>
                }
                .to_html()
            } else {
                view! {
                    <Dialog
                        controls=controls
                        title="Settings"
                        size=size
                        layout=layout
                        dismissable=dismissable
                    >
                        <p>"body"</p>
                    </Dialog>
                }
                .to_html()
            }
        })
    }

    /// The panel's `class` value (the tag carrying `role="dialog"`).
    fn panel_class(html: &str) -> &str {
        let tag = &html[html.find(r#"role="dialog""#).expect("a panel")..];
        let from = tag.find(r#"class=""#).expect("a class") + 7;
        let rest = &tag[from..];
        &rest[..rest.find('"').expect("a closing quote")]
    }

    /// The wrapper's `class` value (the `relative z-10` div).
    fn wrapper_class(html: &str) -> &str {
        let from = html.find(r#"class="relative z-10"#).expect("a wrapper") + 7;
        let rest = &html[from..];
        &rest[..rest.find('"').expect("a closing quote")]
    }

    #[test]
    fn the_default_size_is_a_sheet() {
        assert_eq!(DialogSize::default(), DialogSize::Sheet);
    }

    #[test]
    fn sheet_is_byte_for_byte_what_it_was_before_size_existed() {
        // Written after the capture of the fixture, before any code.
        assert_eq!(rendered("pc-dialog-open", true), SHEET_GOLDEN);
        let explicit = rendered_with("pc-dialog-open", DialogSize::Sheet, "", true, false);
        assert_eq!(explicit, SHEET_GOLDEN);
    }

    #[test]
    fn popup_is_a_small_centred_card_with_no_sm_utility() {
        let html = rendered_with("pc-dialog-popup", DialogSize::Popup, "", true, false);
        let panel = panel_class(&html);
        for class in [
            "w-full max-w-md max-h-[85dvh]",
            "overflow-hidden",
            "rounded-2xl",
            "flex-col",
            "border border-line bg-card shadow-lg",
        ] {
            assert!(panel.contains(class), "{class}: {panel}");
        }
        assert!(!panel.split(' ').any(|c| c == "h-full"), "{panel}");
        assert!(!panel.contains("sm:"), "{panel}");
        assert_eq!(panel.matches("max-w-").count(), 1, "{panel}");
    }

    #[test]
    fn popup_wrapper_owns_the_inset_at_every_width() {
        let html = rendered_with("pc-dialog-popup", DialogSize::Popup, "", true, false);
        let wrapper = wrapper_class(&html);
        assert!(wrapper.split(' ').any(|c| c == "p-4"), "{wrapper}");
        assert!(!wrapper.contains("sm:p-4"), "{wrapper}");
        // The sheet keeps its inset from `sm` only.
        let sheet = rendered_with("pc-dialog-sheet", DialogSize::Sheet, "", true, false);
        assert!(wrapper_class(&sheet).contains("sm:p-4"), "{sheet}");
    }

    #[test]
    fn popup_keeps_the_dialog_semantics_and_the_scroll_region() {
        let html = rendered_with("pc-dialog-popup", DialogSize::Popup, "", true, false);
        assert!(html.contains(r#"role="dialog""#), "{html}");
        assert!(html.contains(r#"aria-modal="true""#), "{html}");
        assert!(
            html.contains(r#"aria-labelledby="pc-dialog-popup-title""#),
            "{html}"
        );
        assert!(html.contains(r#"aria-label="Close""#), "{html}");
        assert!(
            html.contains(r#"class="min-h-0 flex-1 overflow-y-auto px-5 py-4""#),
            "{html}"
        );
        assert_eq!(html.matches("overflow-y-auto").count(), 1, "{html}");
    }

    #[test]
    fn popup_has_no_close_button_when_not_dismissable() {
        let html = rendered_with("pc-dialog-popup", DialogSize::Popup, "", false, false);
        assert!(!html.contains(r#"aria-label="Close""#), "{html}");
    }

    #[test]
    fn popup_pins_its_footer() {
        let html = rendered_with("pc-dialog-popup", DialogSize::Popup, "", true, true);
        assert!(html.contains("<button>Save</button>"), "{html}");
        assert!(
            html.contains(r#"class="flex shrink-0 items-center justify-end gap-2"#),
            "{html}"
        );
    }

    #[test]
    fn popup_with_a_layout_appends_it_and_keeps_one_max_width() {
        let html = rendered_with("pc-dialog-popup", DialogSize::Popup, "mt-4", true, false);
        let panel = panel_class(&html);
        assert!(panel.ends_with(" mt-4"), "{panel}");
        assert!(panel.contains("max-w-md"), "{panel}");
        assert_eq!(panel.matches("max-w-").count(), 1, "{panel}");
    }

    #[test]
    fn sheet_with_a_layout_is_unchanged() {
        let html = rendered_with("pc-dialog-sheet", DialogSize::Sheet, "mt-4", true, false);
        let panel = panel_class(&html);
        assert!(
            panel.starts_with("flex h-full w-full flex-col overflow-hidden bg-card"),
            "{panel}"
        );
        assert!(panel.ends_with("sm:shadow-lg mt-4"), "{panel}");
    }

    #[test]
    fn popup_not_dismissable_with_a_footer_is_a_confirmation() {
        let html = rendered_with("pc-dialog-popup", DialogSize::Popup, "", false, true);
        assert!(!html.contains(r#"aria-label="Close""#), "{html}");
        assert!(html.contains("<button>Save</button>"), "{html}");
        assert!(panel_class(&html).contains("max-w-md"), "{html}");
    }

    #[test]
    fn popup_follows_a_reactive_title() {
        let html = Owner::new().with(|| {
            let controls = Disclosed::new("pc-dialog-popup-title", RwSignal::new(true));
            let french = RwSignal::new(false);
            let title = Signal::derive(move || {
                if french.get() {
                    "Paramètres"
                } else {
                    "Settings"
                }
                .to_string()
            });
            let dialog = view! {
                <Dialog controls=controls title=title size=DialogSize::Popup>
                    <p>"body"</p>
                </Dialog>
            };
            french.set(true);
            dialog.to_html()
        });
        assert!(html.contains(">Paramètres</h2>"), "{html}");
        assert!(panel_class(&html).contains("max-w-md"), "{html}");
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
