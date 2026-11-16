use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sense_core::{card, CaptureRequest, Config, Engine, Source};
use sense_schema::protocol::{Observation, Patch, ReadResult, SourceHello};
use sense_schema::{Level, Os, Session};
use serde_json::{json, Value};

/// An extension connected over the socket, seen by the engine as an ordinary source.
#[derive(Clone)]
struct Remote {
    hello: SourceHello,
    writer: Arc<Mutex<UnixStream>>,
    pending: Arc<Mutex<HashMap<u64, Sender<Vec<Patch>>>>>,
    next_id: Arc<AtomicU64>,
}

impl Source for Remote {
    fn hello(&self) -> SourceHello {
        self.hello.clone()
    }

    fn read_current(&self, signals: &[String], deadline: Duration) -> Vec<Patch> {
        let request_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();
        self.pending.lock().unwrap().insert(request_id, tx);
        let msg = json!({ "jsonrpc": "2.0", "method": "source.read", "params": {
            "request_id": request_id, "signals": signals, "deadline_ms": deadline.as_millis() as u64 } });
        let sent = send_line(&self.writer, &msg);
        let patches = if sent {
            rx.recv_timeout(deadline).unwrap_or_default()
        } else {
            Vec::new()
        };
        self.pending.lock().unwrap().remove(&request_id);
        patches
    }
}

struct Shared {
    engine: Engine,
    remotes: BTreeMap<String, Remote>,
    /// In-process platform sources, connected lazily (their service may start after the daemon).
    local: Vec<Arc<dyn Source + Sync>>,
}

/// Connect the in-process sources that are not connected yet. Cheap when there is nothing to do.
fn connect_local(shared: &mut Shared) {
    #[cfg(target_os = "linux")]
    if !shared.local.iter().any(|s| s.hello().id == "linux.atspi") {
        if let Some(atspi) = sense_platform_linux::atspi::Atspi::connect() {
            shared.engine.register_source(atspi.hello());
            shared.local.push(Arc::new(atspi));
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = shared;
}

fn send_line(writer: &Arc<Mutex<UnixStream>>, value: &Value) -> bool {
    let mut line = value.to_string();
    line.push('\n');
    writer.lock().unwrap().write_all(line.as_bytes()).is_ok()
}

fn linux_session() -> Session {
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let desktop = env("XDG_CURRENT_DESKTOP").map(|d| d.split(':').next_back().unwrap_or("").to_lowercase());
    let display = env("XDG_SESSION_TYPE").unwrap_or_else(|| "unknown".into());
    let family = match (display.as_str(), desktop.as_deref()) {
        ("x11", _) => "x11",
        (_, Some("gnome")) => "gnome",
        (_, Some("kde")) => "kde",
        _ => "unknown",
    };
    Session {
        os: if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        },
        os_version: String::new(),
        display_server: display,
        desktop,
        desktop_version: None,
        family: Some(family.into()),
    }
}

fn default_socket() -> PathBuf {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("sense").join("sense.sock")
}

pub fn run() -> ExitCode {
    let mut socket = default_socket();
    let mut level = Level::OnRequest;
    let mut dev_no_indicator = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--socket" => socket = args.next().map(PathBuf::from).unwrap_or(socket),
            "--level" => {
                level = match args.next().as_deref() {
                    Some("paused") => Level::Paused,
                    Some("session_awareness") => Level::SessionAwareness,
                    _ => Level::OnRequest,
                }
            }
            "--dev-no-indicator" => dev_no_indicator = true,
            other => {
                eprintln!("sensed: unknown argument {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    // Invariant 12: a visible indicator whenever a source runs. The indicator does not exist yet,
    // so the daemon refuses to run unless the developer says, explicitly, that they know.
    if !dev_no_indicator {
        eprintln!(
            "sensed: the indicator is not implemented yet. This build must not observe a real user.\n\
             For development only, pass --dev-no-indicator."
        );
        return ExitCode::FAILURE;
    }
    eprintln!(
        "sensed: DEVELOPMENT BUILD, no indicator. Level: {level:?}. Socket: {}",
        socket.display()
    );

    if let Some(dir) = socket.parent() {
        let _ = std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir);
    }
    if UnixStream::connect(&socket).is_ok() {
        eprintln!("sensed: another instance is already serving {}", socket.display());
        return ExitCode::FAILURE;
    }
    let _ = std::fs::remove_file(&socket);
    let listener = match UnixListener::bind(&socket) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("sensed: cannot bind {}: {e}", socket.display());
            return ExitCode::FAILURE;
        }
    };
    let _ = std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600));

    let cfg = Config {
        level,
        ..Config::default()
    };
    let shared = Arc::new(Mutex::new(Shared {
        engine: Engine::new(cfg, linux_session()),
        remotes: BTreeMap::new(),
        local: Vec::new(),
    }));
    connect_local(&mut shared.lock().unwrap());

    for stream in listener.incoming().flatten() {
        let shared = shared.clone();
        std::thread::spawn(move || connection(stream, shared));
    }
    ExitCode::SUCCESS
}

fn reply(writer: &Arc<Mutex<UnixStream>>, id: &Value, result: Result<Value, (&str, String)>) {
    let msg = match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err((code, message)) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }),
    };
    send_line(writer, &msg);
}

