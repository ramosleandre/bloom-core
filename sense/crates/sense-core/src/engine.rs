//! The engine: applies filtered observations to the state and answers captures.
//! Normative description: `sense/spec/00-overview.md`, `01-data-model.md`, `03-client-api.md`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sense_schema::protocol::{Observation, Patch, SourceHello};
use sense_schema::{
    AccessEntry, Activity, App, Capabilities, Category, Channel, ClipKind, Clipboard, Document, Element, Event,
    ExcludedBy, Field, FocusView, Level, Observed, Os, Pointer, Quality, Reason, Selection, Sensitivity, Session,
    SignalStatus, Snapshot, SourceId, SourceStatus, Window,
};

use crate::privacy::{self, TEXT_CAP};
use crate::profiles::Profiles;
use crate::time;

/// A higher-ranked value older than this loses to a fresher lower-ranked one.
const STALE_MS: u64 = 2_000;
const TIMELINE_MAX: usize = 20_000;
const LEDGER_MAX: usize = 1_000;

/// One way of observing. Platform sources and extension sources both implement this.
pub trait Source: Send {
    fn hello(&self) -> SourceHello;
    /// Report current values for these signals. Must return within `deadline`, with what it has.
    fn read_current(&self, signals: &[String], deadline: Duration) -> Vec<Patch>;
}

#[derive(Clone, Debug)]
pub struct Config {
    pub level: Level,
    /// Raw ids or canonical names the user excluded.
    pub exclude_apps: Vec<String>,
    pub redaction: bool,
    pub text_cap: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            level: Level::OnRequest,
            exclude_apps: Vec::new(),
            redaction: true,
            text_cap: TEXT_CAP,
        }
    }
}

type Precedence = BTreeMap<String, Vec<Channel>>;

fn builtin_precedence() -> Precedence {
    #[derive(serde::Deserialize)]
    struct Entry {
        order: Vec<Channel>,
    }
    let raw: BTreeMap<String, Entry> =
        toml::from_str(include_str!("../data/precedence.toml")).expect("built-in precedence parses");
    raw.into_iter().map(|(k, v)| (k, v.order)).collect()
}

#[derive(Clone, Debug)]
struct Stored<T> {
    value: T,
    via: SourceId,
    channel: Channel,
    at_ms: u64,
    quality: Quality,
    redacted: bool,
    truncated: bool,
}

#[derive(Clone, Debug)]
struct AppRec {
    id: String,
    raw_id: String,
    name: String,
    via: SourceId,
}

type WinKey = (SourceId, String);

#[derive(Clone, Debug)]
struct WinRec {
    id: String,
    pid: u32,
    title: Option<Stored<String>>,
    fullscreen: bool,
    private: bool,
    capture_protected: bool,
    via: SourceId,
}

#[derive(Clone, Debug)]
struct ClipRec {
    kinds: Vec<ClipKind>,
    size: u64,
    concealed: bool,
}

#[derive(Clone)]
pub struct Engine {
    cfg: Config,
    profiles: Profiles,
    precedence: Precedence,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    session: Session,
    seq: u64,
    next_id: u64,
    apps: BTreeMap<u32, AppRec>,
    windows: BTreeMap<WinKey, WinRec>,
    focus: Option<Stored<WinKey>>,
    element: Option<Stored<Element>>,
    selection_text: Option<Stored<String>>,
    selection_len: Option<Stored<u32>>,
    clipboard: Option<Stored<ClipRec>>,
    doc_url: Option<Stored<String>>,
    doc_path: Option<Stored<String>>,
    doc_title: Option<Stored<String>>,
    doc_no_context: bool,
    pointer: Option<Stored<Pointer>>,
    idle_ms: Option<Stored<u64>>,
    locked: Option<Stored<bool>>,
    own_pids: BTreeSet<u32>,
    sources: BTreeMap<SourceId, SourceHello>,
    timeline: VecDeque<Event>,
    ledger: VecDeque<AccessEntry>,
}

/// What a capture asks for. `want` lists state subtrees: focus, selection, document, clipboard,
/// pointer, activity, session.
#[derive(Clone, Debug)]
pub struct CaptureRequest {
    pub client: String,
    pub want: Vec<String>,
    pub deadline: Duration,
    pub recent_limit: usize,
}

