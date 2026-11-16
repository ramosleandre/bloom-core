//! `linux.atspi`: the focused element and its selection, through AT-SPI2 over D-Bus.
//!
//! It keeps one thing between reads: a reference to the accessible that last received focus,
//! learned from the bus's focus events. A reference is an address, not content. Reads, which the
//! engine makes for a capture the user started, ask that one element for its role, state and
//! selected text. Walking an application's tree to find the focus costs about a millisecond per
//! node, so the walk is only a bounded fallback for the time before the first focus event.
//!
//! Rules kept here (`sense/spec/04-privacy-security.md`):
//! - a password field is reported as `secure` and its text is never requested;
//! - every D-Bus call has a short timeout, so a hung application cannot stall a capture;
//! - the only subscription is to focus changes; the desktop-wide accessibility switch is never touched,
//!   so applications that build their tree only for assistive tools are simply not seen.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sense_core::Source;
use sense_schema::protocol::{Patch, SourceHello};
use sense_schema::{Channel, Quality, Role};
use zbus::blocking::connection::Builder;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, Structure, Value};

const REGISTRY: &str = "org.a11y.atspi.Registry";
const ROOT: &str = "/org/a11y/atspi/accessible/root";
const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";
const TEXT: &str = "org.a11y.atspi.Text";
const CALL_TIMEOUT: Duration = Duration::from_millis(100);
/// Upper bound on accessibles visited to find the focused one.
const NODE_BUDGET: usize = 600;

// Bit positions in the AT-SPI state set.
const STATE_ACTIVE: u32 = 1;
const STATE_EDITABLE: u32 = 7;
const STATE_FOCUSED: u32 = 12;
const STATE_MULTI_LINE: u32 = 17;

type Ref = (String, OwnedObjectPath);

pub struct Atspi {
    bus: Connection,
    /// The accessible that last gained focus, from `object:state-changed:focused` events.
    last_focused: Arc<Mutex<Option<Ref>>>,
}

impl Atspi {
    /// Connect to the accessibility bus. `None` when the session has none: the source is then
    /// simply not available, which the capabilities report.
    pub fn connect() -> Option<Self> {
        let session = Connection::session().ok()?;
        let reply = session
            .call_method(
                Some("org.a11y.Bus"),
                "/org/a11y/bus",
                Some("org.a11y.Bus"),
                "GetAddress",
                &(),
            )
            .ok()?;
        let address: String = reply.body().deserialize().ok()?;
        let bus = Builder::address(address.as_str())
            .ok()?
            .method_timeout(CALL_TIMEOUT)
            .build()
            .ok()?;
        let last_focused = Arc::new(Mutex::new(None));
        watch_focus(bus.clone(), last_focused.clone());
        Some(Atspi { bus, last_focused })
    }

    fn call<B, R>(&self, target: &Ref, interface: &str, method: &str, body: &B) -> Option<R>
    where
        B: serde::Serialize + zbus::zvariant::DynamicType,
        R: for<'d> zbus::zvariant::DynamicDeserialize<'d>,
    {
        let reply = self
            .bus
            .call_method(
                Some(target.0.as_str()),
                target.1.as_str(),
                Some(interface),
                method,
                body,
            )
            .ok()?;
        reply.body().deserialize::<R>().ok()
    }

    fn children(&self, of: &Ref) -> Vec<Ref> {
        self.call::<_, Vec<Ref>>(of, ACCESSIBLE, "GetChildren", &())
            .unwrap_or_default()
    }

    fn has_state(&self, of: &Ref, bit: u32) -> bool {
        let set: Vec<u32> = self.call(of, ACCESSIBLE, "GetState", &()).unwrap_or_default();
        set.get((bit / 32) as usize)
            .is_some_and(|word| word & (1 << (bit % 32)) != 0)
    }

    fn states(&self, of: &Ref) -> Vec<u32> {
        self.call(of, ACCESSIBLE, "GetState", &()).unwrap_or_default()
    }

    /// The focused accessible: inside the active window if one says it is active, otherwise the
    /// first focused accessible found. Breadth-first, bounded.
    fn focused(&self, deadline: Instant) -> Option<Ref> {
        // The usual case: the element named by the last focus event, if it still has focus.
        let remembered = self.last_focused.lock().unwrap().clone();
        if let Some(el) = remembered {
            if self.has_state(&el, STATE_FOCUSED) {
                return Some(el);
            }
        }
        let root: Ref = (REGISTRY.to_string(), OwnedObjectPath::try_from(ROOT).ok()?);
        let mut windows: Vec<Ref> = Vec::new();
        for app in self.children(&root) {
            if Instant::now() >= deadline {
                return None;
            }
            windows.extend(self.children(&app));
        }
        // Active windows first.
        let (active, rest): (Vec<Ref>, Vec<Ref>) = windows.into_iter().partition(|w| self.has_state(w, STATE_ACTIVE));
        let mut queue: std::collections::VecDeque<Ref> = active.into_iter().chain(rest).collect();
        let mut visited = 0;
        while let Some(node) = queue.pop_front() {
            if visited >= NODE_BUDGET || Instant::now() >= deadline {
                return None;
            }
            visited += 1;
            if self.has_state(&node, STATE_FOCUSED) && self.children(&node).is_empty() {
                return Some(node);
            }
            queue.extend(self.children(&node));
        }
        None
    }

