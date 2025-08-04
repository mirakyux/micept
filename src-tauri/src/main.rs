// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    {
        // 检查是否有管理员权限
        if !is_elevated::is_elevated() {
            // 没有管理员权限，尝试重启获取权限
            println!("应用需要管理员权限才能正常运行，正在尝试重启...");
            
            // 获取当前可执行文件路径
            if let Ok(current_exe) = std::env::current_exe() {
                if let Some(exe_path) = current_exe.to_str() {
                    // 使用runas库以管理员权限重启应用
                    match runas::Command::new(exe_path).gui(true).status() {
                        Ok(_) => {
                            // 重启成功，退出当前实例
                            std::process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("无法以管理员权限重启应用: {}", e);
                            // 继续运行，但可能功能受限
                        }
                    }
                }
            }
        }
    }
    
    // 正常启动应用
    micept_lib::run()
}
