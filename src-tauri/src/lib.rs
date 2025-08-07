// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod lcu;
use tauri::Manager;
use tauri::tray::TrayIconBuilder;

use tauri::{
    menu::{Menu, MenuItem, CheckMenuItem}
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_decorations(false).unwrap();
            window.set_shadow(false).unwrap();
            window.set_skip_taskbar(true).unwrap();
            window.set_ignore_cursor_events(true).unwrap();
            
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?; 
            let mouse_through_item = CheckMenuItem::with_id(app, "mouse_through", "鼠标穿透", true, true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&mouse_through_item, &quit_item])?;

            let window_clone = window.clone();
            let mouse_through_clone = mouse_through_item.clone();
            let _tray = TrayIconBuilder::with_id("main")
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .on_menu_event(move |_app, event| match event.id.as_ref() {
                "quit" => {
                    println!("quit menu item was clicked");
                    std::process::exit(0);
                }
                "mouse_through" => {
                    println!("mouse through menu item was clicked");
                    let is_checked = mouse_through_clone.is_checked().unwrap_or(false);
                    let new_state = !is_checked;
                    let _ = mouse_through_clone.set_checked(new_state);
                    let _ = window_clone.set_ignore_cursor_events(new_state);
                    println!("Mouse through set to: {}", new_state);
                }
                _ => {
                    println!("menu item {:?} not handled", event.id);
                }
            })
            .build(app)?;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            lcu::check_admin_privileges,
            lcu::get_lcu_auth,
            lcu::get_summoner_info,
            lcu::get_gameflow_phase,
            lcu::accept_match
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
