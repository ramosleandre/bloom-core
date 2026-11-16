//! Data model of the Sense context engine.
//!
//! Normative description: `sense/spec/01-data-model.md` and `10-schema-reference.md`.
//! This crate holds types only: no logic beyond small constructors, no OS code.
//!
//! Implemented so far: the subtrees of the minimum slice (session, activity, apps, windows,
//! focus, selection, clipboard, document, pointer). The optional subtrees (browser, editor,
//! terminal, media, calendar, notifications) are not defined yet.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod protocol;

pub type SourceId = String;
/// RFC 3339, UTC, milliseconds.
pub type Timestamp = String;
pub type Signal = String;
pub type AppId = String;
pub type WindowId = String;
pub type ElementId = String;
pub type TargetId = String;

// ── Envelope ──────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Exact,
    Ambient,
    Inferred,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    System,
    Personal,
    Content,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    NoSource,
    PermissionDenied,
    NotRequested,
    AppNotAccessible,
    Excluded,
    Timeout,
    NotApplicable,
    Level,
    ApprovalDenied,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Observed<T> {
    pub value: T,
    pub via: SourceId,
    pub at: Timestamp,
    pub quality: Quality,
    pub sensitivity: Sensitivity,
    #[serde(default, skip_serializing_if = "is_false")]
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub redacted: bool,
}

/// A leaf that comes from outside the engine: a value with its envelope, or the reason it is
/// not available. "Nothing there" and "could not tell" are different things.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Field<T> {
    Known(Observed<T>),
    Unavailable { unavailable: Reason },
}

impl<T> Field<T> {
    pub fn unavailable(reason: Reason) -> Self {
        Field::Unavailable { unavailable: reason }
    }
    pub fn known(&self) -> Option<&Observed<T>> {
        match self {
            Field::Known(o) => Some(o),
            Field::Unavailable { .. } => None,
        }
    }
    pub fn value(&self) -> Option<&T> {
        self.known().map(|o| &o.value)
    }
    pub fn reason(&self) -> Option<Reason> {
        match self {
            Field::Known(_) => None,
            Field::Unavailable { unavailable } => Some(*unavailable),
        }
    }
}

// ── Enums ─────────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Compositor,
    WindowSystem,
    Accessibility,
    AppExtension,
    AppScripting,
    PrimarySelection,
    Copy,
    Clipboard,
    ProcessInspection,
    Pixels,
    SystemService,
}

