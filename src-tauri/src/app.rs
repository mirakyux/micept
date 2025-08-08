use crate::{core::{background, AppState}, commands, ui::{tray, window}, lol};

/// 应用程序入口点
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state.clone())
        .setup(move |app| {
            window::setup_window(app, &app_state)?;
            
            // 设置系统托盘
            let config = app_state.config.lock().unwrap();
            let mouse_through_state = config.mouse_through;
            let auto_accept_state = config.auto_accept;
            drop(config);
            tray::create_tray(app, &app_state, mouse_through_state, auto_accept_state)?;
            
            start_background_task(app, &app_state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::set_auto_accept,
            commands::save_window_position,
            commands::save_window_visible,
            lol::check_admin_privileges,
            lol::get_lcu_auth,
            lol::get_summoner_info,
            lol::get_gameflow_phase,
            lol::accept_match
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 启动后台任务
fn start_background_task(app: &tauri::App, app_state: &AppState) {
    let app_handle = app.handle().clone();
    let state_for_task = app_state.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            background::background_task(app_handle, state_for_task).await;
        });
    });
}