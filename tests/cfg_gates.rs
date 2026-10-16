//! The grep gates of the build modes (task 067): `hydrate` and `csr` share
//! the code that needs a browser through the `browser` feature, so a
//! `cfg(feature = "hydrate")` left in `src/` would silently drop that code
//! from a `csr` build. These tests read the sources and fail on:
//!
//! - any `feature = "hydrate"` outside `src/hydration.rs` (its `hydrate()`
//!   panics on a DOM the server never rendered, so it stays on `hydrate`);
//! - any `feature = "csr"` outside `src/csr.rs`;
//! - therefore any `not(feature = "hydrate")` twin left unpaired with its
//!   `not(feature = "browser")` rename;
//!
//! and, the other way round, on the two files that own those names losing
//! their own gate: `src/csr.rs` starts with `#![cfg(feature = "csr")]` and
//! `hydrate` / `app_root` in `src/hydration.rs` carry
//! `#[cfg(feature = "hydrate")]`. The matching ignores whitespace, so
//! `feature="hydrate"`, `feature  =  "hydrate"`, `not (feature = "hydrate")`,
//! `not(any(feature = "hydrate", ...))` and a `cfg_attr` are seen. The
//! reverse gates accept the plain attribute, and for `hydrate` / `app_root`
//! also `all(feature = "hydrate", ...)`; a `cfg_attr`, an `any(...)` or a
//! `not(...)` is no gate (`src/csr.rs` takes the plain inner attribute only).

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("the source directory is readable") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            into.push(path);
        }
    }
}

/// `line` with every whitespace removed: `not (feature = "x")` and
/// `not(feature="x")` become the same text.
fn squeeze(line: &str) -> String {
    line.split_whitespace().collect()
}

