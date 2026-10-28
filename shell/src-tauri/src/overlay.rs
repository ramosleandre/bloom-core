use std::sync::Mutex;

use serde::Serialize;
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg(target_os = "macos")]
const SHORTCUT: &str = "Command+Shift+Space";
#[cfg(not(target_os = "macos"))]
const SHORTCUT: &str = "Control+Space";

#[derive(Default)]
struct PendingOverlay {
    anchor: Option<PhysicalPosition<f64>>,
}

#[derive(Default)]
pub struct OverlayState {
    pending: Mutex<PendingOverlay>,
    shortcut_error: Mutex<Option<String>>,
    #[cfg(target_os = "linux")]
    gnome: Mutex<Option<crate::gnome::Adapter>>,
}

fn native_wayland() -> bool {
    cfg!(target_os = "linux")
        && std::env::var("GDK_BACKEND").as_deref() != Ok("x11")
        && (std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland")
            || std::env::var_os("WAYLAND_DISPLAY").is_some())
}

pub fn install_shortcut(app: &tauri::AppHandle) {
    #[cfg(target_os = "linux")]
    if gnome_wayland() && !native_wayland() {
        let handle = app.clone();
        match crate::gnome::Adapter::connect(move |pointer| {
            trigger(&handle, gnome_root_position(pointer));
        }) {
            Ok(adapter) => {
                *app.state::<OverlayState>().gnome.lock().unwrap() = Some(adapter);
                return;
            }
            Err(error) => eprintln!("Bloom GNOME adapter unavailable: {error}"),
        }
    }
    let error = if native_wayland() {
        Some("Your Wayland session needs desktop shortcut integration. Quick launch works while Bloom is focused. Try GDK_BACKEND=x11 make ui for XWayland; global shortcuts still depend on your compositor.".to_owned())
    } else {
        app.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    trigger(app, desktop_cursor(app));
                })
                .build(),
        )
        .err()
        .map(|error| format!("Global shortcut unavailable: {error}. Quick launch works while Bloom is focused."))
    };
    *app.state::<OverlayState>().shortcut_error.lock().unwrap() = error;
}

fn trigger(app: &tauri::AppHandle, anchor: Option<PhysicalPosition<f64>>) {
    if let Some(pie) = app.get_webview_window("pie") {
        if pie.is_visible().unwrap_or(false) {
            let _ = pie.emit("bloom-cancel", ());
            return;
        }
    }
    *app.state::<OverlayState>().pending.lock().unwrap() = PendingOverlay { anchor };
    let _ = app.emit_to("main", "bloom-trigger", ());
}

pub fn shutdown(_app: &tauri::AppHandle) {
    #[cfg(target_os = "linux")]
    {
        let adapter = _app.state::<OverlayState>().gnome.lock().unwrap().take();
        drop(adapter);
    }
}

#[cfg(target_os = "linux")]
fn gnome_wayland() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().split(':')
        .any(|desktop| desktop.eq_ignore_ascii_case("gnome"))
        && (std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland")
            || std::env::var_os("WAYLAND_DISPLAY").is_some())
}

#[cfg(target_os = "linux")]
fn gnome_root_position(pointer: crate::gnome::Pointer) -> Option<PhysicalPosition<f64>> {
    use x11rb::connection::Connection;
    let (connection, screen) = x11rb::connect(None).ok()?;
    let root = connection.setup().roots.get(screen)?;
    pointer.to_root(u32::from(root.width_in_pixels), u32::from(root.height_in_pixels))
}

#[derive(Serialize)]
pub struct ShortcutStatus {
    registered: bool,
    message: String,
}

#[tauri::command]
pub fn configure_shortcut(app: tauri::AppHandle, enabled: bool) -> ShortcutStatus {
    #[cfg(target_os = "linux")]
    if let Some(adapter) = app.state::<OverlayState>().gnome.lock().unwrap().as_ref() {
        return match adapter.set_enabled(enabled) {
            Ok(registered) => ShortcutStatus {
                registered,
                message: if registered {
                    "Ctrl+Space opens Bloom across your desktop using the GNOME adapter.".to_owned()
                } else {
                    "Quick launch is off.".to_owned()
                },
            },
            Err(error) => ShortcutStatus {
                registered: false,
                message: format!("GNOME shortcut unavailable: {error}. Restart Bloom after enabling the adapter."),
            },
        };
    }
    if let Some(error) = app
        .state::<OverlayState>()
        .shortcut_error
        .lock()
        .unwrap()
        .as_ref()
    {
        return ShortcutStatus {
            registered: false,
            message: error.clone(),
        };
    }
    let shortcut = app.global_shortcut();
    let result = if enabled {
        if shortcut.is_registered(SHORTCUT) {
            Ok(())
        } else {
            shortcut.register(SHORTCUT)
        }
    } else {
        shortcut.unregister(SHORTCUT)
    };
    match result {
        Ok(()) => ShortcutStatus {
            registered: enabled,
            message: if enabled {
                #[cfg(target_os = "linux")]
                if gnome_wayland() {
                    return ShortcutStatus {
                        registered: true,
                        message: "Ctrl+Space uses XWayland fallback. Enable Bloom Desktop Adapter for accurate positioning outside Bloom.".to_owned(),
                    };
                }
                format!("{SHORTCUT} opens the pie menu while you use another app.")
            } else {
                "Quick launch is off.".to_owned()
            },
        },
        Err(error) => ShortcutStatus {
            registered: false,
            message: format!("Could not register {SHORTCUT}: {error}. Quick launch works while Bloom is focused."),
        },
    }
}

