//! The privacy filter: pure functions applied to every observation before it is stored or served.
//! Normative description: `sense/spec/04-privacy-security.md`.

use sense_schema::protocol::{Observation, Patch};
use sense_schema::Level;

/// Default cap for one text value, in bytes.
pub const TEXT_CAP: usize = 64 * 1024;

/// Level gate. Returns the observation as it may be stored at this level, or `None`.
///
/// - Paused: nothing.
/// - Locked session: only activity.
/// - On request: only the identity of what is in front (no titles) and activity, unless the
///   observation answers a user-initiated capture.
/// - Session awareness: metadata is kept; selection and clipboard *text* still only on request.
pub fn gate(mut obs: Observation, level: Level, locked: bool) -> Option<Observation> {
    if level == Level::Paused {
        return None;
    }
    if locked && !matches!(obs.patch, Patch::Activity { .. }) {
        return None;
    }
    if obs.on_request {
        return Some(obs);
    }
    match level {
        Level::Paused => None,
        Level::OnRequest => match &mut obs.patch {
            Patch::AppSeen { .. }
            | Patch::AppGone { .. }
            | Patch::WindowGone { .. }
            | Patch::FocusChanged { .. }
            | Patch::Activity { .. } => Some(obs),
            Patch::WindowSeen { title, .. } => {
                *title = None;
                Some(obs)
            }
            _ => None,
        },
        Level::SessionAwareness => {
            match &mut obs.patch {
                Patch::Selection { text, len, .. } => {
                    if let Some(t) = text.take() {
                        len.get_or_insert(t.chars().count() as u32);
                    }
                }
                Patch::Clipboard { text, .. } => *text = None,
                _ => {}
            }
            Some(obs)
        }
    }
}

/// Result of cleaning one text value.
#[derive(Debug, PartialEq)]
pub struct Cleaned {
    pub text: String,
    pub redacted: bool,
    pub truncated: bool,
}

/// Sanitize, redact (optional) and cap a text value read from another application.
pub fn clean(text: &str, redact_secrets: bool, cap: usize) -> Cleaned {
    let sanitized = sanitize(text);
    let (text, redacted) = if redact_secrets {
        redact(&sanitized)
    } else {
        (sanitized, false)
    };
    let (text, truncated) = truncate(text, cap);
    Cleaned {
        text,
        redacted,
        truncated,
    }
}

/// Remove characters used to hide instructions from the user while showing them to a model:
/// the Unicode tag block, zero-width characters, bidirectional overrides, and control characters
/// other than newline and tab.
pub fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            let u = c as u32;
            let hidden = (0xE0000..=0xE007F).contains(&u)
                || matches!(u, 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0x2066..=0x2069 | 0xFEFF);
            let control = c.is_control() && c != '\n' && c != '\t';
            !hidden && !control
        })
        .collect()
}

fn truncate(text: String, cap: usize) -> (String, bool) {
    if text.len() <= cap {
        return (text, false);
    }
    let mut end = cap;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_string(), true)
}

/// Replace high-confidence secret patterns with typed placeholders. Best effort: it catches
/// accidents, it is not a guarantee.
pub fn redact(text: &str) -> (String, bool) {
    let mut out = text.to_string();
    let mut hit = false;
    for pass in [redact_key_blocks, redact_tokens, redact_cards, redact_ibans] {
        let (next, changed) = pass(&out);
        out = next;
        hit |= changed;
    }
    (out, hit)
}

fn redact_key_blocks(text: &str) -> (String, bool) {
    const BEGIN: &str = "-----BEGIN ";
    const KEY: &str = "PRIVATE KEY-----";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut hit = false;
    while let Some(start) = rest.find(BEGIN) {
        let after = &rest[start..];
        let header_end = after.find('\n').unwrap_or(after.len());
        if !after[..header_end].contains(KEY) {
            out.push_str(&rest[..start + BEGIN.len()]);
            rest = &rest[start + BEGIN.len()..];
            continue;
        }
        let end = after
            .find("-----END ")
            .and_then(|e| after[e..].find(KEY).map(|k| e + k + KEY.len()))
            .unwrap_or(after.len());
        out.push_str(&rest[..start]);
        out.push_str("[redacted: private key]");
        rest = &after[end..];
        hit = true;
    }
    out.push_str(rest);
    (out, hit)
}