impl Default for CaptureRequest {
    fn default() -> Self {
        CaptureRequest {
            client: "unknown".into(),
            want: ["session", "focus", "selection", "document"].map(String::from).to_vec(),
            deadline: Duration::from_millis(50),
            recent_limit: 30,
        }
    }
}

impl Engine {
    pub fn new(cfg: Config, session: Session) -> Self {
        Engine::with_clock(cfg, session, Arc::new(time::now_ms))
    }

    pub fn with_clock(cfg: Config, session: Session, clock: Arc<dyn Fn() -> u64 + Send + Sync>) -> Self {
        Engine {
            cfg,
            profiles: Profiles::builtin(),
            precedence: builtin_precedence(),
            clock,
            session,
            seq: 0,
            next_id: 0,
            apps: BTreeMap::new(),
            windows: BTreeMap::new(),
            focus: None,
            element: None,
            selection_text: None,
            selection_len: None,
            clipboard: None,
            doc_url: None,
            doc_path: None,
            doc_title: None,
            doc_no_context: false,
            pointer: None,
            idle_ms: None,
            locked: None,
            own_pids: BTreeSet::new(),
            sources: BTreeMap::new(),
            timeline: VecDeque::new(),
            ledger: VecDeque::new(),
        }
    }

    pub fn level(&self) -> Level {
        self.cfg.level
    }

    /// Changing the level clears everything the new level may not retain.
    pub fn set_level(&mut self, level: Level) {
        let from = self.cfg.level;
        self.cfg.level = level;
        if level < from {
            self.forget_content();
            if level == Level::Paused {
                self.apps.clear();
                self.windows.clear();
                self.focus = None;
            } else {
                for w in self.windows.values_mut() {
                    w.title = None;
                }
            }
            self.timeline.clear();
        }
        self.event(
            "sense",
            "level.changed",
            serde_json::json!({ "from": from, "to": level }),
        );
    }

    pub fn register_own_pids(&mut self, pids: &[u32]) {
        self.own_pids.extend(pids.iter().copied());
    }

    pub fn register_source(&mut self, hello: SourceHello) {
        self.event(
            &hello.id.clone(),
            "source.state",
            serde_json::json!({ "state": "running" }),
        );
        self.sources.insert(hello.id.clone(), hello);
    }

    pub fn unregister_source(&mut self, id: &str) {
        if self.sources.remove(id).is_some() {
            self.windows.retain(|(via, _), _| via != id);
            if self.focus.as_ref().is_some_and(|f| f.via == id) {
                self.focus = None;
                self.forget_content();
            }
            self.event(id, "source.state", serde_json::json!({ "state": "stopped" }));
        }
    }

    pub fn ledger(&self) -> impl Iterator<Item = &AccessEntry> {
        self.ledger.iter()
    }

    pub fn timeline(&self) -> impl Iterator<Item = &Event> {
        self.timeline.iter()
    }

    pub fn capabilities(&self) -> Capabilities {
        let mut signals: BTreeMap<String, SignalStatus> = BTreeMap::new();
        for s in [
            "focus",
            "windows",
            "focus.element",
            "selection",
            "document",
            "clipboard",
            "pointer",
            "activity",
        ] {
            let via: Vec<SourceId> = self
                .sources
                .values()
                .filter(|h| h.provides.iter().any(|p| p == s))
                .map(|h| h.id.clone())
                .collect();
            let status = if via.is_empty() {
                SignalStatus::Unavailable {
                    reason: "no source provides it in this session".into(),
                }
            } else {
                SignalStatus::Available { via }
            };
            signals.insert(s.to_string(), status);
        }
        let sources = self
            .sources
            .values()
            .map(|h| SourceStatus {
                id: h.id.clone(),
                state: "running".into(),
                channel: h.channel,
            })
            .collect();
        Capabilities {
            level: self.cfg.level,
            signals,
            sources,
        }
    }

    // ── Applying observations ─────────────────────────────────────────────────────────────

