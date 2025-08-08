use crate::{background, commands, state::AppState, tray};
use tauri::{Manager, PhysicalPosition};

/// 应用程序入口点
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state.clone())
        .setup(move |app| {
            setup_window(app, &app_state)?;
            setup_tray(app, &app_state)?;
            start_background_task(app, &app_state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::set_auto_accept,
            commands::save_window_position,
            commands::save_window_visible,
            crate::lcu::check_admin_privileges,
            crate::lcu::get_lcu_auth,
            crate::lcu::get_summoner_info,
            crate::lcu::get_gameflow_phase,
            crate::lcu::accept_match
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 设置窗口
fn setup_window(
    app: &tauri::App,
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").unwrap();
    window.set_decorations(false).unwrap();
    window.set_shadow(false).unwrap();
    window.set_skip_taskbar(true).unwrap();

    // 从配置加载初始状态
    let config = app_state.config.lock().unwrap();
    let mouse_through_state = config.mouse_through;
    let window_visible = config.window_visible;

    // 设置窗口位置
    let position = PhysicalPosition::new(config.window_position.x, config.window_position.y);
    if let Err(e) = window.set_position(position) {
        println!("设置窗口位置失败: {}", e);
    } else {
        println!(
            "窗口位置设置为: ({}, {})",
            config.window_position.x, config.window_position.y
        );
    }

    // 设置鼠标穿透状态
    window.set_ignore_cursor_events(mouse_through_state).unwrap();
    drop(config); // 释放锁

    // 根据配置决定是否显示窗口
    if window_visible {
        window.show().unwrap();
    } else {
        // 如果配置为隐藏，则不显示窗口
        println!("根据配置，窗口保持隐藏状态");
    }

    Ok(())
}

/// 设置系统托盘
fn setup_tray(app: &tauri::App, app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let config = app_state.config.lock().unwrap();
    let mouse_through_state = config.mouse_through;
    let auto_accept_state = config.auto_accept;
    drop(config);

    tray::create_tray(app, app_state, mouse_through_state, auto_accept_state)?;
    Ok(())
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