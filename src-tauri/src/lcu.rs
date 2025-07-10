use sysinfo::System;
use serde::{Serialize, Deserialize};
use reqwest;

#[derive(Serialize)]
pub struct LcuAuthInfo {
    pub port: Option<u16>,
    pub token: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SummonerInfo {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "summonerLevel")]
    pub summoner_level: u32,
    #[serde(rename = "profileIconId")]
    pub profile_icon_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameflowSession {
    pub phase: String,
}

#[tauri::command]
pub fn get_lcu_auth() -> LcuAuthInfo {
    let mut sys = System::new_all();
    sys.refresh_processes();
    let mut port = None;
    let mut token = None;
    
    for (_, process) in sys.processes() {
        if process.name() == "LeagueClientUx.exe" {
            let cmd = process.cmd().join(" ");
            for arg in cmd.split_whitespace() {
                if arg.starts_with("--app-port=") {
                    port = arg[11..].parse().ok();
                }
                if arg.starts_with("--remoting-auth-token=") {
                    token = Some(arg[23..].to_string());
                }
            }
        }
    }
    
    let base_url = port.map(|p| format!("https://127.0.0.1:{}", p));
    LcuAuthInfo { port, token, base_url }
}

#[tauri::command]
pub async fn get_summoner_info() -> Result<SummonerInfo, String> {
    let auth = get_lcu_auth();
    
    if let (Some(base_url), Some(token)) = (auth.base_url, auth.token) {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create client: {}", e))?;
        
        let response = client
            .get(&format!("{}/lol-summoner/v1/current-summoner", base_url))
            .basic_auth("riot", Some(&token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let summoner: SummonerInfo = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            Ok(summoner)
        } else {
            Err(format!("API request failed with status: {}", response.status()))
        }
    } else {
        Err("LCU not found or authentication failed".to_string())
    }
}

#[tauri::command]
pub async fn get_gameflow_phase() -> Result<String, String> {
    let auth = get_lcu_auth();
    
    if let (Some(base_url), Some(token)) = (auth.base_url, auth.token) {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create client: {}", e))?;
        
        let response = client
            .get(&format!("{}/lol-gameflow/v1/session", base_url))
            .basic_auth("riot", Some(&token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let session: GameflowSession = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            Ok(session.phase)
        } else {
            Err(format!("API request failed with status: {}", response.status()))
        }
    } else {
        Err("LCU not found or authentication failed".to_string())
    }
}

#[tauri::command]
pub async fn accept_match() -> Result<String, String> {
    let auth = get_lcu_auth();
    
    if let (Some(base_url), Some(token)) = (auth.base_url, auth.token) {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create client: {}", e))?;
        
        let response = client
            .post(&format!("{}/lol-matchmaking/v1/ready-check/accept", base_url))
            .basic_auth("riot", Some(&token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            Ok("Match accepted successfully".to_string())
        } else {
            Err(format!("Failed to accept match: {}", response.status()))
        }
    } else {
        Err("LCU not found or authentication failed".to_string())
    }
}