#[tauri::command]
pub fn prepare_pie(app: tauri::AppHandle) {
    let anchor = desktop_cursor(&app);
    *app.state::<OverlayState>().pending.lock().unwrap() = PendingOverlay { anchor };
}

#[cfg(target_os = "linux")]
fn desktop_cursor(app: &tauri::AppHandle) -> Option<PhysicalPosition<f64>> {
    use x11rb::{connection::Connection, protocol::xproto::ConnectionExt};

    if native_wayland() {
        return None;
    }
    if let Some(adapter) = app.state::<OverlayState>().gnome.lock().unwrap().as_ref() {
        if let Ok(pointer) = adapter.pointer() {
            return gnome_root_position(pointer);
        }
    }
    // Query the desktop root rather than GTK's device/window position cache. The latter
    // can stop updating when Bloom loses pointer focus or its main window is minimized.
    let (connection, screen) = x11rb::connect(None).ok()?;
    let root = connection.setup().roots.get(screen)?.root;
    let pointer = connection.query_pointer(root).ok()?.reply().ok()?;
    pointer.same_screen.then(|| {
        PhysicalPosition::new(f64::from(pointer.root_x), f64::from(pointer.root_y))
    })
}

#[cfg(not(target_os = "linux"))]
fn desktop_cursor(app: &tauri::AppHandle) -> Option<PhysicalPosition<f64>> {
    app.cursor_position().ok()
}

// All geometry uses physical desktop coordinates, including monitors left of the primary.
fn origin(cursor: f64, start: i32, available: u32, extent: u32) -> i32 {
    let end = (start as i64 + available as i64 - extent as i64).max(start as i64);
    (cursor - extent as f64 / 2.0)
        .round()
        .clamp(start as f64, end as f64) as i32
}

pub fn position(app: &tauri::AppHandle, pie: &WebviewWindow) -> Result<bool, String> {
    // Tao returns the placeholder (0, 0) on Wayland rather than a cursor lookup error.
    if native_wayland() {
        pie.center().map_err(|error| error.to_string())?;
        return Ok(false);
    }
    let anchor = app
        .state::<OverlayState>()
        .pending
        .lock()
        .unwrap()
        .anchor
        .take()
        .or_else(|| desktop_cursor(app));
    let Some(cursor) = anchor else {
        pie.center().map_err(|error| error.to_string())?;
        return Ok(false);
    };
    let monitor = pie
        .monitor_from_point(cursor.x, cursor.y)
        .map_err(|error| error.to_string())?
        .ok_or("No monitor found at the cursor")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    // Scale using the destination monitor before positioning, so mixed-DPI displays agree.
    let side = (480.0 * scale).round() as u32;
    let side = side.min(area.size.width).min(area.size.height);
    pie.set_size(PhysicalSize::new(side, side))
        .map_err(|error| error.to_string())?;
    pie.set_position(PhysicalPosition::new(
        origin(cursor.x, area.position.x, area.size.width, side),
        origin(cursor.y, area.position.y, area.size.height, side),
    ))
    .map_err(|error| error.to_string())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::origin;

    #[test]
    fn centers_and_keeps_the_whole_menu_on_the_monitor() {
        assert_eq!(origin(800.0, 0, 1920, 480), 560);
        assert_eq!(origin(3.0, 0, 1920, 480), 0);
        assert_eq!(origin(1910.0, 0, 1920, 480), 1440);
        assert_eq!(origin(-800.0, -1920, 1920, 480), -1040);
        assert_eq!(origin(-1910.0, -1920, 1920, 480), -1920);
        assert_eq!(origin(1200.0, 0, 2880, 960), 720);
    }
}
