// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod lcu;
use tauri::Manager;

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
