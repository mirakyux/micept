# Rust 代码重构说明

## 重构目标
根据功能和作用对所有 Rust 代码进行拆分和统合，提高代码的可维护性和可读性。

## 重构后的目录结构

```
src/
├── app.rs              # 应用程序入口点
├── commands.rs         # Tauri 命令处理器
├── lib.rs             # 库入口文件
├── main.rs            # 主函数
├── core/              # 核心模块
│   ├── mod.rs         # 模块声明
│   ├── state.rs       # 应用状态管理
│   └── background.rs  # 后台任务处理
├── ui/                # 用户界面模块
│   ├── mod.rs         # 模块声明
│   ├── window.rs      # 窗口管理
│   └── tray.rs        # 系统托盘
├── lol/               # 英雄联盟客户端相关
│   ├── mod.rs         # 模块声明
│   └── client.rs      # LCU 客户端交互 (原 lcu.rs)
└── utils/             # 工具模块
    ├── mod.rs         # 模块声明
    └── config.rs      # 配置管理
```

## 模块功能说明

### core/ - 核心模块
- **state.rs**: 应用状态管理，包含 `AppState` 结构体
- **background.rs**: 后台任务处理，负责与 LCU 的持续通信

### ui/ - 用户界面模块
- **window.rs**: 窗口设置和管理功能
- **tray.rs**: 系统托盘创建和事件处理

### lol/ - 英雄联盟客户端模块
- **client.rs**: 与 LCU (League Client Update) 的所有交互功能
  - 管理员权限检查
  - LCU 认证信息获取
  - 召唤师信息获取
  - 游戏流程状态获取
  - 自动接受匹配

### utils/ - 工具模块
- **config.rs**: 应用配置管理，包含配置的加载、保存和更新

## 重构的主要改进

1. **模块化**: 按功能将代码分组到不同的模块中
2. **职责分离**: 每个模块都有明确的职责范围
3. **可维护性**: 相关功能集中在一起，便于维护和扩展
4. **可读性**: 清晰的目录结构和命名约定
5. **重用性**: 模块化设计便于代码重用

## 文件移动记录

- `state.rs` → `core/state.rs`
- `background.rs` → `core/background.rs`
- `config.rs` → `utils/config.rs`
- `lcu.rs` → `lol/client.rs`
- `tray.rs` → `ui/tray.rs`
- 新增 `ui/window.rs` (从 `app.rs` 中分离出窗口设置功能)

## 导入路径更新

所有相关文件的导入路径都已更新以反映新的模块结构：
- `use crate::state::AppState` → `use crate::core::AppState`
- `use crate::lcu` → `use crate::lol`
- `use crate::config::AppConfig` → `use crate::utils::AppConfig`

## 编译状态

✅ 重构完成后项目编译通过，无错误。