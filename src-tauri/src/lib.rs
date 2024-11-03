use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder},
    Manager, WebviewUrl, WebviewWindowBuilder,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn greet(name: String) -> String {
    // Suppose it took 5 seconds to perform other operations, like fetching data from an API.
    std::thread::sleep(Duration::from_secs(5));
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let show_window = MenuItem::with_id(app, "show", "Show", true, None::<&str>).unwrap();
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&show_window, &quit_i]).unwrap();
            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| match event {
                    tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let main_window = tray.app_handle().get_webview_window("main");
                        match main_window {
                            Some(window) => {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            None => {
                                let window = WebviewWindowBuilder::new(
                                    tray.app_handle(),
                                    "main",
                                    WebviewUrl::default(),
                                )
                                .visible(false)
                                .transparent(true)
                                .shadow(false)
                                .build()
                                .unwrap();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    _ => {}
                })
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app_handle, event| match event.id.as_ref() {
                    "show" => match app_handle.get_webview_window("main") {
                        Some(window) => {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        None => {
                            let window = WebviewWindowBuilder::new(
                                app_handle,
                                "main",
                                WebviewUrl::default(),
                            )
                            .visible(false)
                            .transparent(true)
                            .shadow(false)
                            .build()
                            .unwrap();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    },
                    "quit" => {
                        app_handle.exit(0);
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .build(app)
                .unwrap();

            let window = WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::default())
                .min_inner_size(800., 600.)
                .visible(false)
                .center()
                .build()
                .unwrap();
            let _ = window.show();
            let _ = window.set_focus();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match &event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
