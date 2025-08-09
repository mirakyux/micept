use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub window_position: WindowPosition,
    pub mouse_through: bool,
    pub auto_accept: bool,
    pub auto_hide: bool,
    pub window_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_position: WindowPosition { x: -400, y: 0 }, // 默认右上角
            mouse_through: true,  // 默认开启鼠标穿透
            auto_accept: true,
            auto_hide: false,     // 默认关闭自动隐藏
            window_visible: true,
        }
    }
}

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("无法获取配置目录")?
            .join("micept");
        
        // 确保配置目录存在
        fs::create_dir_all(&config_dir)?;
        
        Ok(config_dir.join("config.json"))
    }

    /// 从文件加载配置
    pub fn load() -> Self {
        match Self::config_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match serde_json::from_str::<AppConfig>(&content) {
                                Ok(config) => {
                                    println!("成功加载配置文件: {:?}", config);
                                    return config;
                                }
                                Err(e) => {
                                    println!("解析配置文件失败: {}, 使用默认配置", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("读取配置文件失败: {}, 使用默认配置", e);
                        }
                    }
                } else {
                    println!("配置文件不存在，使用默认配置");
                }
            }
            Err(e) => {
                println!("获取配置文件路径失败: {}, 使用默认配置", e);
            }
        }
        
        Self::default()
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        println!("配置已保存到: {:?}", path);
        Ok(())
    }

    /// 更新窗口位置
    pub fn update_window_position(&mut self, x: i32, y: i32) {
        self.window_position.x = x;
        self.window_position.y = y;
        if let Err(e) = self.save() {
            println!("保存窗口位置失败: {}", e);
        }
    }

    /// 更新鼠标穿透状态
    pub fn update_mouse_through(&mut self, enabled: bool) {
        self.mouse_through = enabled;
        if let Err(e) = self.save() {
            println!("保存鼠标穿透状态失败: {}", e);
        }
    }

    /// 更新自动接受状态
    pub fn update_auto_accept(&mut self, enabled: bool) {
        self.auto_accept = enabled;
        if let Err(e) = self.save() {
            println!("保存自动接受状态失败: {}", e);
        }
    }

    /// 更新窗口可见性
    pub fn update_window_visible(&mut self, visible: bool) {
        self.window_visible = visible;
        if let Err(e) = self.save() {
            println!("保存窗口可见性失败: {}", e);
        }
    }

    /// 更新自动隐藏状态
    pub fn update_auto_hide(&mut self, enabled: bool) {
        self.auto_hide = enabled;
        if let Err(e) = self.save() {
            println!("保存自动隐藏状态失败: {}", e);
        }
    }
}