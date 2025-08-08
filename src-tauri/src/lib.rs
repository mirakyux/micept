// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// 模块声明
mod app;
mod core;
mod ui;
mod lol;
mod utils;
mod commands;

// 重新导出主要的运行函数
pub use app::run;
