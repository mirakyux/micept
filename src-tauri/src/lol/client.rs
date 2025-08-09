use serde::Serialize;
use base64::{Engine as _, engine::general_purpose};

#[cfg(not(target_os = "windows"))]
use std::process::Command;

#[cfg(target_os = "windows")]
use std::process::Command as WinCommand;

#[derive(Serialize, Clone)]
pub struct LcuAuthInfo {
    pub port: String,
    pub token: String,
    pub is_connected: bool,
}

#[derive(Serialize, Clone)]
pub struct SummonerInfo {
    pub display_name: String,
    pub summoner_level: u32,
    pub profile_icon_id: u32,
    pub xp_since_last_level: u32,
    pub xp_until_next_level: u32,
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

/// 验证LCU连接是否有效
pub async fn validate_lcu_connection(port: String, token: String) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
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
        .map_err(|e| format!("验证LCU连接失败: {}", e))?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("LCU连接验证失败，状态码: {}", response.status()))
    }
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
    #[cfg(target_os = "windows")]
    {
        // Windows平台使用tasklist命令获取进程信息，然后使用wmic获取命令行
        let check_output = WinCommand::new("tasklist")
            .args(&["/FI", "IMAGENAME eq LeagueClientUx.exe"])
            .output()
            .map_err(|e| format!("执行tasklist命令失败: {}", e))?;
        
        let check_str = String::from_utf8_lossy(&check_output.stdout);
        println!("tasklist输出: {}", check_str);
        
        if !check_str.contains("LeagueClientUx.exe") {
            return Err("未找到英雄联盟客户端进程".to_string());
        }
        
        // 使用PowerShell的Get-Process命令获取命令行参数
        let output = WinCommand::new("powershell")
            .args(&["-Command", "(Get-CimInstance Win32_Process | Where-Object Name -eq 'LeagueClientUx.exe').CommandLine"])
            .output()
            .map_err(|e| format!("执行PowerShell命令失败: {}", e))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        println!("PowerShell输出: {}", output_str);
        
        // 将所有输出合并为一行进行处理，因为命令行可能跨多行
        let combined_output = output_str.replace('\n', " ").replace('\r', " ");
        
        if combined_output.contains("LeagueClientUx.exe") && combined_output.contains("--app-port=") {
            let mut port = String::new();
            let mut token = String::new();
            
            println!("开始解析命令行参数...");
            println!("合并后的输出长度: {}", combined_output.len());
            
            // 使用正则表达式来精确匹配参数
            if let Some(port_match) = combined_output.find("--app-port=") {
                let port_start = port_match + "--app-port=".len();
                let port_end = combined_output[port_start..].find(' ').unwrap_or(combined_output.len() - port_start);
                port = combined_output[port_start..port_start + port_end].trim_matches('"').to_string();
                println!("找到端口: '{}'", port);
            }
            
            if let Some(token_match) = combined_output.find("--remoting-auth-token=") {
                let token_start = token_match + "--remoting-auth-token=".len();
                let token_end = combined_output[token_start..].find(' ').unwrap_or(combined_output.len() - token_start);
                token = combined_output[token_start..token_start + token_end].trim_matches('"').to_string();
                println!("找到token: '{}...'", &token[..8.min(token.len())]);
            }
            
            if !port.is_empty() && !token.is_empty() {
                println!("成功解析LCU认证信息: port={}, token={}...", port, &token[..8.min(token.len())]);
                return Ok(LcuAuthInfo {
                    port,
                    token,
                    is_connected: true,
                });
            } else {
                println!("解析失败: port={}, token={}", port, token);
            }
        }
        
        Err("未找到英雄联盟客户端进程".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // 非Windows平台使用ps命令
        let output = Command::new("ps")
            .args(&["aux"])
            .output()
            .map_err(|e| format!("执行ps命令失败: {}", e))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.contains("LeagueClientUx") && line.contains("--app-port=") {
                let mut port = String::new();
                let mut token = String::new();
                
                // 解析命令行参数
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.starts_with("--app-port=") {
                        port = part.strip_prefix("--app-port=").unwrap_or("").to_string();
                    } else if part.starts_with("--remoting-auth-token=") {
                        token = part.strip_prefix("--remoting-auth-token=").unwrap_or("").to_string();
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
        
        // 优先使用 gameName，如果为空则使用 displayName
        let display_name = if let Some(game_name) = summoner["gameName"].as_str() {
            if !game_name.is_empty() {
                // 如果有 tagLine，则组合显示
                if let Some(tag_line) = summoner["tagLine"].as_str() {
                    if !tag_line.is_empty() {
                        format!("{}#{}", game_name, tag_line)
                    } else {
                        game_name.to_string()
                    }
                } else {
                    game_name.to_string()
                }
            } else {
                summoner["displayName"].as_str().unwrap_or("未知").to_string()
            }
        } else {
            summoner["displayName"].as_str().unwrap_or("未知").to_string()
        };

        Ok(SummonerInfo {
            display_name,
            summoner_level: summoner["summonerLevel"].as_u64().unwrap_or(0) as u32,
            profile_icon_id: summoner["profileIconId"].as_u64().unwrap_or(0) as u32,
            xp_since_last_level: summoner["xpSinceLastLevel"].as_u64().unwrap_or(0) as u32,
            xp_until_next_level: summoner["xpUntilNextLevel"].as_u64().unwrap_or(0) as u32,
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
        
        let phase = session["phase"].as_str().unwrap_or("None").to_string();

        Ok(GameflowSession {
            phase,
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