fn connection(stream: UnixStream, shared: Arc<Mutex<Shared>>) {
    let Ok(read_half) = stream.try_clone() else { return };
    let writer = Arc::new(Mutex::new(stream));
    let mut lines = BufReader::new(read_half).lines();

    // The first message decides the role.
    let Some(Ok(first)) = lines.next() else { return };
    let Ok(msg) = serde_json::from_str::<Value>(&first) else {
        return;
    };
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    match msg.get("method").and_then(Value::as_str) {
        Some("client.hello") => {
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let pids: Vec<u32> = params
                .get("own_pids")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_u64().map(|n| n as u32)).collect())
                .unwrap_or_default();
            shared.lock().unwrap().engine.register_own_pids(&pids);
            reply(
                &writer,
                &id,
                Ok(json!({ "protocol": "1", "daemon_version": env!("CARGO_PKG_VERSION") })),
            );
            reader_loop(lines, &writer, &shared, &name);
        }
        Some("source.hello") => {
            let Ok(hello) = serde_json::from_value::<SourceHello>(params) else {
                reply(&writer, &id, Err(("bad_request", "malformed source.hello".into())));
                return;
            };
            let remote = Remote {
                hello: hello.clone(),
                writer: writer.clone(),
                pending: Arc::new(Mutex::new(HashMap::new())),
                next_id: Arc::new(AtomicU64::new(1)),
            };
            let level = {
                let mut s = shared.lock().unwrap();
                s.engine.register_source(hello.clone());
                s.remotes.insert(hello.id.clone(), remote.clone());
                s.engine.level()
            };
            reply(
                &writer,
                &id,
                Ok(json!({ "accepted": true, "config": { "level": level } })),
            );
            source_loop(lines, &shared, &remote);
            let mut s = shared.lock().unwrap();
            s.remotes.remove(&hello.id);
            s.engine.unregister_source(&hello.id);
        }
        _ => reply(
            &writer,
            &id,
            Err((
                "bad_request",
                "first message must be client.hello or source.hello".into(),
            )),
        ),
    }
}

fn source_loop(lines: std::io::Lines<BufReader<UnixStream>>, shared: &Arc<Mutex<Shared>>, remote: &Remote) {
    for line in lines.map_while(Result::ok) {
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        match msg.get("method").and_then(Value::as_str) {
            Some("source.observe") => {
                let patches: Vec<Patch> =
                    serde_json::from_value(params.get("patches").cloned().unwrap_or(Value::Null)).unwrap_or_default();
                let mut s = shared.lock().unwrap();
                let at = sense_core::time::rfc3339(sense_core::time::now_ms());
                for patch in patches {
                    // A source may only report what it declared.
                    if !remote
                        .hello
                        .provides
                        .iter()
                        .any(|p| p == patch.signal() || patch.signal().starts_with(p.as_str()))
                    {
                        continue;
                    }
                    s.engine.apply(Observation {
                        via: remote.hello.id.clone(),
                        channel: remote.hello.channel,
                        at: at.clone(),
                        patch,
                        on_request: false,
                    });
                }
            }
            Some("source.result") => {
                if let Ok(res) = serde_json::from_value::<ReadResult>(params) {
                    if let Some(tx) = remote.pending.lock().unwrap().get(&res.request_id) {
                        let _ = tx.send(res.patches);
                    }
                }
            }
            _ => {}
        }
    }
}

fn reader_loop(
    lines: std::io::Lines<BufReader<UnixStream>>,
    writer: &Arc<Mutex<UnixStream>>,
    shared: &Arc<Mutex<Shared>>,
    client: &str,
) {
    for line in lines.map_while(Result::ok) {
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        let result = match msg.get("method").and_then(Value::as_str).unwrap_or("") {
            "capabilities.get" => Ok(serde_json::to_value(shared.lock().unwrap().engine.capabilities()).unwrap()),
            "capture" => {
                let mut req = CaptureRequest {
                    client: client.to_string(),
                    ..CaptureRequest::default()
                };
                if let Some(w) = params.get("want").and_then(Value::as_array) {
                    req.want = w.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                }
                if let Some(d) = params.get("deadline_ms").and_then(Value::as_u64) {
                    req.deadline = Duration::from_millis(d.min(5_000));
                }
                let mut s = shared.lock().unwrap();
                connect_local(&mut s);
                let remotes: Vec<Remote> = s.remotes.values().cloned().collect();
                let local = s.local.clone();
                // Extensions first (the compositor knows the window), then in-process sources.
                let mut sources: Vec<&dyn Source> = remotes.iter().map(|r| r as &dyn Source).collect();
                sources.extend(local.iter().map(|l| l.as_ref() as &dyn Source));
                let snap = s.engine.capture(&req, &sources);
                if params.get("format").and_then(Value::as_str) == Some("card") {
                    Ok(json!({ "card": card::render(&snap) }))
                } else {
                    Ok(serde_json::to_value(snap).unwrap())
                }
            }
            "privacy.get" => Ok(json!({ "level": shared.lock().unwrap().engine.level() })),
            // Not enforced yet: raising the level must require the admin scope and a consent screen.
            "privacy.set" => match serde_json::from_value::<Level>(params.get("level").cloned().unwrap_or(Value::Null))
            {
                Ok(level) => {
                    shared.lock().unwrap().engine.set_level(level);
                    Ok(json!({ "level": level }))
                }
                Err(_) => Err((
                    "bad_request",
                    "level must be paused, on_request or session_awareness".into(),
                )),
            },
            "access.log" => {
                let s = shared.lock().unwrap();
                Ok(serde_json::to_value(s.engine.ledger().collect::<Vec<_>>()).unwrap())
            }
            "timeline.query" => {
                let s = shared.lock().unwrap();
                let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
                let all: Vec<_> = s.engine.timeline().collect();
                let from = all.len().saturating_sub(limit);
                Ok(serde_json::to_value(&all[from..]).unwrap())
            }
            other => Err(("bad_request", format!("unknown method {other}"))),
        };
        reply(writer, &id, result);
    }
}