const TOKEN_PREFIXES: &[(&str, usize)] = &[
    ("github_pat_", 20),
    ("ghp_", 20),
    ("gho_", 20),
    ("ghs_", 20),
    ("glpat-", 16),
    ("sk-", 20),
    ("xoxb-", 16),
    ("xoxp-", 16),
    ("AKIA", 16),
];

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn redact_tokens(text: &str) -> (String, bool) {
    let mut out = String::with_capacity(text.len());
    let mut hit = false;
    let mut i = 0;
    let bytes = text.as_bytes();
    'outer: while i < text.len() {
        let at_boundary = i == 0 || !is_token_char(bytes[i - 1] as char);
        if at_boundary {
            for (prefix, min_body) in TOKEN_PREFIXES {
                if text[i..].starts_with(prefix) {
                    let body = text[i + prefix.len()..]
                        .chars()
                        .take_while(|&c| is_token_char(c))
                        .count();
                    if body >= *min_body {
                        out.push_str("[redacted: access token]");
                        i += prefix.len() + body;
                        hit = true;
                        continue 'outer;
                    }
                }
            }
            if let Some(len) = jwt_len(&text[i..]) {
                out.push_str("[redacted: access token]");
                i += len;
                hit = true;
                continue;
            }
        }
        let c = text[i..].chars().next().expect("in bounds");
        out.push(c);
        i += c.len_utf8();
    }
    (out, hit)
}

/// Three base64url segments separated by dots, the first two starting with `eyJ`.
fn jwt_len(s: &str) -> Option<usize> {
    if !s.starts_with("eyJ") {
        return None;
    }
    let seg = |s: &str| s.chars().take_while(|&c| is_token_char(c)).count();
    let a = seg(s);
    let rest = s.get(a..)?.strip_prefix('.')?;
    if !rest.starts_with("eyJ") {
        return None;
    }
    let b = seg(rest);
    let rest2 = rest.get(b..)?.strip_prefix('.')?;
    let c = seg(rest2);
    (a >= 10 && b >= 10 && c >= 10).then_some(a + 1 + b + 1 + c)
}

