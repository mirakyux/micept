// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// 模块声明
mod app;
mod background;
mod commands;
mod config;
mod lcu;
mod state;
mod tray;

// 重新导出主要的运行函数
pub use app::run;
