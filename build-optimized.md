# EXE体积优化说明

## 已实施的优化措施

### 1. Rust编译优化 (Cargo.toml)
- `opt-level = "z"`: 使用最激进的体积优化
- `lto = true`: 启用链接时优化
- `codegen-units = 1`: 单个代码生成单元，提高优化效果
- `panic = "abort"`: 禁用panic展开，减小体积
- `strip = true`: 移除调试符号
- `overflow-checks = false`: 禁用溢出检查
- `debug = false`: 完全禁用调试信息
- `rpath = false`: 禁用rpath

### 2. Cargo编译器标志 (.cargo/config.toml)
- `target-cpu=native`: 针对本机CPU优化
- `link-arg=-s`: 链接时strip符号
- `target-feature=+crt-static`: 静态链接CRT
- `SUBSYSTEM:WINDOWS`: Windows子系统优化

### 3. Tauri配置优化 (tauri.conf.json)
- `removeUnusedCommands: true`: 移除未使用的命令
- `withGlobalTauri: false`: 不使用全局Tauri
- `compression: "lzma2"`: 使用LZMA2压缩安装包

### 4. 前端构建优化 (vite.config.ts)
- `minify: 'terser'`: 使用Terser压缩
- `drop_console: true`: 移除console语句
- `drop_debugger: true`: 移除debugger语句
- `manualChunks: undefined`: 禁用代码分割

## 构建命令
```bash
# 构建优化版本
pnpm tauri build

# 或者使用release模式
pnpm tauri build --release
```

## 预期效果
- 可减少30-50%的exe文件体积
- 安装包体积也会相应减小
- 启动速度可能略有提升

## 注意事项
- 这些优化可能会增加编译时间
- 某些调试功能将不可用
- 如需调试，请使用dev模式构建