//! Optional Linux adapter. The shared UI and other OS builds do not depend on GNOME.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread::JoinHandle;
use std::time::Duration;

use dbus::{blocking::SyncConnection, message::MatchRule};
use tauri::PhysicalPosition;

const BUS: &str = "com.bloom.Desktop";
const PATH: &str = "/com/bloom/Desktop";
const TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pointer {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Pointer {
    fn from_tuple((x, y, width, height): (i32, i32, u32, u32)) -> Self {
        Self { x, y, width, height }
    }

    pub fn to_root(self, width: u32, height: u32) -> Option<PhysicalPosition<f64>> {
        if self.width == 0 || self.height == 0 || width == 0 || height == 0 {
            return None;
        }
        Some(PhysicalPosition::new(
            f64::from(self.x) * f64::from(width) / f64::from(self.width),
            f64::from(self.y) * f64::from(height) / f64::from(self.height),
        ))
    }
}

pub struct Adapter {
    requests: mpsc::Sender<Request>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

enum Request {
    Pointer(mpsc::Sender<Result<Pointer, String>>),
    Enabled(bool, mpsc::Sender<Result<bool, String>>),
}

impl Adapter {
    pub fn connect(on_trigger: impl Fn(Pointer) + Send + Sync + 'static) -> Result<Self, String> {
        let connection = SyncConnection::new_session().map_err(|error| error.to_string())?;
        let proxy = connection.with_proxy(BUS, PATH, TIMEOUT);
        let (version,): (u32,) = proxy.method_call(BUS, "GetVersion", ())
            .map_err(|error| error.to_string())?;
        if version != 1 {
            return Err(format!("Unsupported GNOME adapter version {version}"));
        }
        connection.add_match(
            MatchRule::new_signal(BUS, "Trigger").with_sender(BUS).with_path(PATH),
            move |pointer: (i32, i32, u32, u32), _, _| {
                on_trigger(Pointer::from_tuple(pointer));
                true
            },
        ).map_err(|error| error.to_string())?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let (requests, pending) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            // All reads and RPCs use one thread: processing signals concurrently with a
            // blocking RPC on the same D-Bus connection can consume that RPC's reply.
            while !stopped.load(Ordering::Relaxed) {
                for request in pending.try_iter() {
                    let proxy = connection.with_proxy(BUS, PATH, TIMEOUT);
                    match request {
                        Request::Pointer(reply) => {
                            let result = proxy.method_call(BUS, "GetPointer", ())
                                .map(Pointer::from_tuple).map_err(|error| error.to_string());
                            let _ = reply.send(result);
                        }
                        Request::Enabled(enabled, reply) => {
                            let result = proxy.method_call(BUS, "SetShortcutEnabled", (enabled,))
                                .map(|(registered,): (bool,)| registered)
                                .map_err(|error| error.to_string());
                            let _ = reply.send(result);
                        }
                    }
                }
                if connection.process(Duration::from_millis(30)).is_err() {
                    break;
                }
            }
        });
        Ok(Self { requests, stop, worker: Some(worker) })
    }

    pub fn pointer(&self) -> Result<Pointer, String> {
        let (reply, response) = mpsc::channel();
        self.requests.send(Request::Pointer(reply)).map_err(|error| error.to_string())?;
        response.recv_timeout(Duration::from_secs(1)).map_err(|error| error.to_string())?
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<bool, String> {
        let (reply, response) = mpsc::channel();
        self.requests.send(Request::Enabled(enabled, reply)).map_err(|error| error.to_string())?;
        response.recv_timeout(Duration::from_secs(1)).map_err(|error| error.to_string())?
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // Shutdown can run on GTK's main thread while the receiver awaits a GUI callback.
        // Detach instead of joining here. Closing the connection releases the Shell lease.
        self.worker.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_stage_coordinates_to_xwayland_root_without_window_offsets() {
        let pointer = Pointer { x: 1500, y: 700, width: 1920, height: 1080 };
        assert_eq!(pointer.to_root(1920, 1080), Some(PhysicalPosition::new(1500.0, 700.0)));
        assert_eq!(pointer.to_root(3840, 2160), Some(PhysicalPosition::new(3000.0, 1400.0)));
        assert_eq!(Pointer { x: -200, ..pointer }.to_root(3840, 2160), Some(PhysicalPosition::new(-400.0, 1400.0)));
        assert_eq!(Pointer { width: 0, ..pointer }.to_root(1920, 1080), None);
    }

    #[test]
    #[ignore = "run with npm run test:gnome-native (private D-Bus and GJS required)"]
    fn session_bus_contract() {
        use std::process::{Child, Command};
        struct Fixture(Child);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/gnome-dbus-fixture.js");
        let _fixture = Fixture(Command::new("gjs").arg("-m").arg(script).spawn().unwrap());
        let (send, received) = std::sync::mpsc::channel();
        let mut adapter = None;
        for _ in 0..50 {
            let send = send.clone();
            if let Ok(client) = Adapter::connect(move |pointer| { let _ = send.send(pointer); }) {
                adapter = Some(client);
                break;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        let adapter = adapter.expect("GJS service did not start");
        assert_eq!(adapter.pointer().unwrap(), Pointer { x: 1500, y: 700, width: 1920, height: 1080 });
        assert!(adapter.set_enabled(true).unwrap());
        assert_eq!(received.recv_timeout(Duration::from_secs(2)).unwrap().x, 1740);
        let competitor = SyncConnection::new_session().unwrap();
        let proxy = competitor.with_proxy(BUS, PATH, TIMEOUT);
        let stolen: Result<(bool,), _> = proxy.method_call(BUS, "SetShortcutEnabled", (false,));
        assert!(stolen.is_err(), "another client disabled the active owner");
        assert!(!adapter.set_enabled(false).unwrap());
        let (enabled,): (bool,) = proxy.method_call(BUS, "SetShortcutEnabled", (true,)).unwrap();
        assert!(enabled);
        drop(proxy);
        drop(competitor); // Simulate a crash: no explicit disable call.
        let mut reacquired = false;
        for _ in 0..50 {
            if adapter.set_enabled(true).unwrap_or(false) {
                reacquired = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        assert!(reacquired, "disconnected owner did not release the shortcut");
        drop(adapter);
        let (entered, waiting) = mpsc::channel();
        let (resume, blocked) = mpsc::channel();
        let blocked = std::sync::Mutex::new(blocked);
        let blocker = Adapter::connect(move |_| {
            let _ = entered.send(());
            let _ = blocked.lock().unwrap().recv();
        }).unwrap();
        let mut enabled = false;
        for _ in 0..50 {
            if blocker.set_enabled(true).unwrap_or(false) {
                enabled = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        assert!(enabled);
        waiting.recv_timeout(Duration::from_secs(2)).unwrap();
        let (done, ended) = mpsc::channel();
        let dropper = std::thread::spawn(move || {
            drop(blocker);
            let _ = done.send(());
        });
        let nonblocking = ended.recv_timeout(Duration::from_millis(300));
        let _ = resume.send(());
        dropper.join().unwrap();
        assert!(nonblocking.is_ok(), "shutdown waited for a blocked GUI callback");
    }
}