    /// Apply one observation: privacy gate first, then the reducer.
    pub fn apply(&mut self, obs: Observation) {
        let locked = self.locked.as_ref().is_some_and(|l| l.value);
        let Some(obs) = privacy::gate(obs, self.cfg.level, locked) else {
            return;
        };
        let now = (self.clock)();
        let (via, channel) = (obs.via.clone(), obs.channel);
        match obs.patch {
            Patch::AppSeen { pid, raw_id, name } => {
                let id = format!("a:{}", self.next_id + 1);
                if let std::collections::btree_map::Entry::Vacant(slot) = self.apps.entry(pid) {
                    self.next_id += 1;
                    slot.insert(AppRec {
                        id,
                        raw_id,
                        name,
                        via: via.clone(),
                    });
                    self.event(&via, "app.launched", serde_json::json!({}));
                }
            }
            Patch::AppGone { pid } => {
                if self.apps.remove(&pid).is_some() {
                    self.event(&via, "app.quit", serde_json::json!({}));
                }
            }
            Patch::WindowSeen {
                key,
                pid,
                title,
                fullscreen,
                private,
                capture_protected,
            } => {
                let wkey = (via.clone(), key);
                let cleaned = title.map(|t| privacy::clean(&t, self.cfg.redaction, self.cfg.text_cap));
                let fresh = self.fresh_id_if(!self.windows.contains_key(&wkey), "w");
                let rec = self.windows.entry(wkey).or_insert_with(|| WinRec {
                    id: fresh.unwrap_or_default(),
                    pid,
                    title: None,
                    fullscreen,
                    private,
                    capture_protected,
                    via: via.clone(),
                });
                rec.pid = pid;
                rec.fullscreen = fullscreen;
                rec.private = private;
                rec.capture_protected = capture_protected;
                if let Some(c) = cleaned {
                    rec.title = Some(Stored {
                        value: c.text,
                        via: via.clone(),
                        channel,
                        at_ms: now,
                        quality: Quality::Exact,
                        redacted: c.redacted,
                        truncated: c.truncated,
                    });
                }
            }
            Patch::WindowGone { key } => {
                let wkey = (via.clone(), key);
                self.windows.remove(&wkey);
                if self.focus.as_ref().is_some_and(|f| f.value == wkey) {
                    self.focus = None;
                    self.forget_content();
                }
            }
            Patch::FocusChanged { key } => self.focus_changed(key, &via, channel, now),
            Patch::Element {
                role,
                raw_role,
                label,
                editable,
                multiline,
                secure,
                value_len,
                caret,
            } => {
                let el = Element {
                    role,
                    raw_role,
                    label,
                    editable,
                    multiline,
                    secure,
                    value_len,
                    caret,
                };
                if self.accepts("element", self.element.as_ref(), channel, &via, now) {
                    self.element = Some(Stored::exact(el, &via, channel, now));
                }
            }
            Patch::Selection { text, len, quality } => {
                if let Some(t) = text {
                    if self.accepts("selection", self.selection_text.as_ref(), channel, &via, now) {
                        let c = privacy::clean(&t, self.cfg.redaction, self.cfg.text_cap);
                        let n = c.text.chars().count() as u32;
                        self.selection_text = Some(Stored {
                            value: c.text,
                            via: via.clone(),
                            channel,
                            at_ms: now,
                            quality,
                            redacted: c.redacted,
                            truncated: c.truncated,
                        });
                        self.selection_len = Some(Stored {
                            quality,
                            ..Stored::exact(n, &via, channel, now)
                        });
                    }
                } else if let Some(n) = len {
                    if self.accepts("selection", self.selection_len.as_ref(), channel, &via, now) {
                        self.selection_len = Some(Stored {
                            quality,
                            ..Stored::exact(n, &via, channel, now)
                        });
                        self.selection_text = None;
                        self.event(
                            &via,
                            "selection.changed",
                            serde_json::json!({ "len": n, "quality": quality }),
                        );
                    }
                }
            }
            // Clipboard text needs its own scope, which this slice does not grant: the text a
            // source may have sent is dropped here and never stored.
            Patch::Clipboard {
                kinds,
                size,
                text: _,
                concealed,
            } => {
                if self.accepts("clipboard", self.clipboard.as_ref(), channel, &via, now) {
                    self.event(
                        &via,
                        "clipboard.changed",
                        serde_json::json!({ "kinds": kinds, "size": size, "concealed": concealed }),
                    );
                    self.clipboard = Some(Stored::exact(ClipRec { kinds, size, concealed }, &via, channel, now));
                }
            }
            Patch::Document {
                url,
                path,
                title,
                no_context,
            } => {
                self.doc_no_context = no_context;
                let (cap, red) = (self.cfg.text_cap, self.cfg.redaction);
                let put = |slot: &mut Option<Stored<String>>, name: &str, v: Option<String>, this: &Engine| {
                    if let Some(v) = v {
                        if this.accepts(name, slot.as_ref(), channel, &via, now) {
                            let c = privacy::clean(&v, red, cap);
                            *slot = Some(Stored {
                                value: c.text,
                                via: via.clone(),
                                channel,
                                at_ms: now,
                                quality: Quality::Exact,
                                redacted: c.redacted,
                                truncated: c.truncated,
                            });
                        }
                    }
                };
                let (mut u, mut p, mut t) = (self.doc_url.take(), self.doc_path.take(), self.doc_title.take());
                put(&mut u, "document_url", url, self);
                put(&mut p, "document_path", path, self);
                put(&mut t, "document_title", title, self);
                (self.doc_url, self.doc_path, self.doc_title) = (u, p, t);
            }
            Patch::Pointer { x, y } => {
                if self.accepts("pointer", self.pointer.as_ref(), channel, &via, now) {
                    self.pointer = Some(Stored::exact(Pointer { x, y }, &via, channel, now));
                }
            }
            Patch::Activity { idle_ms, locked } => {
                if let Some(i) = idle_ms {
                    self.idle_ms = Some(Stored::exact(i, &via, channel, now));
                }
                if let Some(l) = locked {
                    let was = self.locked.as_ref().is_some_and(|s| s.value);
                    self.locked = Some(Stored::exact(l, &via, channel, now));
                    if l && !was {
                        // Lock clears what the engine knows, as the specification requires.
                        self.forget_content();
                        self.focus = None;
                        self.timeline.clear();
                        self.event(&via, "activity.locked", serde_json::json!({}));
                    } else if !l && was {
                        self.event(&via, "activity.unlocked", serde_json::json!({}));
                    }
                }
            }
        }
    }