/// What the user has allowed. See `spec/00-overview.md`, "Observation levels".
#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Paused,
    OnRequest,
    SessionAwareness,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Macos,
    Windows,
    Linux,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Browser,
    Mail,
    Chat,
    Editor,
    Terminal,
    Office,
    Notes,
    Files,
    Media,
    Meeting,
    PasswordManager,
    System,
    #[default]
    Other,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Window,
    Dialog,
    Document,
    WebArea,
    TextField,
    TextArea,
    SecureTextField,
    Button,
    Link,
    Checkbox,
    Radio,
    ComboBox,
    Menu,
    MenuItem,
    Tab,
    List,
    ListItem,
    Table,
    Row,
    Cell,
    Heading,
    Paragraph,
    Image,
    Toolbar,
    Terminal,
    Canvas,
    Group,
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExcludedBy {
    User,
    AppProfile,
    CaptureProtected,
    PrivateWindow,
    ProviderHint,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClipKind {
    Text,
    Html,
    Image,
    Files,
    Other,
}

// ── State ─────────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Session {
    pub os: Os,
    pub os_version: String,
    pub display_server: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desktop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desktop_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Activity {
    pub idle_ms: Field<u64>,
    pub locked: Field<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct App {
    pub id: AppId,
    pub raw_id: String,
    pub name: String,
    pub pid: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
    pub category: Category,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded: Option<ExcludedBy>,
    pub via: SourceId,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Window {
    pub id: WindowId,
    pub app: AppId,
    pub title: Field<String>,
    pub fullscreen: bool,
    pub private: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded: Option<ExcludedBy>,
    pub via: SourceId,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Element {
    pub role: Role,
    pub raw_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub editable: bool,
    pub multiline: bool,
    pub secure: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caret: Option<u32>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Selection {
    pub text: Field<String>,
    pub len: Field<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_window: Option<WindowId>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Clipboard {
    pub kinds: Field<Vec<ClipKind>>,
    pub size: Field<u64>,
    pub text: Field<String>,
    pub concealed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_at: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Document {
    pub url: Field<String>,
    pub path: Field<String>,
    pub title: Field<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded: Option<ExcludedBy>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Pointer {
    pub x: i32,
    pub y: i32,
}

// ── Events ────────────────────────────────────────────────────────────────────────────────

/// Events carry metadata only: never titles, paths, URLs or text.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Event {
    pub seq: u64,
    pub at: Timestamp,
    pub via: SourceId,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    pub data: serde_json::Value,
}

// ── Snapshot ──────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct FocusView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<App>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<Window>,
    pub element: Field<Element>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<Timestamp>,
}

/// What `capture` returns: a consistent cut of the state at one `seq`.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub seq: u64,
    pub at: Timestamp,
    pub level: Level,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<Session>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<Activity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<FocusView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<Selection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clipboard: Option<Clipboard>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<Field<Pointer>>,
    pub recent: Field<Vec<Event>>,
    pub timings_ms: BTreeMap<SourceId, u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<TargetId>,
}

// ── Capabilities ──────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SignalStatus {
    Available { via: Vec<SourceId> },
    Unavailable { reason: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct SourceStatus {
    pub id: SourceId,
    pub state: String,
    pub channel: Channel,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Capabilities {
    pub level: Level,
    pub signals: BTreeMap<Signal, SignalStatus>,
    pub sources: Vec<SourceStatus>,
}

// ── Access ledger ─────────────────────────────────────────────────────────────────────────

/// One read by a client. Never titles, URLs or content.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct AccessEntry {
    pub at: Timestamp,
    pub client: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    pub kinds: Vec<String>,
    pub bytes: u64,
    pub redacted: bool,
    pub excluded: bool,
}

/// Everything the schema file must describe, in one root, for JSON Schema emission.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SchemaRoot {
    pub snapshot: Snapshot,
    pub capabilities: Capabilities,
    pub access_entry: AccessEntry,
    pub observation: protocol::Observation,
    pub source_hello: protocol::SourceHello,
}

pub fn json_schema() -> String {
    let schema = schemars::schema_for!(SchemaRoot);
    let mut out = serde_json::to_string_pretty(&schema).expect("schema serializes");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(text: &str) -> Field<String> {
        Field::Known(Observed {
            value: text.to_string(),
            via: "test.fake".into(),
            at: "2026-11-16T10:12:03.412Z".into(),
            quality: Quality::Exact,
            sensitivity: Sensitivity::Content,
            truncated: false,
            redacted: false,
        })
    }

    #[test]
    fn field_known_and_unavailable_have_distinct_json_shapes() {
        let k = serde_json::to_value(seen("hello")).unwrap();
        assert_eq!(k["value"], "hello");
        assert_eq!(k["quality"], "exact");
        assert!(k.get("truncated").is_none(), "false flags are omitted");

        let u = serde_json::to_value(Field::<String>::unavailable(Reason::AppNotAccessible)).unwrap();
        assert_eq!(u, serde_json::json!({ "unavailable": "app_not_accessible" }));
    }

    #[test]
    fn field_round_trips() {
        for f in [seen("x"), seen(""), Field::unavailable(Reason::Level)] {
            let json = serde_json::to_string(&f).unwrap();
            let back: Field<String> = serde_json::from_str(&json).unwrap();
            assert_eq!(f, back);
        }
    }

    #[test]
    fn empty_selection_is_not_the_same_as_unknown_selection() {
        let nothing_selected = seen("");
        let could_not_tell = Field::<String>::unavailable(Reason::NoSource);
        assert_ne!(
            serde_json::to_string(&nothing_selected).unwrap(),
            serde_json::to_string(&could_not_tell).unwrap()
        );
        assert_eq!(nothing_selected.value().map(String::as_str), Some(""));
        assert_eq!(could_not_tell.value(), None);
    }

    /// The committed schema must match what this crate emits. Regenerate with
    /// `SENSE_UPDATE_SCHEMA=1 cargo test -p sense-schema`.
    #[test]
    fn committed_json_schema_is_current() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema/sense.schema.json");
        let emitted = json_schema();
        if std::env::var_os("SENSE_UPDATE_SCHEMA").is_some() {
            std::fs::write(path, &emitted).unwrap();
        }
        let committed = std::fs::read_to_string(path).unwrap_or_default();
        assert_eq!(committed, emitted, "schema drift: run with SENSE_UPDATE_SCHEMA=1");
    }
}
