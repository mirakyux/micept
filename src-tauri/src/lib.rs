// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod lcu;
use tauri::Manager;
use tauri::tray::TrayIconBuilder;
use std::sync::{Arc, Mutex};

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
            
            // 使用Arc<Mutex<bool>>来跟踪鼠标穿透状态
            let mouse_through_state = Arc::new(Mutex::new(true));
            
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?; 
            let mouse_through_item = CheckMenuItem::with_id(app, "mouse_through", "鼠标穿透", true, true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&mouse_through_item, &quit_item])?;

            let window_clone = window.clone();
            let state_clone = mouse_through_state.clone();
            let _tray = TrayIconBuilder::with_id("main")
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .on_menu_event(move |app, event| match event.id.as_ref() {
                "quit" => {
                    println!("quit menu item was clicked");
                    std::process::exit(0);
                }
                "mouse_through" => {
                    println!("mouse through menu item was clicked");
                    
                    // 获取当前状态并切换
                    let mut current_state = state_clone.lock().unwrap();
                    let new_state = !*current_state;
                    *current_state = new_state;
                    
                    println!("Current state: {}, New state: {}", !new_state, new_state);
                    
                    // 设置窗口鼠标穿透状态
                    if let Err(e) = window_clone.set_ignore_cursor_events(new_state) {
                        println!("Failed to set ignore cursor events: {:?}", e);
                    } else {
                        println!("Successfully set ignore cursor events to: {}", new_state);
                    }
                    
                    // 重新构建菜单以确保状态更新
                    if let Some(tray) = app.tray_by_id("main") {
                        if let Ok(quit_item_new) = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>) {
                            if let Ok(mouse_through_item_new) = CheckMenuItem::with_id(app, "mouse_through", "鼠标穿透", true, new_state, None::<&str>) {
                                if let Ok(new_menu) = Menu::with_items(app, &[&mouse_through_item_new, &quit_item_new]) {
                                    if let Err(e) = tray.set_menu(Some(new_menu)) {
                                        println!("Failed to update tray menu: {:?}", e);
                                    } else {
                                        println!("Successfully updated tray menu");
                                    }
                                }
                            }
                        }
                    }
                    
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
