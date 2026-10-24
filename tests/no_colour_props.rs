//! 079's and 080's refusals, held by a scan: the display, input and feedback components take a TONE or a
//! variant, never a colour. In their code (comments and tests aside) it
//! refuses a hex value inside a string, a Tailwind palette colour
//! (`text-red-500`, `bg-slate-800`), an arbitrary value (`bg-[…]`), and a
//! `style=` attribute or a `color`/`colour`/`style` prop.

const FILES: [&str; 12] = [
    "src/tone.rs",
    "src/badge.rs",
    "src/surface.rs",
    "src/avatar.rs",
    "src/callout.rs",
    "src/tabs.rs",
    "src/data_table.rs",
    "src/field.rs",
    "src/switch.rs",
    "src/copy_field.rs",
    "src/toast.rs",
    "src/confirm.rs",
];

const PALETTE: [&str; 24] = [
    "slate", "gray", "zinc", "neutral", "stone", "red", "orange", "amber", "yellow", "lime",
    "green", "emerald", "teal", "cyan", "sky", "blue", "indigo", "violet", "purple", "fuchsia",
    "pink", "rose", "black", "white",
];

const UTILITIES: [&str; 9] = [
    "bg-", "text-", "border-", "fill-", "stroke-", "ring-", "outline-", "from-", "to-",
];

/// The source without its `//` comments and its tests module.
fn code(path: &str) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let body = text.split("#[cfg(test)]").next().unwrap_or(&text);
    let body = body.split("#[cfg(all(test").next().unwrap_or(body);
    body.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The string literals of a line (between double quotes).
fn strings(line: &str) -> Vec<&str> {
    line.split('"').skip(1).step_by(2).collect()
}

/// Whether `s` holds `#` then 3, 4, 6 or 8 hex digits ending there — not
/// an anchor like `#add-member`, whose digits run into a letter or a dash.
fn has_hex(s: &str) -> bool {
    s.match_indices('#').any(|(i, _)| {
        let digits = s[i + 1..]
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .count();
        [3, 4, 6, 8].contains(&digits)
            && !s[i + 1 + digits..]
                .starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '-')
    })
}

/// Whether a class list uses a palette colour or an arbitrary COLOUR value
/// (`text-[11px]` is a size and passes).
fn has_palette(s: &str) -> bool {
    s.split_whitespace().any(|class| {
        let class = class.rsplit(':').next().unwrap_or(class);
        UTILITIES.iter().any(|u| {
            class.strip_prefix(u).is_some_and(|rest| {
                let arbitrary_colour = rest.starts_with('[')
                    && ["#", "rgb", "hsl", "var("].iter().any(|c| rest.contains(c));
                arbitrary_colour
                    || PALETTE
                        .iter()
                        .any(|p| rest == *p || rest.starts_with(&format!("{p}-")))
            })
        })
    })
}

fn findings(path: &str, src: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (n, line) in src.lines().enumerate() {
        let at = format!("{path}:{}", n + 1);
        for s in strings(line) {
            if has_hex(s) {
                found.push(format!("{at}: a hex colour: {s}"));
            }
            if has_palette(s) {
                found.push(format!("{at}: a palette or arbitrary colour: {s}"));
            }
        }
        if line.contains(" style=") || line.contains("<style") {
            found.push(format!("{at}: a style attribute: {line}"));
        }
        let trimmed = line.trim_start();
        for prop in ["color:", "colour:", "style:"] {
            if trimmed.starts_with(prop) {
                found.push(format!("{at}: a colour or style prop: {line}"));
            }
        }
    }
    found
}

#[test]
fn no_display_component_carries_a_colour() {
    let found: Vec<String> = FILES.iter().flat_map(|p| findings(p, &code(p))).collect();
    assert_eq!(found.len(), 0, "\n{}", found.join("\n"));
}

#[test]
fn the_scan_sees_each_form_it_refuses() {
    for (bad, form) in [
        (r#"let c = "bg-card #ff0000";"#, "hex"),
        (r#"let c = "text-red-500";"#, "palette"),
        (r#"let c = "hover:bg-slate-800";"#, "palette under a state"),
        (r#"let c = "bg-[#123456]";"#, "arbitrary"),
        (r#"<div style="color: red">"#, "style attribute"),
        ("    color: String,", "colour prop"),
    ] {
        assert_eq!(
            findings("probe", bad).len().min(1),
            1,
            "{form} not seen: {bad}"
        );
    }
    for good in [
        r#"let c = "bg-card text-muted border-danger-text";"#,
        "#[derive(Clone)]",
        r#"let c = "bg-success-soft text-success";"#,
        r#"let c = "size-7 text-[11px] bg-transparent";"#,
        r##"let href = "#add-member";"##,
    ] {
        assert_eq!(findings("probe", good).len(), 0, "a false alarm: {good}");
    }
}
