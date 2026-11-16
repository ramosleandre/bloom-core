//! `sensed`: the Sense daemon. One per user session.
//!
//! This first slice serves the client API and the source protocol over a user-only Unix socket
//! (`sense/spec/03-client-api.md`, `02-sources.md`). Not implemented yet, and stated here so that
//! nobody mistakes this for a shippable build: scopes and tokens, peer-credential checks, the
//! indicator, per-application approval, deep content reads, and every in-process platform source.

#[cfg(unix)]
mod server;

#[cfg(unix)]
fn main() -> std::process::ExitCode {
    server::run()
}

#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("sensed: this slice only supports Unix sockets; the named-pipe transport is not written yet");
    std::process::ExitCode::FAILURE
}
