# 舰R助手启动器

Windows x64，Tauri 2 + Rust + React，基于 AzurPilotLauncher fork。原上游许可证和历史文件保留。

## 使用

把 `WarshipPilot.exe` 放在可写目录中运行，首次点击“部署”，完成后点击“启动”。停止服务和修复环境均在启动器首页；托盘菜单可恢复窗口或退出。

首次运行下载 Git/ADB、Python 3.14 与锁定依赖；不需要 Node.js。组件与独立引导在 `.warship/`，用户配置在 `config/`，日志在 `log/`。

网页更新在当前任务结束后委托启动器执行；依赖失败或新版本无法启动时恢复旧提交。启动器通过产品名、真实后台 PID 的 Job 归属、安装目录和提交号确认健康，端口被其他程序占用时不结束它。

## 接管现有安装

1. 先更新后端到支持 `deploy/launcher.py` 的版本，确认 Git 工作区干净。
2. 在旧控制台停止调度，等待当前任务完成，退出旧后台和包装程序；记录原调度开关与下次运行时间。
3. 备份 `config/deploy.yaml`、旧计划任务 XML 和快捷方式，禁用旧登录计划任务以防重复启动。
4. 把启动器放在已有安装根目录，部署后启动。旧的便携 `.venv` 会保留在 `.warship/backups/environment-*/.venv`，修复也会保留完整旧环境。
5. 验证访问码、任务参数和计划保持、MuMu 连接正常，再改快捷方式并启用启动器的开机自启。首次启动后根据记录恢复调度。

回退时先退出启动器，恢复对应备份环境到原 `.venv` 路径，以及备份的 `deploy.yaml`、旧快捷方式和计划任务。代码若已更新，需要恢复迁移前提交并同步该提交的依赖。不要删除配置、日志或计时文件。

## 构建和验证

```powershell
npm.cmd --prefix frontend ci
npm.cmd --prefix frontend test
npm.cmd --prefix frontend run build
$env:AZURPILOT_BOOTSTRAP_UV=(Get-Command uv).Source
cargo test --locked --all-targets
cargo build --locked --release --features custom-protocol
python tools/package_launcher.py
```

原始 Cargo 产物为 `target/release/warshippilot-launcher.exe`，交付名为 `WarshipPilot.exe`。

`WarshipPilot.exe --verify-install` 在当前安装目录验证部署、启动、健康检查和停止，结果写入 `log/launcher-verification.json`；`--repair` 先备份并重建环境。已有后台运行时会拒绝操作。

## 分发

Windows 工作流默认只构建并上传产物；显式启用 `publish` 才发布。组件以内容哈希命名，独立引导固定到后端完整提交 SHA 并逐文件校验。先上传组件和 EXE，最后发布 `manifest` Release 的 `stable.json`。公开镜像与直连地址均列入清单。

独立 EXE 的首次联网部署需要已发布的舰R清单；仅有本地构建产物不能替代发布验证。