    fn focus_changed(&mut self, key: Option<String>, via: &str, channel: Channel, now: u64) {
        if !self.accepts("focus", self.focus.as_ref(), channel, via, now) {
            return;
        }
        let Some(key) = key else {
            // Nothing focused (for instance the desktop itself). Keep the last target: a client
            // whose own surface took focus without being a window must still see what was there.
            return;
        };
        let wkey = (via.to_string(), key);
        let pid = self.windows.get(&wkey).map(|w| w.pid);
        // A client's own windows never become "what the user is doing".
        if pid.is_some_and(|p| self.own_pids.contains(&p)) {
            return;
        }
        if self.focus.as_ref().is_some_and(|f| f.value == wkey) {
            return;
        }
        self.forget_content();
        self.focus = Some(Stored::exact(wkey, via, channel, now));
        let category = pid.and_then(|p| self.apps.get(&p)).map(|a| self.classify(&a.raw_id).1);
        self.event(via, "focus.changed", serde_json::json!({ "to_category": category }));
    }

    /// What belongs to the focused target must not survive a change of target.
    fn forget_content(&mut self) {
        self.element = None;
        self.selection_text = None;
        self.selection_len = None;
        self.doc_url = None;
        self.doc_path = None;
        self.doc_title = None;
        self.doc_no_context = false;
    }

    fn rank(&self, field: &str, channel: Channel) -> usize {
        self.precedence
            .get(field)
            .and_then(|o| o.iter().position(|c| *c == channel))
            .unwrap_or(usize::MAX)
    }

    /// Precedence: a new value replaces the current one if it comes from the same source, from a
    /// channel ranked at least as high, or if the current one is stale.
    fn accepts<T>(&self, field: &str, current: Option<&Stored<T>>, channel: Channel, via: &str, now: u64) -> bool {
        match current {
            None => true,
            Some(cur) => {
                cur.via == via
                    || self.rank(field, channel) <= self.rank(field, cur.channel)
                    || now.saturating_sub(cur.at_ms) > STALE_MS
            }
        }
    }

