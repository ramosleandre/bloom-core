//! `sense`: the reference client of the Sense daemon. It only speaks the client API.

use std::process::ExitCode;

#[cfg(unix)]
fn main() -> ExitCode {
    unix::run()
}

#[cfg(not(unix))]
fn main() -> ExitCode {
    eprintln!("sense: this slice only supports Unix sockets");
    ExitCode::FAILURE
}

#[cfg(unix)]
mod unix {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::process::ExitCode;

    use serde_json::{json, Value};

    const USAGE: &str = "usage: sense [--socket PATH] <command>
  caps                               what this session can observe
  capture [--want a,b] [--deadline MS] [--card]
                                     the context, now (default: session,focus,selection,document)
  level [paused|on_request|session_awareness]
                                     show or set the observation level
  log                                the access ledger: who read what, never the content
  timeline                           recent events (session awareness only)";

    fn call(
        stream: &mut UnixStream,
        reader: &mut BufReader<UnixStream>,
        method: &str,
        params: Value,
    ) -> Result<Value, String> {
        let msg = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        writeln!(stream, "{msg}").map_err(|e| e.to_string())?;
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_str(&line).map_err(|e| format!("bad reply: {e}"))?;
        match v.get("error") {
            Some(e) => Err(e.to_string()),
            None => Ok(v.get("result").cloned().unwrap_or(Value::Null)),
        }
    }

    pub fn run() -> ExitCode {
        let mut args: Vec<String> = std::env::args().skip(1).collect();
        let mut socket = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("sense")
            .join("sense.sock");
        if args.first().map(String::as_str) == Some("--socket") && args.len() >= 2 {
            socket = PathBuf::from(args.remove(1));
            args.remove(0);
        }
        let Some(command) = args.first().cloned() else {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        };
        let flag = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| args.get(i + 1))
                .cloned()
        };

        let mut stream = match UnixStream::connect(&socket) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sense: cannot reach the daemon at {}: {e}", socket.display());
                return ExitCode::FAILURE;
            }
        };
        let mut reader = BufReader::new(stream.try_clone().expect("clone socket"));
        let hello = json!({ "name": "sense-cli", "version": env!("CARGO_PKG_VERSION"), "role": "reader" });
        if let Err(e) = call(&mut stream, &mut reader, "client.hello", hello) {
            eprintln!("sense: handshake failed: {e}");
            return ExitCode::FAILURE;
        }

        let result = match command.as_str() {
            "caps" => call(&mut stream, &mut reader, "capabilities.get", json!({})),
            "capture" => {
                let mut params = json!({});
                if let Some(w) = flag("--want") {
                    params["want"] = json!(w.split(',').collect::<Vec<_>>());
                }
                if let Some(d) = flag("--deadline").and_then(|d| d.parse::<u64>().ok()) {
                    params["deadline_ms"] = json!(d);
                }
                if args.iter().any(|a| a == "--card") {
                    params["format"] = json!("card");
                }
                call(&mut stream, &mut reader, "capture", params)
            }
            "level" => match args.get(1) {
                Some(l) => call(&mut stream, &mut reader, "privacy.set", json!({ "level": l })),
                None => call(&mut stream, &mut reader, "privacy.get", json!({})),
            },
            "log" => call(&mut stream, &mut reader, "access.log", json!({})),
            "timeline" => call(&mut stream, &mut reader, "timeline.query", json!({})),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
        };
        match result {
            Ok(v) => {
                match v.get("card").and_then(Value::as_str) {
                    Some(card) => print!("{card}"),
                    None => println!("{}", serde_json::to_string_pretty(&v).unwrap()),
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("sense: {e}");
                ExitCode::FAILURE
            }
        }
    }
}