    fn describe(&self, el: &Ref, want_selection: bool) -> Vec<Patch> {
        let raw_role: String = self.call(el, ACCESSIBLE, "GetRoleName", &()).unwrap_or_default();
        let states = self.states(el);
        let bit = |b: u32| states.get((b / 32) as usize).is_some_and(|w| w & (1 << (b % 32)) != 0);
        let secure = raw_role == "password text";
        let editable = bit(STATE_EDITABLE);
        let multiline = bit(STATE_MULTI_LINE);
        let role = match raw_role.as_str() {
            "password text" => Role::SecureTextField,
            "entry" | "text" if !multiline => Role::TextField,
            "text" | "paragraph" if editable => Role::TextArea,
            "terminal" => Role::Terminal,
            "document web" | "document frame" | "document text" => Role::Document,
            "push button" | "button" => Role::Button,
            "link" => Role::Link,
            "heading" => Role::Heading,
            "paragraph" => Role::Paragraph,
            "list item" => Role::ListItem,
            "table cell" => Role::Cell,
            _ => Role::Unknown,
        };
        let mut patches = vec![Patch::Element {
            role,
            raw_role,
            label: None,
            editable,
            multiline,
            secure,
            value_len: None,
            caret: None,
        }];
        // A secure field is described, never read.
        if want_selection && !secure {
            if let Some(n) = self.call::<_, i32>(el, TEXT, "GetNSelections", &()) {
                let text = if n > 0 {
                    self.call::<_, (i32, i32)>(el, TEXT, "GetSelection", &(0i32,))
                        .and_then(|(start, end)| self.call::<_, String>(el, TEXT, "GetText", &(start, end)))
                } else {
                    Some(String::new())
                };
                if let Some(text) = text {
                    patches.push(Patch::Selection {
                        text: Some(text),
                        len: None,
                        quality: Quality::Exact,
                    });
                }
            }
        }
        patches
    }
}

/// Listen for focus changes on the accessibility bus and remember who has focus.
fn watch_focus(bus: Connection, slot: Arc<Mutex<Option<Ref>>>) {
    // Tell the registry someone listens, or applications do not emit the event. The argument list
    // grew over AT-SPI versions; try the current form, then the older one.
    let registry = "/org/a11y/atspi/registry";
    let event = "object:state-changed:focused";
    let name = bus.unique_name().map(|n| n.to_string()).unwrap_or_default();
    let current = bus.call_method(
        Some(REGISTRY),
        registry,
        Some(REGISTRY),
        "RegisterEvent",
        &(event, Vec::<String>::new(), name.as_str()),
    );
    if current.is_err() {
        let _ = bus.call_method(Some(REGISTRY), registry, Some(REGISTRY), "RegisterEvent", &(event,));
    }
    let Ok(rule) = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .interface("org.a11y.atspi.Event.Object")
        .and_then(|b| b.member("StateChanged"))
        .map(|b| b.build())
    else {
        return;
    };
    let Ok(events) = zbus::blocking::MessageIterator::for_match_rule(rule, &bus, Some(64)) else {
        return;
    };
    std::thread::spawn(move || {
        for msg in events.flatten() {
            // Body: (detail, detail1, detail2, any, properties). Only the first two matter here,
            // and the tail changed across versions, so read the fields dynamically.
            let body = msg.body();
            let Ok(fields) = body.deserialize::<Structure>() else {
                continue;
            };
            let fields = fields.fields();
            let gained = matches!(
                (fields.first(), fields.get(1)),
                (Some(Value::Str(detail)), Some(Value::I32(1))) if detail.as_str() == "focused"
            );
            if !gained {
                continue;
            }
            let header = msg.header();
            if let (Some(sender), Some(path)) = (header.sender(), header.path()) {
                *slot.lock().unwrap() = Some((sender.to_string(), path.to_owned().into()));
            }
        }
    });
}

impl Source for Atspi {
    fn hello(&self) -> SourceHello {
        SourceHello {
            id: "linux.atspi".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            channel: Channel::Accessibility,
            provides: vec!["focus.element".into(), "selection".into()],
        }
    }

    fn read_current(&self, signals: &[String], deadline: Duration) -> Vec<Patch> {
        let until = Instant::now() + deadline;
        match self.focused(until) {
            Some(el) => self.describe(&el, signals.iter().any(|s| s == "selection")),
            None => Vec::new(),
        }
    }
}