fn luhn_ok(digits: &[u8]) -> bool {
    let mut sum = 0u32;
    for (i, d) in digits.iter().rev().enumerate() {
        let mut v = u32::from(*d);
        if i % 2 == 1 {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
    }
    sum % 10 == 0
}

/// Runs of 13 to 19 digits, optionally grouped by single spaces or dashes, that pass Luhn.
fn redact_cards(text: &str) -> (String, bool) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut hit = false;
    let mut i = 0;
    while i < chars.len() {
        let starts_run = chars[i].is_ascii_digit() && (i == 0 || !chars[i - 1].is_ascii_alphanumeric());
        if starts_run {
            let mut j = i;
            let mut digits = Vec::new();
            while j < chars.len() {
                if chars[j].is_ascii_digit() {
                    digits.push(chars[j] as u8 - b'0');
                    j += 1;
                } else if (chars[j] == ' ' || chars[j] == '-')
                    && j + 1 < chars.len()
                    && chars[j + 1].is_ascii_digit()
                    && digits.len() < 19
                {
                    j += 1;
                } else {
                    break;
                }
            }
            let ends_clean = j == chars.len() || !chars[j].is_ascii_alphanumeric();
            if ends_clean && (13..=19).contains(&digits.len()) && luhn_ok(&digits) {
                out.push_str("[redacted: card number]");
                hit = true;
                i = j;
                continue;
            }
            out.extend(&chars[i..j]);
            i = j;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    (out, hit)
}

fn iban_ok(compact: &str) -> bool {
    if !(15..=34).contains(&compact.len()) {
        return false;
    }
    let rearranged = compact[4..].chars().chain(compact[..4].chars());
    let mut rem = 0u32;
    for c in rearranged {
        let v = match c {
            '0'..='9' => c as u32 - '0' as u32,
            'A'..='Z' => c as u32 - 'A' as u32 + 10,
            _ => return false,
        };
        rem = if v > 9 {
            (rem * 100 + v) % 97
        } else {
            (rem * 10 + v) % 97
        };
    }
    rem == 1
}

/// Two letters, two digits, then 11 to 30 alphanumerics, optionally in space-separated groups,
/// with a valid ISO 13616 checksum.
fn redact_ibans(text: &str) -> (String, bool) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut hit = false;
    let mut i = 0;
    while i < chars.len() {
        let boundary = i == 0 || !chars[i - 1].is_ascii_alphanumeric();
        let looks = boundary
            && i + 4 <= chars.len()
            && chars[i].is_ascii_uppercase()
            && chars[i + 1].is_ascii_uppercase()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit();
        if looks {
            let mut j = i;
            let mut compact = String::new();
            while j < chars.len() && compact.len() < 34 {
                let c = chars[j];
                if c.is_ascii_uppercase() || c.is_ascii_digit() {
                    compact.push(c);
                    j += 1;
                } else if c == ' ' && j + 1 < chars.len() && chars[j + 1].is_ascii_alphanumeric() {
                    j += 1;
                } else {
                    break;
                }
            }
            // Try the longest candidate first, then shorter ones ending on a group boundary.
            let mut end = j;
            let mut cand = compact.clone();
            loop {
                let clean_end = end == chars.len() || !chars[end].is_ascii_alphanumeric();
                if clean_end && iban_ok(&cand) {
                    out.push_str("[redacted: IBAN]");
                    hit = true;
                    i = end;
                    break;
                }
                // Drop the last space-separated group, if any.
                let Some(sp) = chars[i..end].iter().rposition(|&c| c == ' ') else {
                    break;
                };
                end = i + sp;
                cand = chars[i..end].iter().filter(|c| **c != ' ').collect();
                if cand.len() < 15 {
                    break;
                }
            }
            if i == end && hit {
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    (out, hit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sense_schema::{Channel, Quality};

    fn obs(patch: Patch, on_request: bool) -> Observation {
        Observation {
            via: "t".into(),
            channel: Channel::Compositor,
            at: "now".into(),
            patch,
            on_request,
        }
    }
    fn window(title: &str) -> Patch {
        Patch::WindowSeen {
            key: "w".into(),
            pid: 1,
            title: Some(title.into()),
            fullscreen: false,
            private: false,
            capture_protected: false,
        }
    }
    fn selection(text: &str) -> Patch {
        Patch::Selection {
            text: Some(text.into()),
            len: None,
            quality: Quality::Exact,
        }
    }

    #[test]
    fn paused_drops_everything() {
        assert!(gate(obs(window("x"), true), Level::Paused, false).is_none());
        assert!(gate(
            obs(
                Patch::Activity {
                    idle_ms: Some(1),
                    locked: None
                },
                false
            ),
            Level::Paused,
            false
        )
        .is_none());
    }

    #[test]
    fn locked_keeps_only_activity() {
        assert!(gate(obs(window("x"), true), Level::SessionAwareness, true).is_none());
        assert!(gate(
            obs(
                Patch::Activity {
                    idle_ms: None,
                    locked: Some(false)
                },
                false
            ),
            Level::OnRequest,
            true
        )
        .is_some());
    }

    #[test]
    fn on_request_level_keeps_identity_but_not_titles_or_content() {
        let kept = gate(obs(window("Secret plan"), false), Level::OnRequest, false).unwrap();
        assert!(matches!(kept.patch, Patch::WindowSeen { title: None, .. }));
        assert!(gate(obs(selection("abc"), false), Level::OnRequest, false).is_none());
        assert!(gate(obs(Patch::Pointer { x: 1, y: 2 }, false), Level::OnRequest, false).is_none());
        assert!(gate(
            obs(Patch::FocusChanged { key: Some("w".into()) }, false),
            Level::OnRequest,
            false
        )
        .is_some());
    }

    #[test]
    fn on_request_level_lets_capture_answers_through() {
        let kept = gate(obs(window("Secret plan"), true), Level::OnRequest, false).unwrap();
        assert!(matches!(kept.patch, Patch::WindowSeen { title: Some(_), .. }));
        assert!(gate(obs(selection("abc"), true), Level::OnRequest, false).is_some());
    }

    #[test]
    fn session_awareness_keeps_selection_length_not_text() {
        let kept = gate(obs(selection("héllo"), false), Level::SessionAwareness, false).unwrap();
        assert!(matches!(
            kept.patch,
            Patch::Selection {
                text: None,
                len: Some(5),
                ..
            }
        ));
    }

    #[test]
    fn sanitize_strips_hidden_characters() {
        let hidden = "pay\u{200B}ment \u{E0049}\u{E0067}nore \u{202E}txet";
        assert_eq!(sanitize(hidden), "payment nore txet");
        assert_eq!(sanitize("line1\n\tline2\u{0007}"), "line1\n\tline2");
    }

    #[test]
    fn redacts_card_numbers_that_pass_luhn_only() {
        let (t, hit) = redact("card 4111 1111 1111 1111 exp 12/29");
        assert!(hit);
        assert_eq!(t, "card [redacted: card number] exp 12/29");
        let (t, hit) = redact("order 4111 1111 1111 1112");
        assert!(!hit, "fails Luhn, left alone");
        assert_eq!(t, "order 4111 1111 1111 1112");
        let (t, hit) = redact("phone 0612345678 and id 1234567890123");
        assert!(!hit, "{t}");
    }

    #[test]
    fn redacts_private_key_blocks() {
        let text = "before\n-----BEGIN OPENSSH PRIVATE KEY-----\nabc\ndef\n-----END OPENSSH PRIVATE KEY-----\nafter";
        let (t, hit) = redact(text);
        assert!(hit);
        assert_eq!(t, "before\n[redacted: private key]\nafter");
        let (t, hit) = redact("-----BEGIN CERTIFICATE-----\nabc\n-----END CERTIFICATE-----");
        assert!(!hit);
        assert!(t.contains("CERTIFICATE"));
    }

    #[test]
    fn redacts_tokens_with_known_prefixes() {
        let (t, hit) = redact("export GH=ghp_abcdefghijklmnopqrstuvwxyz0123456789 ok");
        assert!(hit);
        assert_eq!(t, "export GH=[redacted: access token] ok");
        let (_, hit) = redact("risk-management and sk-short");
        assert!(!hit);
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let (t, hit) = redact(&format!("Bearer {jwt}"));
        assert!(hit);
        assert_eq!(t, "Bearer [redacted: access token]");
    }

    #[test]
    fn redacts_ibans_with_valid_checksum_only() {
        let (t, hit) = redact("IBAN: FR14 2004 1010 0505 0001 3M02 606 merci");
        assert!(hit, "{t}");
        assert_eq!(t, "IBAN: [redacted: IBAN] merci");
        let (t, hit) = redact("GB82WEST12345698765432.");
        assert!(hit);
        assert_eq!(t, "[redacted: IBAN].");
        let (_, hit) = redact("FR14 2004 1010 0505 0001 3M02 607");
        assert!(!hit, "wrong checksum, left alone");
    }

    #[test]
    fn clean_caps_on_a_char_boundary() {
        let c = clean("ééééé", false, 5);
        assert!(c.truncated);
        assert_eq!(c.text, "éé");
    }
}
