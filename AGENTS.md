# 舰R助手启动器工作约定

用户已授权从 AzurPilot fork 改造为 WarshipPilot，当前交付平台为 Windows x64。
- 产品名舰R助手，EXE 为 WarshipPilot.exe；内部 AP_/ALAS_ 环境协议保留兼容。
- 四语言文件继续同步；所有新增子进程隐藏窗口；日志用 tracing，敏感 URL/令牌不打印。
- config/log/用户状态保留，更新用事务与健康检查；修复保留旧环境备份。
- 当前实现见 src/warship.rs；来源与旧跨平台机制见下方历史说明。
- 构建/验证：npm --prefix frontend ci/test/run build；cargo check/test --locked --all-targets；cargo build --locked --release --features custom-protocol。

历史上游约定保存在 docs/AGENTS.upstream.md，仅供合并参考；当前产品与部署以 README.md 为准。
- 新增用户文案同步四语言词典；发布版本同步 Cargo.toml、tauri.conf.json、CHANGELOG.md。
- 仅 CI 输出 URL 掩码，更新 tools/ci/log-mask.txt 后同步 workflow 的 AZURPILOT_LOG_MASK_B64。
- 不再依赖上游的 mTLS、R2、Worker、CNB 发布服务。Windows asInvoker，不请求管理员。
