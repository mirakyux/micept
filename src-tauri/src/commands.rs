use crate::core::AppState;
use tauri::State;

/// 获取当前应用状态
#[tauri::command]
pub fn get_app_state(state: State<AppState>) -> serde_json::Value {
    let mouse_through = *state.mouse_through.lock().unwrap();
    let auto_accept = *state.auto_accept.lock().unwrap();
    let auto_hide = *state.auto_hide.lock().unwrap();
    let gameflow_phase = state.gameflow_phase.lock().unwrap().clone();
    let lcu_auth = state.lcu_auth.lock().unwrap().clone();
    let summoner_info = state.summoner_info.lock().unwrap().clone();
    
    serde_json::json!({
        "mouse_through": mouse_through,
        "auto_accept": auto_accept,
        "auto_hide": auto_hide,
        "gameflow_phase": gameflow_phase,
        "lcu_connected": lcu_auth.is_some(),
        "summoner_info": summoner_info
    })
}

/// 设置自动接受状态
#[tauri::command]
pub fn set_auto_accept(state: State<AppState>, enabled: bool) -> Result<String, String> {
    *state.auto_accept.lock().unwrap() = enabled;
    // 更新配置文件
    state.config.lock().unwrap().update_auto_accept(enabled);
    Ok(format!("自动接受已{}", if enabled { "开启" } else { "关闭" }))
}

/// 保存窗口位置
#[tauri::command]
pub fn save_window_position(state: State<AppState>, x: i32, y: i32) -> Result<String, String> {
    state.config.lock().unwrap().update_window_position(x, y);
    Ok("窗口位置已保存".to_string())
}

/// 保存窗口可见性
#[tauri::command]
pub fn save_window_visible(state: State<AppState>, visible: bool) -> Result<String, String> {
    state.config.lock().unwrap().update_window_visible(visible);
    Ok(format!("窗口可见性已保存: {}", visible))
}