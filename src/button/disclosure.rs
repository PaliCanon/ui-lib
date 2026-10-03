//! Something that opens and closes — the rail, a menu, a dialog — and the
//! id of the element it shows, for `aria-controls`.

use leptos::prelude::*;

/// Opens and closes an element whose id is [`Disclosure::id`].
pub trait Disclosure: Copy + Send + Sync + 'static {
    /// The id of the element shown and hidden.
    fn id(&self) -> &'static str;
    /// Whether it is open (tracked: reading it in a view follows changes).
    fn is_open(&self) -> bool;
    /// Opens or closes it.
    fn set_open(&self, open: bool);
    /// Opens it if closed, closes it if open.
    fn toggle(&self) {
        self.set_open(!self.is_open());
    }
}

/// The plain disclosure: an id and an open flag.
#[derive(Clone, Copy, Debug)]
pub struct Disclosed {
    id: &'static str,
    open: RwSignal<bool>,
}

impl Disclosed {
    /// `id` names the element; `open` is its state (a plain or persisted signal).
    #[must_use]
    pub fn new(id: &'static str, open: RwSignal<bool>) -> Self {
        Self { id, open }
    }

    /// The underlying signal, to read it elsewhere.
    #[must_use]
    pub fn signal(&self) -> RwSignal<bool> {
        self.open
    }
}

impl Disclosure for Disclosed {
    fn id(&self) -> &'static str {
        self.id
    }
    fn is_open(&self) -> bool {
        self.open.get()
    }
    fn set_open(&self, open: bool) {
        self.open.set(open);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_and_names_what_it_controls() {
        Owner::new().with(|| {
            let d = Disclosed::new("pc-rail", RwSignal::new(false));
            assert_eq!(d.id(), "pc-rail");
            assert!(!d.is_open());
            d.toggle();
            assert!(d.is_open());
            assert!(d.signal().get_untracked());
            d.set_open(false);
            assert!(!d.is_open());
        });
    }
}
