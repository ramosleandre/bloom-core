#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod overlay;
#[cfg(target_os = "linux")]
mod gnome;

use overlay::{configure_shortcut, prepare_pie, OverlayState};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
fn show_pie(app: tauri::AppHandle) -> Result<bool, String> {
    let pie = app
        .get_webview_window("pie")
        .ok_or("Pie overlay unavailable")?;
    let anchored = overlay::position(&app, &pie)?;
    pie.show().map_err(|error| error.to_string())?;
    pie.set_focus().map_err(|error| error.to_string())?;
    Ok(anchored)
}

#[tauri::command]
fn hide_pie(app: tauri::AppHandle, approval: bool) -> Result<(), String> {
    if let Some(pie) = app.get_webview_window("pie") {
        pie.hide().map_err(|error| error.to_string())?;
    }
    if approval {
        show_main(app)?;
    }
    Ok(())
}

#[tauri::command]
fn show_main(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main) = app.get_webview_window("main") {
        main.show().map_err(|error| error.to_string())?;
        main.unminimize().map_err(|error| error.to_string())?;
        main.set_focus().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn main() {
    // Use GTK's native titlebar controls for this process, including on XWayland.
    #[cfg(target_os = "linux")]
    std::env::set_var("GTK_CSD", "1");

    tauri::Builder::default()
        .manage(OverlayState::default())
        .invoke_handler(tauri::generate_handler![
            show_pie,
            hide_pie,
            show_main,
            prepare_pie,
            configure_shortcut
        ])
        .setup(|app| {
            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::GtkSettingsExt;
                if let Some(settings) = gtk::Settings::default() {
                    settings.set_gtk_decoration_layout(Some(":minimize,maximize,close"));
                }
            }
            let launch = std::env::var("BLOOM_UI_URL")
                .map_err(|_| "Launch the desktop shell with make desktop from the repository root")?;
            let main_url: tauri::Url = launch.parse()?;
            if main_url.host_str() != Some("127.0.0.1") || main_url.scheme() != "http" {
                return Err("Bloom requires its local Python server".into());
            }
            let mut pie_url = main_url.clone();
            pie_url.query_pairs_mut().append_pair("view", "pie");
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(main_url))
                .title("Bloom")
                .inner_size(1320.0, 900.0)
                .min_inner_size(760.0, 620.0)
                .build()?;
            let pie = WebviewWindowBuilder::new(app, "pie", WebviewUrl::External(pie_url))
                .title("Bloom · Your next action")
                .inner_size(480.0, 480.0)
                .decorations(false)
                .transparent(true)
                .background_color(tauri::webview::Color(0, 0, 0, 0))
                .shadow(false)
                .skip_taskbar(true)
                .resizable(false)
                .always_on_top(true)
                .visible_on_all_workspaces(true)
                .visible(false)
                .build()?;
            let pie_handle = pie.clone();
            pie.on_window_event(move |event| {
                let cancel = match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        true
                    }
                    tauri::WindowEvent::Focused(false) => pie_handle.is_visible().unwrap_or(false),
                    _ => false,
                };
                if cancel {
                    let _ = pie_handle.hide();
                    let _ = pie_handle.emit("bloom-cancel", ());
                }
            });
            overlay::install_shortcut(app.handle());
            let handle = app.handle().clone();
            let main = app
                .get_webview_window("main")
                .ok_or("Main window unavailable")?;
            main.on_window_event(move |event| {
                if let tauri::WindowEvent::Destroyed = event {
                    overlay::shutdown(&handle);
                    handle.exit(0);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Bloom desktop shell failed to start");
}
