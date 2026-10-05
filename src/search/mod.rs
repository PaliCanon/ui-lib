//! The search forms: [`SearchCard`], the large card of a home page, and
//! [`SearchRow`], the compact one-line bar of a results page's header.
//!
//! Both are plain `<form role="search" method="get">`s, submitted by the
//! browser itself: no JavaScript is needed to search.
//!
//! Two things hold for every form of this module:
//!
//! - **Ids are unique per page.** The field's `id` is its `name` and the
//!   row's form `id` is its `form_id`, so a page showing a second form
//!   passes a distinct `name` (and `form_id`): two forms with the default
//!   `q` would give two elements with the same `id`, and the label of one
//!   would point at the field of the other.
//! - **`group/card` and `group/row` are page-global names.** The submit
//!   button dims while the field is empty through
//!   `group-has-[input:placeholder-shown]/card:` (and `/row:`), which
//!   Tailwind resolves against the nearest ancestor carrying that group
//!   name: a caller's own markup must not reuse `group/card` or
//!   `group/row`.

mod card;
mod row;

pub use card::SearchCard;
pub use row::SearchRow;