    fn fresh_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}:{}", self.next_id)
    }

    fn fresh_id_if(&mut self, cond: bool, prefix: &str) -> Option<String> {
        cond.then(|| self.fresh_id(prefix))
    }

    fn event(&mut self, via: &str, kind: &str, data: serde_json::Value) {
        self.seq += 1;
        if self.cfg.level < Level::SessionAwareness && !kind.starts_with("source.") && !kind.starts_with("level.") {
            return;
        }
        let at = time::rfc3339((self.clock)());
        self.timeline.push_back(Event {
            seq: self.seq,
            at,
            via: via.into(),
            kind: kind.into(),
            subject: None,
            data,
        });
        while self.timeline.len() > TIMELINE_MAX {
            self.timeline.pop_front();
        }
    }

    fn classify(&self, raw_id: &str) -> (Option<String>, Category) {
        match self.profiles.lookup(raw_id) {
            Some(p) => (Some(p.canonical.clone()), p.category),
            None => (None, Category::Other),
        }
    }

    fn exclusion(&self, app: Option<&AppRec>, win: Option<&WinRec>) -> Option<ExcludedBy> {
        if let Some(w) = win {
            if w.capture_protected {
                return Some(ExcludedBy::CaptureProtected);
            }
            if w.private {
                return Some(ExcludedBy::PrivateWindow);
            }
        }
        let app = app?;
        let (canonical, category) = self.classify(&app.raw_id);
        let user = self
            .cfg
            .exclude_apps
            .iter()
            .any(|e| e.eq_ignore_ascii_case(&app.raw_id) || canonical.as_deref().is_some_and(|c| c == e));
        if user {
            Some(ExcludedBy::User)
        } else if category == Category::PasswordManager {
            Some(ExcludedBy::AppProfile)
        } else {
            None
        }
    }

    // ── Capture ───────────────────────────────────────────────────────────────────────────

    /// A consistent snapshot. At the on-request level the values are read from the sources for
    /// this call and are not retained afterwards.
    pub fn capture(&mut self, req: &CaptureRequest, sources: &[&dyn Source]) -> Snapshot {
        if self.cfg.level == Level::Paused {
            return self.empty_snapshot();
        }
        let wants = |s: &str| req.want.iter().any(|w| w == s);
        let mut signals: Vec<String> = Vec::new();
        if wants("focus") {
            signals.extend(["windows", "focus.element"].map(String::from));
        }
        for s in ["selection", "document", "clipboard", "pointer"] {
            if wants(s) {
                signals.push(s.to_string());
            }
        }

        // On request: work on a copy and throw it away. Session awareness: refresh the real state.
        let transient = self.cfg.level == Level::OnRequest;
        let mut scratch = transient.then(|| self.clone());
        let work: &mut Engine = scratch.as_mut().unwrap_or(self);

        let started = Instant::now();
        let mut timings: BTreeMap<SourceId, u32> = BTreeMap::new();
        let mut timed_out = false;
        for src in sources {
            let hello = src.hello();
            let asked: Vec<String> = signals.iter().filter(|s| hello.provides.contains(s)).cloned().collect();
            if asked.is_empty() {
                continue;
            }
            let Some(left) = req.deadline.checked_sub(started.elapsed()) else {
                timed_out = true;
                break;
            };
            let t0 = Instant::now();
            let patches = src.read_current(&asked, left);
            timings.insert(hello.id.clone(), t0.elapsed().as_millis() as u32);
            if started.elapsed() > req.deadline {
                // Late answers are not used: capture never returns stale-by-deadline data silently.
                timed_out = true;
                break;
            }
            let at = time::rfc3339((work.clock)());
            for patch in patches {
                work.apply(Observation {
                    via: hello.id.clone(),
                    channel: hello.channel,
                    at: at.clone(),
                    patch,
                    on_request: true,
                });
            }
        }

        let mut snap = work.snapshot(req, timed_out);
        snap.timings_ms = timings;

        let entry = AccessEntry {
            at: snap.at.clone(),
            client: req.client.clone(),
            method: "capture".into(),
            app: snap.focus.as_ref().and_then(|f| f.app.as_ref()).map(|a| {
                a.canonical
                    .clone()
                    .unwrap_or_else(|| serde_json::to_value(a.category).unwrap().as_str().unwrap().to_string())
            }),
            kinds: req.want.clone(),
            bytes: snapshot_content_bytes(&snap),
            redacted: snapshot_redacted(&snap),
            excluded: snap.focus.as_ref().is_some_and(|f| {
                f.app.as_ref().is_some_and(|a| a.excluded.is_some())
                    || f.window.as_ref().is_some_and(|w| w.excluded.is_some())
            }),
        };
        drop(scratch);
        self.seq += 1;
        self.ledger.push_back(entry);
        while self.ledger.len() > LEDGER_MAX {
            self.ledger.pop_front();
        }
        snap
    }

    fn empty_snapshot(&self) -> Snapshot {
        Snapshot {
            seq: self.seq,
            at: time::rfc3339((self.clock)()),
            level: self.cfg.level,
            session: None,
            activity: None,
            focus: None,
            selection: None,
            clipboard: None,
            document: None,
            pointer: None,
            recent: Field::unavailable(Reason::Level),
            timings_ms: BTreeMap::new(),
            target: None,
        }
    }

    fn provided(&self, signal: &str) -> bool {
        self.sources.values().any(|h| h.provides.iter().any(|p| p == signal))
    }

    /// Why a field that should be there is not.
    fn missing(&self, signal: &str, excluded: bool, timed_out: bool) -> Reason {
        if excluded {
            Reason::Excluded
        } else if !self.provided(signal) {
            Reason::NoSource
        } else if timed_out {
            Reason::Timeout
        } else {
            Reason::AppNotAccessible
        }
    }

    fn field<T: Clone>(&self, s: Option<&Stored<T>>, sens: Sensitivity, or: Reason) -> Field<T> {
        match s {
            Some(s) => Field::Known(Observed {
                value: s.value.clone(),
                via: s.via.clone(),
                at: time::rfc3339(s.at_ms),
                quality: s.quality,
                sensitivity: sens,
                truncated: s.truncated,
                redacted: s.redacted,
            }),
            None => Field::unavailable(or),
        }
    }

    fn snapshot(&self, req: &CaptureRequest, timed_out: bool) -> Snapshot {
        let wants = |s: &str| req.want.iter().any(|w| w == s);
        let win = self.focus.as_ref().and_then(|f| self.windows.get(&f.value));
        let app = win.and_then(|w| self.apps.get(&w.pid));
        let excluded_by = self.exclusion(app, win);
        let secure = self.element.as_ref().is_some_and(|e| e.value.secure);
        let doc_excluded = self.doc_no_context.then_some(ExcludedBy::ProviderHint);
        let content_excluded = excluded_by.is_some() || doc_excluded.is_some();

        let focus = wants("focus").then(|| FocusView {
            app: app.map(|a| {
                let (canonical, category) = self.classify(&a.raw_id);
                if excluded_by.is_some() {
                    // An excluded application is described by its category only.
                    App {
                        id: a.id.clone(),
                        raw_id: String::new(),
                        name: String::new(),
                        pid: 0,
                        canonical: None,
                        category,
                        excluded: excluded_by,
                        via: a.via.clone(),
                    }
                } else {
                    let pid = self
                        .apps
                        .iter()
                        .find(|(_, r)| r.id == a.id)
                        .map(|(p, _)| *p)
                        .unwrap_or(0);
                    App {
                        id: a.id.clone(),
                        raw_id: a.raw_id.clone(),
                        name: a.name.clone(),
                        pid,
                        canonical,
                        category,
                        excluded: None,
                        via: a.via.clone(),
                    }
                }
            }),
            window: win.map(|w| Window {
                id: w.id.clone(),
                app: app.map(|a| a.id.clone()).unwrap_or_default(),
                title: if excluded_by.is_some() {
                    Field::unavailable(Reason::Excluded)
                } else {
                    self.field(
                        w.title.as_ref(),
                        Sensitivity::Personal,
                        self.missing("windows", false, timed_out),
                    )
                },
                fullscreen: w.fullscreen,
                private: w.private,
                excluded: excluded_by,
                via: w.via.clone(),
            }),
            element: if excluded_by.is_some() {
                Field::unavailable(Reason::Excluded)
            } else {
                self.field(
                    self.element.as_ref(),
                    Sensitivity::Personal,
                    self.missing("focus.element", false, timed_out),
                )
            },
            since: self.focus.as_ref().map(|f| time::rfc3339(f.at_ms)),
        });

        let selection = wants("selection").then(|| {
            let blocked = content_excluded || secure;
            Selection {
                text: if blocked {
                    Field::unavailable(Reason::Excluded)
                } else {
                    self.field(
                        self.selection_text.as_ref(),
                        Sensitivity::Content,
                        self.missing("selection", false, timed_out),
                    )
                },
                len: if blocked {
                    Field::unavailable(Reason::Excluded)
                } else {
                    self.field(
                        self.selection_len.as_ref(),
                        Sensitivity::System,
                        self.missing("selection", false, timed_out),
                    )
                },
                owner_window: None,
            }
        });

        let document = wants("document").then(|| {
            let r = |s: Option<&Stored<String>>, name: &str| {
                if content_excluded {
                    Field::unavailable(Reason::Excluded)
                } else if s.is_none() && self.provided("document") && !timed_out {
                    Field::unavailable(Reason::NotApplicable)
                } else {
                    self.field(s, Sensitivity::Personal, self.missing(name, false, timed_out))
                }
            };
            Document {
                url: r(self.doc_url.as_ref(), "document"),
                path: r(self.doc_path.as_ref(), "document"),
                title: r(self.doc_title.as_ref(), "document"),
                excluded: doc_excluded.or(excluded_by),
            }
        });

        let clipboard = wants("clipboard").then(|| {
            let c = self.clipboard.as_ref();
            let miss = self.missing("clipboard", false, timed_out);
            Clipboard {
                kinds: self.field(
                    c.map(|s| s.map(|v| v.kinds.clone())).as_ref(),
                    Sensitivity::System,
                    miss,
                ),
                size: self.field(c.map(|s| s.map(|v| v.size)).as_ref(), Sensitivity::System, miss),
                // Clipboard text needs its own scope; this slice never serves it.
                text: Field::unavailable(Reason::NotRequested),
                concealed: c.is_some_and(|s| s.value.concealed),
                changed_at: c.map(|s| time::rfc3339(s.at_ms)),
            }
        });

        let recent = if self.cfg.level >= Level::SessionAwareness {
            let events: Vec<Event> = self
                .timeline
                .iter()
                .rev()
                .take(req.recent_limit)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            Field::Known(Observed {
                value: events,
                via: "sense".into(),
                at: time::rfc3339((self.clock)()),
                quality: Quality::Exact,
                sensitivity: Sensitivity::Personal,
                truncated: false,
                redacted: false,
            })
        } else {
            Field::unavailable(Reason::Level)
        };

        Snapshot {
            seq: self.seq,
            at: time::rfc3339((self.clock)()),
            level: self.cfg.level,
            session: wants("session").then(|| self.session.clone()),
            activity: wants("activity").then(|| Activity {
                idle_ms: self.field(
                    self.idle_ms.as_ref(),
                    Sensitivity::System,
                    self.missing("activity", false, false),
                ),
                locked: self.field(
                    self.locked.as_ref(),
                    Sensitivity::System,
                    self.missing("activity", false, false),
                ),
            }),
            focus,
            selection,
            clipboard,
            document,
            pointer: wants("pointer").then(|| {
                self.field(
                    self.pointer.as_ref(),
                    Sensitivity::System,
                    self.missing("pointer", false, timed_out),
                )
            }),
            recent,
            timings_ms: BTreeMap::new(),
            target: win.map(|w| format!("t:{}", w.id)),
        }
    }
}

