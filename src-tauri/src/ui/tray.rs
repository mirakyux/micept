use crate::core::AppState;
use tauri::{
    menu::{Menu, MenuItem, CheckMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

/// 创建系统托盘
pub fn create_tray(
    app: &tauri::App,
    app_state: &AppState,
    mouse_through_state: bool,
    auto_accept_state: bool,
    auto_hide_state: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let mouse_through_item = CheckMenuItem::with_id(
        app,
        "mouse_through",
        "鼠标穿透",
        true,
        mouse_through_state,
        None::<&str>,
    )?;
    let auto_accept_item = CheckMenuItem::with_id(
        app,
        "auto_accept",
        "自动接受",
        true,
        auto_accept_state,
        None::<&str>,
    )?;
    let auto_hide_item = CheckMenuItem::with_id(
        app,
        "auto_hide",
        "自动隐藏",
        true,
        auto_hide_state,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&mouse_through_item, &auto_accept_item, &auto_hide_item, &quit_item])?;

    let window = app.get_webview_window("main").unwrap();
    let window_clone = window.clone();
    let window_for_tray = window.clone();
    let state_for_tray = app_state.clone();
    let state_for_menu = app_state.clone();

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
                            // 保存窗口可见性状态
                            state_for_tray
                                .config
                                .lock()
                                .unwrap()
                                .update_window_visible(false);
                        } else {
                            let _ = window_for_tray.show();
                            // 保存窗口可见性状态
                            state_for_tray
                                .config
                                .lock()
                                .unwrap()
                                .update_window_visible(true);
                        }
                    }
                }
                _ => {}
            }
        })
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                println!("quit menu item was clicked");
                *state_for_menu.is_running.lock().unwrap() = false;
                std::process::exit(0);
            }
            "mouse_through" => {
                handle_mouse_through_event(app, &state_for_menu, &window_clone);
            }
            "auto_accept" => {
                handle_auto_accept_event(app, &state_for_menu);
            }
            "auto_hide" => {
                handle_auto_hide_event(app, &state_for_menu);
            }
            _ => {
                println!("menu item {:?} not handled", event.id);
            }
        })
        .build(app)?;

    Ok(())
}

/// 处理鼠标穿透菜单事件
fn handle_mouse_through_event(
    app: &tauri::AppHandle,
    state: &AppState,
    window: &tauri::WebviewWindow,
) {
    println!("mouse through menu item was clicked");

    // 获取当前状态并切换
    let mut current_state = state.mouse_through.lock().unwrap();
    let new_state = !*current_state;
    *current_state = new_state;

    // 更新配置文件
    state.config.lock().unwrap().update_mouse_through(new_state);

    println!("Current state: {}, New state: {}", !new_state, new_state);

    // 设置窗口鼠标穿透状态
    if let Err(e) = window.set_ignore_cursor_events(new_state) {
        println!("Failed to set ignore cursor events: {:?}", e);
    } else {
        println!("Successfully set ignore cursor events to: {}", new_state);
    }

    // 重新构建菜单以确保状态更新
    update_tray_menu(app, state, new_state, *state.auto_accept.lock().unwrap(), *state.auto_hide.lock().unwrap());

    println!("Mouse through set to: {}", new_state);
}

/// 处理自动接受菜单事件
fn handle_auto_accept_event(app: &tauri::AppHandle, state: &AppState) {
    println!("auto accept menu item was clicked");

    // 获取当前状态并切换
    let mut current_state = state.auto_accept.lock().unwrap();
    let new_state = !*current_state;
    *current_state = new_state;

    // 更新配置文件
    state.config.lock().unwrap().update_auto_accept(new_state);

    println!("Auto accept set to: {}", new_state);

    // 重新构建菜单以确保状态更新
    update_tray_menu(app, state, *state.mouse_through.lock().unwrap(), new_state, *state.auto_hide.lock().unwrap());
}

/// 处理自动隐藏菜单事件
fn handle_auto_hide_event(app: &tauri::AppHandle, state: &AppState) {
    println!("auto hide menu item was clicked");

    // 获取当前状态并切换
    let mut current_state = state.auto_hide.lock().unwrap();
    let new_state = !*current_state;
    *current_state = new_state;

    // 更新配置文件
    state.config.lock().unwrap().update_auto_hide(new_state);

    println!("Auto hide set to: {}", new_state);

    // 重新构建菜单以确保状态更新
    update_tray_menu(app, state, *state.mouse_through.lock().unwrap(), *state.auto_accept.lock().unwrap(), new_state);
}

/// 更新托盘菜单
fn update_tray_menu(
    app: &tauri::AppHandle,
    _state: &AppState,
    mouse_through_state: bool,
    auto_accept_state: bool,
    auto_hide_state: bool,
) {
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(quit_item_new) = MenuItem::with_id(app, "quit", "退出", true, None::<&str>) {
            if let Ok(mouse_through_item_new) = CheckMenuItem::with_id(
                app,
                "mouse_through",
                "鼠标穿透",
                true,
                mouse_through_state,
                None::<&str>,
            ) {
                if let Ok(auto_accept_item_new) = CheckMenuItem::with_id(
                    app,
                    "auto_accept",
                    "自动接受",
                    true,
                    auto_accept_state,
                    None::<&str>,
                ) {
                    if let Ok(auto_hide_item_new) = CheckMenuItem::with_id(
                        app,
                        "auto_hide",
                        "自动隐藏",
                        true,
                        auto_hide_state,
                        None::<&str>,
                    ) {
                        if let Ok(new_menu) = Menu::with_items(
                            app,
                            &[&mouse_through_item_new, &auto_accept_item_new, &auto_hide_item_new, &quit_item_new],
                        ) {
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
}