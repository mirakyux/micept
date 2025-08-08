use crate::core::AppState;
use tauri::{Manager, PhysicalPosition};

/// 设置窗口
pub fn setup_window(
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