use crate::utils::AppConfig;
use crate::lol::{LcuAuthInfo, SummonerInfo};
use std::sync::{Arc, Mutex};

/// 应用状态管理器
#[derive(Clone)]
pub struct AppState {
    pub mouse_through: Arc<Mutex<bool>>,
    pub auto_accept: Arc<Mutex<bool>>,
    pub lcu_auth: Arc<Mutex<Option<LcuAuthInfo>>>,
    pub gameflow_phase: Arc<Mutex<String>>,
    pub summoner_info: Arc<Mutex<Option<SummonerInfo>>>,
    pub is_running: Arc<Mutex<bool>>,
    pub config: Arc<Mutex<AppConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::load();
        Self {
            mouse_through: Arc::new(Mutex::new(config.mouse_through)),
            auto_accept: Arc::new(Mutex::new(config.auto_accept)),
            lcu_auth: Arc::new(Mutex::new(None)),
            gameflow_phase: Arc::new(Mutex::new("None".to_string())),
            summoner_info: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(true)),
            config: Arc::new(Mutex::new(config)),
        }
    }
}