/// Whether `line` names `feature = "<name>"`, however it is spaced.
fn names_feature(line: &str, name: &str) -> bool {
    squeeze(line).contains(&format!(r#"feature="{name}""#))
}

/// The text between the parenthesis that `open` (the index just after a
/// `(`) starts and its matching `)`.
fn balanced(text: &str, open: usize) -> Option<&str> {
    let mut depth = 1_usize;
    for (offset, c) in text[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[open..open + offset]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether `line` has a `not(...)` whose content names `feature = "<name>"`,
/// however it is spaced or nested: `not(feature = "x")`,
/// `not(any(feature = "x", ...))`, `not(all(...))`.
fn negates_feature(line: &str, name: &str) -> bool {
    let text = squeeze(line);
    text.match_indices("not(").any(|(at, _)| {
        let word_start = text[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        word_start
            && balanced(&text, at + "not(".len())
                .is_some_and(|inner| inner.contains(&format!(r#"feature="{name}""#)))
    })
}

/// `list` split at its top-level commas (`a,all(b,c),d` gives three).
fn split_top(list: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0_usize, 0);
    for (at, c) in list.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&list[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    parts.push(&list[start..]);
    parts
}

/// Whether the cfg predicate `pred` (squeezed) can only hold with feature
/// `name` on: the feature itself, an `all(...)` with a part that requires it,
/// an `any(...)` whose every part does. `not` and anything else do not.
fn requires_feature(pred: &str, name: &str) -> bool {
    if pred == format!(r#"feature="{name}""#) {
        return true;
    }
    let inside = |head: &str| pred.strip_prefix(head)?.strip_suffix(')');
    if let Some(list) = inside("all(") {
        return split_top(list)
            .into_iter()
            .any(|p| requires_feature(p, name));
    }
    if let Some(list) = inside("any(") {
        let parts = split_top(list);
        return !list.is_empty() && parts.into_iter().all(|p| requires_feature(p, name));
    }
    false
}

/// Whether `line` is a `#[cfg(...)]` (not a `cfg_attr`) whose predicate
/// requires feature `name`: the plain gate, or `all(feature = "<name>", ...)`.
fn is_gate_requiring(line: &str, name: &str) -> bool {
    let text = squeeze(line);
    text.strip_prefix("#[cfg(")
        .and_then(|rest| rest.strip_suffix(")]"))
        .is_some_and(|pred| requires_feature(pred, name))
}

/// Whether `line` is exactly the plain gate `#![cfg(feature = "<name>")]`
/// (`inner`) or `#[cfg(feature = "<name>")]`, spacing aside. A `cfg_attr`,
/// an `any` or an `all` is not a gate: it can switch the item on without
/// the feature, or off with it.
fn is_plain_gate(line: &str, name: &str, inner: bool) -> bool {
    let bang = if inner { "!" } else { "" };
    squeeze(line) == format!(r#"#{bang}[cfg(feature="{name}")]"#)
}

/// The `file:line: text` of each line under `src/` for which `matches` holds,
/// leaving out the file named `allowed` (relative to `src/`).
fn offenders(matches: impl Fn(&str) -> bool, allowed: &str) -> Vec<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut found = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&src).expect("under src/");
        if relative == Path::new(allowed) {
            continue;
        }
        let text = fs::read_to_string(&file).expect("a UTF-8 source file");
        for (number, line) in text.lines().enumerate() {
            if matches(line) {
                found.push(format!(
                    "{}:{}: {}",
                    relative.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }
    found
}

#[test]
fn no_bare_hydrate_feature_outside_hydration_rs() {
    let found = offenders(|line| names_feature(line, "hydrate"), "hydration.rs");
    assert_eq!(
        found.len(),
        0,
        "code that csr needs is gated on `hydrate`; use `browser`:\n{}",
        found.join("\n")
    );
}

#[test]
fn no_bare_csr_feature_outside_csr_rs() {
    let found = offenders(|line| names_feature(line, "csr"), "csr.rs");
    assert_eq!(
        found.len(),
        0,
        "only src/csr.rs is gated on `csr`:\n{}",
        found.join("\n")
    );
}

#[test]
fn no_unpaired_not_hydrate() {
    let found = offenders(|line| negates_feature(line, "hydrate"), "hydration.rs");
    assert_eq!(
        found.len(),
        0,
        "a `not(hydrate)` twin must be `not(browser)` with its positive side:\n{}",
        found.join("\n")
    );
}

#[test]
fn no_cookie_access_outside_the_store() {
    for needle in ["HtmlDocument", ".cookie()", "set_cookie("] {
        let found = offenders(|line| line.contains(needle), "store.rs");
        assert_eq!(
            found.len(),
            0,
            "document.cookie is named in src/store.rs only:\n{}",
            found.join("\n")
        );
    }
}

#[test]
fn the_gates_see_through_spacing_and_cfg_attr() {
    for line in [
        r#"#[cfg(feature = "hydrate")]"#,
        r#"#[cfg(feature="hydrate")]"#,
        r#"#[cfg(feature  =  "hydrate")]"#,
        "#[cfg(any(feature =\t\"hydrate\", test))]",
        r#"#[cfg_attr( feature = "hydrate" , derive(Debug))]"#,
        r#"#[cfg(not(feature="hydrate"))]"#,
    ] {
        assert!(names_feature(line, "hydrate"), "{line}");
    }
    for line in [
        r#"#[cfg(feature = "browser")]"#,
        r#"#[cfg(feature = "csr")]"#,
        "// hydrate with feature flags",
    ] {
        assert!(!names_feature(line, "hydrate"), "{line}");
    }
}

/// The first line of `text` that is neither blank nor a `//` comment.
fn first_code_line(text: &str) -> Option<&str> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("//"))
}

/// Whether the attributes directly above the line that starts with `decl`
/// (skipping doc comments) include a `#[cfg(...)]` that requires the feature:
/// the plain gate or `all(feature = "<name>", ...)`; an `any(...)` or a
/// `cfg_attr` does not.
fn declaration_is_gated(text: &str, decl: &str, name: &str) -> bool {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let Some(at) = lines.iter().position(|line| line.starts_with(decl)) else {
        return false;
    };
    lines[..at]
        .iter()
        .rev()
        .take_while(|line| line.starts_with("///") || line.starts_with("#["))
        .any(|line| is_gate_requiring(line, name))
}

#[test]
fn a_plain_gate_is_the_only_gate_that_counts() {
    assert!(is_plain_gate(r#"#![cfg(feature = "csr")]"#, "csr", true));
    assert!(is_plain_gate(r#"#![ cfg( feature="csr" ) ]"#, "csr", true));
    assert!(is_plain_gate(
        r#"#[cfg(feature = "hydrate")]"#,
        "hydrate",
        false
    ));
    for line in [
        r#"#![cfg_attr(feature = "csr", cfg(feature = "csr"))]"#,
        r#"#![cfg(any(feature = "csr", test))]"#,
        r#"#![cfg(all(feature = "csr", target_arch = "wasm32"))]"#,
        r#"#![cfg(not(feature = "csr"))]"#,
        r#"#[cfg(feature = "csr")]"#,
    ] {
        assert!(!is_plain_gate(line, "csr", true), "{line}");
    }
    assert!(
        !is_plain_gate(
            r#"#[cfg_attr(feature = "hydrate", cfg(feature = "hydrate"))]"#,
            "hydrate",
            false
        ),
        "cfg_attr"
    );
}

#[test]
fn a_negation_is_seen_through_spacing() {
    for line in [
        r#"#[cfg(not(feature = "hydrate"))]"#,
        r#"#[cfg(not (feature = "hydrate"))]"#,
        r#"#[cfg(all(unix, not  (  feature="hydrate" )))]"#,
        "#[cfg(not\t(feature = \"hydrate\"))]",
        r#"#[cfg(not(any(feature = "hydrate", test)))]"#,
        r#"#[cfg(not(all(feature="hydrate", unix)))]"#,
        r#"#[cfg(not ( any ( unix , feature = "hydrate" ) ))]"#,
        r#"#[cfg(all(unix, not(any(test, all(feature = "hydrate", debug_assertions)))))]"#,
    ] {
        assert!(negates_feature(line, "hydrate"), "{line}");
    }
    for line in [
        r#"#[cfg(feature = "hydrate")]"#,
        r#"#[cfg(not(feature = "browser"))]"#,
        r#"#[cfg(not(any(feature = "browser", test)))]"#,
        r#"#[cfg(all(not(test), feature = "hydrate"))]"#,
        r#"#[cfg(cannot(feature = "hydrate"))]"#,
        "let nothing = feature_hydrate;",
    ] {
        assert!(!negates_feature(line, "hydrate"), "{line}");
    }
}

#[test]
fn a_compound_gate_counts_when_it_requires_the_feature() {
    let gated = |attr: &str| {
        let text = format!("/// doc\n{attr}\npub fn f() {{}}\n");
        declaration_is_gated(&text, "pub fn f", "hydrate")
    };
    for attr in [
        r#"#[cfg(feature = "hydrate")]"#,
        r#"#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]"#,
        r#"#[cfg(all(unix, feature="hydrate"))]"#,
        r#"#[cfg( all ( unix , all ( feature = "hydrate" , test ) ) )]"#,
        r#"#[cfg(any(feature = "hydrate", all(feature = "hydrate", unix)))]"#,
    ] {
        assert!(gated(attr), "{attr}");
    }
    for attr in [
        r#"#[cfg(any(feature = "hydrate", test))]"#,
        r#"#[cfg(all(unix, any(feature = "hydrate", test)))]"#,
        r#"#[cfg(not(feature = "hydrate"))]"#,
        r#"#[cfg(all(unix, not(feature = "hydrate")))]"#,
        r#"#[cfg_attr(feature = "hydrate", cfg(feature = "hydrate"))]"#,
        r#"#[cfg(all(unix, feature = "browser"))]"#,
        "#[cfg(all())]",
        "#[cfg(any())]",
    ] {
        assert!(!gated(attr), "{attr}");
    }
}

#[test]
fn the_reverse_helpers_read_a_gate_or_its_absence() {
    let gated = "/// doc\n#[cfg(feature  = \"x\")]\n/// more doc\npub fn f() {}\n";
    assert!(declaration_is_gated(gated, "pub fn f", "x"));
    assert!(
        !declaration_is_gated(gated, "pub fn f", "y"),
        "other feature"
    );
    assert!(!declaration_is_gated(
        "/// doc\npub fn f() {}\n",
        "pub fn f",
        "x"
    ));
    assert!(
        !declaration_is_gated(gated, "pub fn g", "x"),
        "no such item"
    );
    let separated = "#[cfg(feature = \"x\")]\nuse a;\npub fn f() {}\n";
    assert!(
        !declaration_is_gated(separated, "pub fn f", "x"),
        "another item's"
    );
    assert_eq!(
        first_code_line("//! doc\n\n#![cfg(a)]\nx"),
        Some("#![cfg(a)]")
    );
    assert_eq!(first_code_line("//! only docs\n"), None);
}

fn source(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
    fs::read_to_string(path).expect("a UTF-8 source file")
}

#[test]
fn csr_rs_is_gated_on_csr_as_a_whole() {
    let text = source("csr.rs");
    let first = first_code_line(&text).expect("csr.rs has code");
    assert!(
        is_plain_gate(first, "csr", true),
        "src/csr.rs must start with the plain `#![cfg(feature = \"csr\")]`, not `{first}`"
    );
}

#[test]
fn the_hydrate_entries_carry_the_hydrate_gate() {
    let text = source("hydration.rs");
    for decl in ["pub fn app_root(", "pub fn hydrate<"] {
        assert!(
            declaration_is_gated(&text, decl, "hydrate"),
            "`{decl}` in src/hydration.rs must carry `#[cfg(feature = \"hydrate\")]`"
        );
    }
}
