//! The `card` rendering: a compact, deterministic text form of a snapshot, meant to be placed in
//! a model's context. Normative description: `sense/spec/09-exposing-context-to-agents.md`.
//!
//! Rules applied here: small first; say how sure each fact is; say what is unavailable and why;
//! third-party text sits in a tagged block and is never mixed into the prose.

use sense_schema::{Field, Quality, Reason, Snapshot};

fn reason(r: Reason) -> &'static str {
    match r {
        Reason::NoSource => "no source in this session",
        Reason::PermissionDenied => "permission denied",
        Reason::NotRequested => "not requested",
        Reason::AppNotAccessible => "application not accessible",
        Reason::Excluded => "excluded",
        Reason::Timeout => "timed out",
        Reason::NotApplicable => "not applicable",
        Reason::Level => "observation level does not allow it",
        Reason::ApprovalDenied => "approval denied",
    }
}

fn quality(q: Quality) -> &'static str {
    match q {
        Quality::Exact => "exact",
        Quality::Ambient => "ambient, may be stale or from another window",
        Quality::Inferred => "inferred",
    }
}

/// Tags that would let content close its own block are neutralized.
fn inert(text: &str) -> String {
    text.replace("</selection>", "<\u{2060}/selection>")
}

pub fn render(s: &Snapshot) -> String {
    let mut out = String::new();
    let mut unavailable: Vec<String> = Vec::new();
    let mut note = |what: &str, f: Option<Reason>| {
        if let Some(r) = f {
            if r != Reason::NotApplicable {
                unavailable.push(format!("{what} ({})", reason(r)));
            }
        }
    };

    let platform = s
        .session
        .as_ref()
        .map(|x| match &x.desktop {
            Some(d) => format!("{}/{}", serde_json::to_value(x.os).unwrap().as_str().unwrap(), d),
            None => serde_json::to_value(x.os).unwrap().as_str().unwrap().to_string(),
        })
        .unwrap_or_else(|| "unknown".into());
    out.push_str(&format!(
        "[desktop context · {} · {}]\n",
        s.at[..16.min(s.at.len())].replace('T', " "),
        platform
    ));

    if let Some(f) = &s.focus {
        match &f.app {
            Some(a) if a.excluded.is_some() => {
                let cat = serde_json::to_value(a.category).unwrap().as_str().unwrap().to_string();
                out.push_str(&format!("app: (excluded {cat} application)\n"));
            }
            Some(a) => {
                let name = a.canonical.as_deref().unwrap_or(&a.name);
                let cat = serde_json::to_value(a.category).unwrap().as_str().unwrap().to_string();
                out.push_str(&format!("app: {name} ({cat})\n"));
            }
            None => out.push_str("app: none in focus\n"),
        }
        if let Some(w) = &f.window {
            match &w.title {
                Field::Known(t) => out.push_str(&format!("window: {:?}\n", t.value)),
                Field::Unavailable { unavailable } => note("window title", Some(*unavailable)),
            }
        }
        match &f.element {
            Field::Known(e) => {
                let role = serde_json::to_value(e.value.role)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .replace('_', " ");
                let ed = if e.value.editable { ", editable" } else { "" };
                let sec = if e.value.secure { ", secure" } else { "" };
                out.push_str(&format!("focus: {role}{ed}{sec}\n"));
            }
            Field::Unavailable { unavailable } => note("focused element", Some(*unavailable)),
        }
    }

    if let Some(d) = &s.document {
        match (&d.url, &d.path) {
            (Field::Known(u), _) => out.push_str(&format!("document: {}\n", u.value)),
            (_, Field::Known(p)) => out.push_str(&format!("document: {}\n", p.value)),
            (Field::Unavailable { unavailable }, _) => note("document", Some(*unavailable)),
        }
    }

    let mut selection_block = None;
    if let Some(sel) = &s.selection {
        match &sel.text {
            Field::Known(t) if t.value.is_empty() => out.push_str("selection: nothing selected\n"),
            Field::Known(t) => {
                let red = if t.redacted { ", partly redacted" } else { "" };
                let tr = if t.truncated { ", truncated" } else { "" };
                out.push_str(&format!(
                    "selection ({}, {} chars{red}{tr}): see <selection>\n",
                    quality(t.quality),
                    t.value.chars().count()
                ));
                selection_block = Some(format!(
                    "<selection source=\"{}\" trust=\"third-party\">\n{}\n</selection>\n",
                    t.via,
                    inert(&t.value)
                ));
            }
            Field::Unavailable { unavailable } => note("selection", Some(*unavailable)),
        }
    }

    if let Some(c) = &s.clipboard {
        if let Field::Known(k) = &c.kinds {
            let kinds: Vec<String> = k
                .value
                .iter()
                .map(|x| serde_json::to_value(x).unwrap().as_str().unwrap().to_string())
                .collect();
            let size = c.size.value().copied().unwrap_or(0);
            let hidden = if c.concealed { ", concealed" } else { "" };
            out.push_str(&format!(
                "clipboard: {}, {size} bytes{hidden} (not included)\n",
                kinds.join("+")
            ));
        }
    }

    match &s.recent {
        Field::Known(r) if !r.value.is_empty() => {
            let kinds: Vec<&str> = r.value.iter().rev().take(6).map(|e| e.kind.as_str()).collect();
            out.push_str(&format!(
                "recent: {}\n",
                kinds.into_iter().rev().collect::<Vec<_>>().join(" → ")
            ));
        }
        _ => {}
    }

    if !unavailable.is_empty() {
        out.push_str(&format!("unavailable: {}\n", unavailable.join("; ")));
    }
    if let Some(b) = selection_block {
        out.push('\n');
        out.push_str(&b);
    }
    out
}
