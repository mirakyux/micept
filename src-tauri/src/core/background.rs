use crate::core::AppState;
use crate::lol;
use tauri::{Manager, Emitter};
use std::time::Duration;

/// 后台状态管理任务
pub async fn background_task(app_handle: tauri::AppHandle, state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    
    loop {
        interval.tick().await;
        
        // 检查是否应该停止
        if !*state.is_running.lock().unwrap() {
            break;
        }
        
        // 尝试获取LCU认证信息
        match lol::get_lcu_auth().await {
            Ok(auth) => {
                *state.lcu_auth.lock().unwrap() = Some(auth.clone());
                
                // 获取召唤师信息
                match lol::get_summoner_info(auth.port.clone(), auth.token.clone()).await {
                    Ok(summoner) => {
                        *state.summoner_info.lock().unwrap() = Some(summoner);
                    }
                    Err(_) => {
                        *state.summoner_info.lock().unwrap() = None;
                    }
                }
                
                // 获取游戏流程状态
                match lol::get_gameflow_phase(auth.port.clone(), auth.token.clone()).await {
                    Ok(session) => {
                        let old_phase = state.gameflow_phase.lock().unwrap().clone();
                        *state.gameflow_phase.lock().unwrap() = session.phase.clone();
                        
                        // 如果状态发生变化，通知前端
                        if old_phase != session.phase {
                            let _ = app_handle.emit("gameflow-changed", &session.phase);
                            
                            // 当状态变为游戏中时，自动隐藏窗口
                            if session.phase == "InGame" {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if let Err(e) = window.hide() {
                                        println!("隐藏窗口失败: {}", e);
                                    } else {
                                        println!("游戏开始，自动隐藏窗口");
                                    }
                                }
                            }
                            // 当状态从游戏中退出时，可以选择显示窗口（可选功能）
                            else if old_phase == "InGame" && session.phase != "InGame" {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if let Err(e) = window.show() {
                                        println!("显示窗口失败: {}", e);
                                    } else {
                                        println!("游戏结束，自动显示窗口");
                                    }
                                }
                            }
                        }
                        
                        // 自动接受匹配
                        if session.phase == "ReadyCheck" && *state.auto_accept.lock().unwrap() {
                            match lol::accept_match(auth.port, auth.token).await {
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
                *state.summoner_info.lock().unwrap() = None;
                *state.gameflow_phase.lock().unwrap() = "None".to_string();
            }
        }
    }
}