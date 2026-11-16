//! What sources send to the engine, and the messages of the source protocol.
//! Normative description: `sense/spec/02-sources.md`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Channel, ClipKind, Quality, Role, Signal, SourceId, Timestamp};

/// A fact reported by a source, in OS terms. Sources do not interpret, rank or normalize.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Patch {
    AppSeen {
        pid: u32,
        raw_id: String,
        name: String,
    },
    AppGone {
        pid: u32,
    },
    /// `key` identifies the window for this source only; the engine assigns the public id.
    WindowSeen {
        key: String,
        pid: u32,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        fullscreen: bool,
        #[serde(default)]
        private: bool,
        /// The application refuses screen capture for this window.
        #[serde(default)]
        capture_protected: bool,
    },
    WindowGone {
        key: String,
    },
    FocusChanged {
        #[serde(default)]
        key: Option<String>,
    },
    Element {
        role: Role,
        raw_role: String,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        editable: bool,
        #[serde(default)]
        multiline: bool,
        #[serde(default)]
        secure: bool,
        #[serde(default)]
        value_len: Option<u32>,
        #[serde(default)]
        caret: Option<u32>,
    },
    Selection {
        /// `Some("")` means "nothing is selected". `None` means the source knows only the length.
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        len: Option<u32>,
        quality: Quality,
    },
    Clipboard {
        kinds: Vec<ClipKind>,
        size: u64,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        concealed: bool,
    },
    Document {
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        title: Option<String>,
        /// Provider hint: the page or application asked not to be read.
        #[serde(default)]
        no_context: bool,
    },
    Pointer {
        x: i32,
        y: i32,
    },
    Activity {
        #[serde(default)]
        idle_ms: Option<u64>,
        #[serde(default)]
        locked: Option<bool>,
    },
}

impl Patch {
    /// The state path this patch can fill, for capability reporting and level gating.
    pub fn signal(&self) -> &'static str {
        match self {
            Patch::AppSeen { .. } | Patch::AppGone { .. } => "apps",
            Patch::WindowSeen { .. } | Patch::WindowGone { .. } => "windows",
            Patch::FocusChanged { .. } => "focus",
            Patch::Element { .. } => "focus.element",
            Patch::Selection { .. } => "selection",
            Patch::Clipboard { .. } => "clipboard",
            Patch::Document { .. } => "document",
            Patch::Pointer { .. } => "pointer",
            Patch::Activity { .. } => "activity",
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct Observation {
    pub via: SourceId,
    pub channel: Channel,
    pub at: Timestamp,
    pub patch: Patch,
    /// True when this observation answers a read made for a user-initiated capture.
    /// Set by the engine, never trusted from the wire.
    #[serde(default, skip_serializing)]
    pub on_request: bool,
}

/// First message of a source connection.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct SourceHello {
    pub id: SourceId,
    pub version: String,
    pub channel: Channel,
    pub provides: Vec<Signal>,
}

/// A request from the engine to a source: report your current values for these signals.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ReadCurrent {
    pub request_id: u64,
    pub signals: Vec<Signal>,
    pub deadline_ms: u32,
}

/// The answer: the patches, in the same shape as unsolicited observations.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ReadResult {
    pub request_id: u64,
    pub patches: Vec<Patch>,
}
