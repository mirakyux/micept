use serde::Serialize;
use sysinfo::System;
use base64::{Engine as _, engine::general_purpose};

#[cfg(not(target_os = "windows"))]
use std::process::Command;

#[derive(Serialize)]
pub struct LcuAuthInfo {
    pub port: String,
    pub token: String,
    pub is_connected: bool,
}

#[derive(Serialize)]
pub struct SummonerInfo {
    pub display_name: String,
    pub summoner_level: u32,
    pub profile_icon_id: u32,
}

#[derive(Serialize)]
pub struct GameflowSession {
    pub phase: String,
}

#[derive(Serialize)]
pub struct AdminStatus {
    pub is_admin: bool,
    pub message: String,
}

#[tauri::command]
pub async fn check_admin_privileges() -> Result<AdminStatus, String> {
    #[cfg(target_os = "windows")]
    {
        // 使用is_elevated库检查管理员权限
        let is_admin = is_elevated::is_elevated();
        
        if is_admin {
            Ok(AdminStatus {
                is_admin: true,
                message: "应用正在以管理员权限运行".to_string(),
            })
        } else {
            Ok(AdminStatus {
                is_admin: false,
                message: "应用未以管理员权限运行，可能无法检测到英雄联盟进程".to_string(),
            })
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // 非Windows平台的简单检查
        let output = Command::new("id")
            .arg("-u")
            .output()
            .map_err(|e| format!("执行id命令失败: {}", e))?;
            
        let uid_str = String::from_utf8_lossy(&output.stdout);
        let uid: u32 = uid_str.trim().parse().unwrap_or(1000);
        
        let is_admin = uid == 0;
        let message = if is_admin {
            "应用正在以root权限运行".to_string()
        } else {
            "应用未以root权限运行".to_string()
        };
        
        Ok(AdminStatus { is_admin, message })
    }
}

#[tauri::command]
pub async fn get_lcu_auth() -> Result<LcuAuthInfo, String> {
    let mut system = System::new_all();
    system.refresh_processes();
    
    for (_, process) in system.processes() {
        if process.name().contains("LeagueClientUx") {
            let cmd_line = process.cmd();
            
            let mut port = String::new();
            let mut token = String::new();
            
            for arg in cmd_line {
                let arg_str = arg.to_string();
                if arg_str.starts_with("--app-port=") {
                    port = arg_str.strip_prefix("--app-port=").unwrap_or("").to_string();
                } else if arg_str.starts_with("--remoting-auth-token=") {
                    token = arg_str.strip_prefix("--remoting-auth-token=").unwrap_or("").to_string();
                }
            }
            
            if !port.is_empty() && !token.is_empty() {
                return Ok(LcuAuthInfo {
                    port,
                    token,
                    is_connected: true,
                });
            }
        }
    }
    
    Err("未找到英雄联盟客户端进程".to_string())
}

#[tauri::command]
pub async fn get_summoner_info(port: String, token: String) -> Result<SummonerInfo, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    let url = format!("https://127.0.0.1:{}/lol-summoner/v1/current-summoner", port);
    let auth = format!("riot:{}", token);
    let auth_header = format!("Basic {}", general_purpose::STANDARD.encode(auth));
    
    let response = client
        .get(&url)
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if response.status().is_success() {
        let summoner: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析JSON失败: {}", e))?;
        
        Ok(SummonerInfo {
            display_name: summoner["displayName"].as_str().unwrap_or("未知").to_string(),
            summoner_level: summoner["summonerLevel"].as_u64().unwrap_or(0) as u32,
            profile_icon_id: summoner["profileIconId"].as_u64().unwrap_or(0) as u32,
        })
    } else {
        Err(format!("获取召唤师信息失败: {}", response.status()))
    }
}

#[tauri::command]
pub async fn get_gameflow_phase(port: String, token: String) -> Result<GameflowSession, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    let url = format!("https://127.0.0.1:{}/lol-gameflow/v1/session", port);
    let auth = format!("riot:{}", token);
    let auth_header = format!("Basic {}", general_purpose::STANDARD.encode(auth));
    
    let response = client
        .get(&url)
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if response.status().is_success() {
        let session: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析JSON失败: {}", e))?;
        
        Ok(GameflowSession {
            phase: session["phase"].as_str().unwrap_or("None").to_string(),
        })
    } else {
        Err(format!("获取游戏流程状态失败: {}", response.status()))
    }
}

#[tauri::command]
pub async fn accept_match(port: String, token: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    let url = format!("https://127.0.0.1:{}/lol-matchmaking/v1/ready-check/accept", port);
    let auth = format!("riot:{}", token);
    let auth_header = format!("Basic {}", general_purpose::STANDARD.encode(auth));
    
    let response = client
        .post(&url)
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if response.status().is_success() {
        Ok("匹配已接受".to_string())
    } else {
        Err(format!("接受匹配失败: {}", response.status()))
    }
}