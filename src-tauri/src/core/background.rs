use crate::core::AppState;
use crate::lol;
use tauri::{Manager, Emitter};
use std::time::Duration;

/// 后台状态管理任务
pub async fn background_task(app_handle: tauri::AppHandle, state: AppState) {
    // 使用自适应间隔，根据LCU连接状态调整检查频率
    let base_interval = Duration::from_secs(3);
    let mut current_interval = base_interval;
    let mut consecutive_failures = 0;
    
    loop {
        tokio::time::sleep(current_interval).await;
        
        // 检查是否应该停止
        if !*state.is_running.lock().unwrap() {
            println!("后台任务停止");
            break;
        }
        
        // 首先检查是否已有缓存的LCU认证信息
        let cached_auth = {
            let lcu_guard = state.lcu_auth.lock().unwrap();
            lcu_guard.clone()
        };
        
        let auth = match cached_auth {
            Some(cached) => {
                // 验证缓存的认证信息是否仍然有效
                println!("使用缓存的LCU认证信息进行验证...");
                match lol::validate_lcu_connection(cached.port.clone(), cached.token.clone()).await {
                    Ok(_) => {
                        println!("缓存的LCU认证信息仍然有效");
                        consecutive_failures = 0;
                        current_interval = base_interval;
                        cached
                    }
                    Err(_) => {
                        println!("缓存的LCU认证信息已失效，重新获取...");
                        // 缓存失效，重新获取
                        match lol::get_lcu_auth().await {
                            Ok(new_auth) => {
                                println!("成功获取新的LCU认证信息: port={}, token={}...", new_auth.port, &new_auth.token[..8.min(new_auth.token.len())]);
                                consecutive_failures = 0;
                                current_interval = base_interval;
                                
                                // 更新缓存
                                *state.lcu_auth.lock().unwrap() = Some(new_auth.clone());
                                let _ = app_handle.emit("lcu-status-changed", true);
                                
                                new_auth
                            }
                            Err(e) => {
                                println!("重新获取LCU认证信息失败: {}", e);
                                consecutive_failures += 1;
                                current_interval = base_interval * (1 + consecutive_failures.min(5));
                                
                                // 清理缓存和状态
                                *state.lcu_auth.lock().unwrap() = None;
                                *state.summoner_info.lock().unwrap() = None;
                                *state.gameflow_phase.lock().unwrap() = "None".to_string();
                                let _ = app_handle.emit("lcu-status-changed", false);
                                continue;
                            }
                        }
                    }
                }
            }
            None => {
                // 没有缓存，首次获取
                println!("首次获取LCU认证信息...");
                match lol::get_lcu_auth().await {
                    Ok(new_auth) => {
                        println!("成功获取LCU认证信息: port={}, token={}...", new_auth.port, &new_auth.token[..8.min(new_auth.token.len())]);
                        consecutive_failures = 0;
                        current_interval = base_interval;
                        
                        // 缓存认证信息
                        *state.lcu_auth.lock().unwrap() = Some(new_auth.clone());
                        let _ = app_handle.emit("lcu-status-changed", true);
                        
                        new_auth
                    }
                    Err(e) => {
                        println!("获取LCU认证信息失败: {}", e);
                        consecutive_failures += 1;
                        current_interval = base_interval * (1 + consecutive_failures.min(5));
                        continue;
                    }
                }
            }
        };
        
        // 获取召唤师信息 - 只在必要时更新
        let should_update_summoner = {
            let summoner_guard = state.summoner_info.lock().unwrap();
            summoner_guard.is_none()
        };
        
        if should_update_summoner {
            match lol::get_summoner_info(auth.port.clone(), auth.token.clone()).await {
                Ok(summoner) => {
                    let mut summoner_guard = state.summoner_info.lock().unwrap();
                    let summoner_changed = match &*summoner_guard {
                        Some(existing) => existing.display_name != summoner.display_name || existing.summoner_level != summoner.summoner_level,
                        None => true,
                    };
                    
                    if summoner_changed {
                        *summoner_guard = Some(summoner.clone());
                        drop(summoner_guard);
                        let _ = app_handle.emit("summoner-info-updated", &summoner);
                    }
                }
                Err(_) => {
                    *state.summoner_info.lock().unwrap() = None;
                }
            }
        }
        
        // 获取游戏流程状态
        match lol::get_gameflow_phase(auth.port.clone(), auth.token.clone()).await {
            Ok(session) => {
                let old_phase = {
                    let phase_guard = state.gameflow_phase.lock().unwrap();
                    phase_guard.clone()
                };
                
                // 只有状态真正改变时才更新
                if old_phase != session.phase {
                    println!("游戏状态变化: {} -> {}", old_phase, session.phase);
                    *state.gameflow_phase.lock().unwrap() = session.phase.clone();
                    let _ = app_handle.emit("gameflow-changed", &session.phase);
                    
                    // 根据游戏状态调整检查频率
                    match session.phase.as_str() {
                        "ReadyCheck" => {
                            // 在准备检查阶段提高频率
                            current_interval = Duration::from_millis(500);
                        }
                        "InGame" => {
                            // 游戏中降低频率
                            current_interval = Duration::from_secs(10);
                            
                            // 检查自动隐藏功能是否开启
                            let auto_hide_enabled = *state.auto_hide.lock().unwrap();
                            if auto_hide_enabled {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.hide();
                                    println!("自动隐藏功能已开启，窗口已隐藏");
                                }
                            }
                        }
                        _ => {
                            current_interval = base_interval;
                            // 从游戏中退出时，如果自动隐藏功能开启，则显示窗口
                            if old_phase == "InGame" {
                                let auto_hide_enabled = *state.auto_hide.lock().unwrap();
                                if auto_hide_enabled {
                                    if let Some(window) = app_handle.get_webview_window("main") {
                                        let _ = window.show();
                                        println!("游戏结束，自动显示窗口");
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 自动接受匹配
                if session.phase == "ReadyCheck" {
                    let auto_accept_enabled = *state.auto_accept.lock().unwrap();
                    println!("检测到ReadyCheck状态，自动接受开关: {}", auto_accept_enabled);
                    
                    if auto_accept_enabled {
                        println!("尝试自动接受匹配...");
                        match lol::accept_match(auth.port, auth.token).await {
                            Ok(_) => {
                                println!("匹配已自动接受");
                                let _ = app_handle.emit("match-accepted", "匹配已自动接受");
                            }
                            Err(e) => {
                                eprintln!("自动接受匹配失败: {}", e);
                            }
                        }
                    } else {
                        println!("自动接受功能已关闭，跳过自动接受");
                    }
                }
            }
            Err(_) => {
                *state.gameflow_phase.lock().unwrap() = "None".to_string();
            }
        }
    }
}