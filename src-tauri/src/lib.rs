// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod lcu;
use tauri::{Manager, Emitter};
use tauri::tray::TrayIconBuilder;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem, CheckMenuItem}
};

// 应用状态管理器
#[derive(Clone)]
pub struct AppState {
    pub mouse_through: Arc<Mutex<bool>>,
    pub auto_accept: Arc<Mutex<bool>>,
    pub lcu_auth: Arc<Mutex<Option<lcu::LcuAuthInfo>>>,
    pub gameflow_phase: Arc<Mutex<String>>,
    pub is_running: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mouse_through: Arc::new(Mutex::new(true)),
            auto_accept: Arc::new(Mutex::new(true)),
            lcu_auth: Arc::new(Mutex::new(None)),
            gameflow_phase: Arc::new(Mutex::new("None".to_string())),
            is_running: Arc::new(Mutex::new(true)),
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 获取当前应用状态
#[tauri::command]
fn get_app_state(state: tauri::State<AppState>) -> serde_json::Value {
    let mouse_through = *state.mouse_through.lock().unwrap();
    let auto_accept = *state.auto_accept.lock().unwrap();
    let gameflow_phase = state.gameflow_phase.lock().unwrap().clone();
    let lcu_auth = state.lcu_auth.lock().unwrap().clone();
    
    serde_json::json!({
        "mouse_through": mouse_through,
        "auto_accept": auto_accept,
        "gameflow_phase": gameflow_phase,
        "lcu_connected": lcu_auth.is_some()
    })
}

// 设置自动接受状态
#[tauri::command]
fn set_auto_accept(state: tauri::State<AppState>, enabled: bool) -> Result<String, String> {
    *state.auto_accept.lock().unwrap() = enabled;
    Ok(format!("自动接受已{}", if enabled { "开启" } else { "关闭" }))
}

// 后台状态管理任务
async fn background_task(app_handle: tauri::AppHandle, state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    
    loop {
        interval.tick().await;
        
        // 检查是否应该停止
        if !*state.is_running.lock().unwrap() {
            break;
        }
        
        // 尝试获取LCU认证信息
        match lcu::get_lcu_auth().await {
            Ok(auth) => {
                *state.lcu_auth.lock().unwrap() = Some(auth.clone());
                
                // 获取游戏流程状态
                match lcu::get_gameflow_phase(auth.port.clone(), auth.token.clone()).await {
                    Ok(session) => {
                        let old_phase = state.gameflow_phase.lock().unwrap().clone();
                        *state.gameflow_phase.lock().unwrap() = session.phase.clone();
                        
                        // 如果状态发生变化，通知前端
                        if old_phase != session.phase {
                            let _ = app_handle.emit("gameflow-changed", &session.phase);
                        }
                        
                        // 自动接受匹配
                        if session.phase == "ReadyCheck" && *state.auto_accept.lock().unwrap() {
                            match lcu::accept_match(auth.port, auth.token).await {
                                Ok(_) => {
                                    println!("自动接受匹配成功");
                                    let _ = app_handle.emit("match-accepted", "匹配已自动接受");
                                }
                                Err(e) => {
                                    println!("自动接受匹配失败: {}", e);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        *state.gameflow_phase.lock().unwrap() = "None".to_string();
                    }
                }
            }
            Err(_) => {
                *state.lcu_auth.lock().unwrap() = None;
                *state.gameflow_phase.lock().unwrap() = "None".to_string();
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();
    
    tauri::Builder::default()
        .manage(app_state.clone())
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_decorations(false).unwrap();
            window.set_shadow(false).unwrap();
            window.set_skip_taskbar(true).unwrap();
            window.set_ignore_cursor_events(true).unwrap();
            
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?; 
            let mouse_through_item = CheckMenuItem::with_id(app, "mouse_through", "鼠标穿透", true, true, None::<&str>)?;
            let auto_accept_item = CheckMenuItem::with_id(app, "auto_accept", "自动接受", true, true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&mouse_through_item, &auto_accept_item, &quit_item])?;

            let window_clone = window.clone();
            let _state_clone = app_state.clone();
            let window_for_tray = window.clone();
            let state_for_tray = app_state.clone();
            
            let _tray = TrayIconBuilder::with_id("main")
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_tray_icon_event(move |_tray, event| {
                match event {
                    tauri::tray::TrayIconEvent::Click { 
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        .. 
                    } => {
                        // 左键点击切换窗口显示/隐藏
                        if let Ok(is_visible) = window_for_tray.is_visible() {
                            if is_visible {
                                let _ = window_for_tray.hide();
                            } else {
                                let _ = window_for_tray.show();
                            }
                        }
                    }
                    _ => {}
                }
            })
            .on_menu_event(move |app, event| match event.id.as_ref() {
                "quit" => {
                    println!("quit menu item was clicked");
                    *state_for_tray.is_running.lock().unwrap() = false;
                    std::process::exit(0);
                }
                "mouse_through" => {
                    println!("mouse through menu item was clicked");
                    
                    // 获取当前状态并切换
                    let mut current_state = state_for_tray.mouse_through.lock().unwrap();
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
                        let auto_accept_state = *state_for_tray.auto_accept.lock().unwrap();
                        if let Ok(quit_item_new) = MenuItem::with_id(app, "quit", "退出", true, None::<&str>) {
                            if let Ok(mouse_through_item_new) = CheckMenuItem::with_id(app, "mouse_through", "鼠标穿透", true, new_state, None::<&str>) {
                                if let Ok(auto_accept_item_new) = CheckMenuItem::with_id(app, "auto_accept", "自动接受", true, auto_accept_state, None::<&str>) {
                                    if let Ok(new_menu) = Menu::with_items(app, &[&mouse_through_item_new, &auto_accept_item_new, &quit_item_new]) {
                                        if let Err(e) = tray.set_menu(Some(new_menu)) {
                                            println!("Failed to update tray menu: {:?}", e);
                                        } else {
                                            println!("Successfully updated tray menu");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    println!("Mouse through set to: {}", new_state);
                }
                "auto_accept" => {
                    println!("auto accept menu item was clicked");
                    
                    // 获取当前状态并切换
                    let mut current_state = state_for_tray.auto_accept.lock().unwrap();
                    let new_state = !*current_state;
                    *current_state = new_state;
                    
                    println!("Auto accept set to: {}", new_state);
                    
                    // 重新构建菜单以确保状态更新
                    if let Some(tray) = app.tray_by_id("main") {
                        let mouse_through_state = *state_for_tray.mouse_through.lock().unwrap();
                        if let Ok(quit_item_new) = MenuItem::with_id(app, "quit", "退出", true, None::<&str>) {
                            if let Ok(mouse_through_item_new) = CheckMenuItem::with_id(app, "mouse_through", "鼠标穿透", true, mouse_through_state, None::<&str>) {
                                if let Ok(auto_accept_item_new) = CheckMenuItem::with_id(app, "auto_accept", "自动接受", true, new_state, None::<&str>) {
                                    if let Ok(new_menu) = Menu::with_items(app, &[&mouse_through_item_new, &auto_accept_item_new, &quit_item_new]) {
                                        if let Err(e) = tray.set_menu(Some(new_menu)) {
                                            println!("Failed to update tray menu: {:?}", e);
                                        } else {
                                            println!("Successfully updated tray menu");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    println!("menu item {:?} not handled", event.id);
                }
            })
            .build(app)?;
            
            // 启动后台任务
            let app_handle = app.handle().clone();
            let state_for_task = app_state.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    background_task(app_handle, state_for_task).await;
                });
            });
            
            window.show().unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_state,
            set_auto_accept,
            lcu::check_admin_privileges,
            lcu::get_lcu_auth,
            lcu::get_summoner_info,
            lcu::get_gameflow_phase,
            lcu::accept_match
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