impl<T> Stored<T> {
    fn exact(value: T, via: &str, channel: Channel, at_ms: u64) -> Self {
        Stored {
            value,
            via: via.into(),
            channel,
            at_ms,
            quality: Quality::Exact,
            redacted: false,
            truncated: false,
        }
    }
    fn map<U>(&self, f: impl FnOnce(&T) -> U) -> Stored<U> {
        Stored {
            value: f(&self.value),
            via: self.via.clone(),
            channel: self.channel,
            at_ms: self.at_ms,
            quality: self.quality,
            redacted: self.redacted,
            truncated: self.truncated,
        }
    }
}

fn snapshot_content_bytes(s: &Snapshot) -> u64 {
    let mut n = 0usize;
    if let Some(f) = &s.focus {
        n += f.window.as_ref().and_then(|w| w.title.value()).map_or(0, String::len);
    }
    if let Some(sel) = &s.selection {
        n += sel.text.value().map_or(0, String::len);
    }
    if let Some(d) = &s.document {
        for f in [&d.url, &d.path, &d.title] {
            n += f.value().map_or(0, String::len);
        }
    }
    n as u64
}

fn snapshot_redacted(s: &Snapshot) -> bool {
    let sel = s
        .selection
        .as_ref()
        .and_then(|x| x.text.known())
        .is_some_and(|o| o.redacted);
    let title = s
        .focus
        .as_ref()
        .and_then(|f| f.window.as_ref())
        .and_then(|w| w.title.known())
        .is_some_and(|o| o.redacted);
    sel || title
}

/// A session description for tests and for sources that have not reported one yet.
pub fn unknown_session(os: Os) -> Session {
    Session {
        os,
        os_version: String::new(),
        display_server: String::new(),
        desktop: None,
        desktop_version: None,
        family: None,
    }
}
