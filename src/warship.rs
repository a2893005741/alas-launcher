//! WarshipPilot contract. Component and process mechanisms stay in their owning modules.
use crate::window_util::CreateNoWindow as _;
use crate::{components, progress::SplashUpdate, setup, sources::Preferences};
use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{atomic::AtomicBool, Arc},
};

fn entry(root: &Path) -> PathBuf {
    root.join(".warship/bootstrap/deploy/launcher.py")
}

pub(crate) fn command(root: &Path, action: &str) -> Command {
    let mut cmd = Command::new(setup::venv_python());
    cmd.arg(entry(root))
        .arg(action)
        .arg("--root")
        .arg(root)
        .arg("--json");
    cmd.current_dir(root).create_no_window();
    setup::isolate_python_child_environment(&mut cmd);
    cmd
}

fn execute(mut cmd: Command, root: &Path, cancel: &AtomicBool) -> Result<()> {
    let log_dir = root.join("log");
    fs::create_dir_all(&log_dir)?;
    let log = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_dir.join("launcher-deploy.log"))?;
    cmd.stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log));
    if !setup::run_status_command(&mut cmd, cancel)?.success() {
        bail!("部署步骤失败，详情见 log/launcher-deploy.log");
    }
    Ok(())
}

fn ensure_not_running(root: &Path) -> Result<()> {
    let port =
        crate::backend::WebuiLaunchConfig::from_deploy_config(setup::get_deploy_config().as_ref())
            .port;
    if std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse()?,
        std::time::Duration::from_millis(300),
    )
    .is_ok()
    {
        bail!("端口 {port} 正在使用。请先在旧控制台停止任务并退出旧后台，再迁移或修复。");
    }
    // Lock ownership matters even when another installer has not opened a port.
    let python = setup::venv_python();
    if python.is_file() && root.join("deploy/lock.py").is_file() {
        let mut cmd = Command::new(python);
        cmd.args(["-c", "from deploy.lock import instance_lock; import sys; c=instance_lock(sys.argv[1]); c.__enter__(); c.__exit__(None,None,None)"]).arg(root);
        cmd.current_dir(root).create_no_window();
        setup::isolate_python_child_environment(&mut cmd);
        if !cmd.status()?.success() {
            bail!("此安装已有后台或安装器运行，请先退出");
        }
    }
    Ok(())
}

/// Keep old environments and configuration until the user explicitly cleans backups.
fn backup_environment(root: &Path) -> Result<Option<PathBuf>> {
    let venv = root.join(".venv");
    if !venv.exists() {
        return Ok(None);
    }
    if crate::fs_util::is_symlink_or_reparse_point(&fs::symlink_metadata(&venv)?) {
        bail!("不能迁移链接形式的 .venv，请使用独立安装目录");
    }
    let backup = root.join(".warship/backups").join(format!(
        "environment-{}",
        chrono::Local::now().format("%Y%m%d-%H%M%S-%f")
    ));
    fs::create_dir_all(&backup)?;
    let config = root.join("config/deploy.yaml");
    if config.exists() {
        fs::copy(&config, backup.join("deploy.yaml"))?;
    }
    fs::write(
        root.join(".warship/environment-backup.json"),
        serde_json::to_vec(&serde_json::json!({"backup":backup,"configExisted":config.exists()}))?,
    )?;
    fs::rename(&venv, backup.join(".venv")).context("旧环境仍在使用，无法备份")?;
    Ok(Some(backup))
}

fn restore_environment(root: &Path, backup: &Path) -> Result<()> {
    let venv = root.join(".venv");
    if venv.exists() {
        fs::rename(
            &venv,
            backup.join(format!(
                "failed-environment-{}",
                chrono::Local::now().timestamp_millis()
            )),
        )?;
    }
    fs::rename(backup.join(".venv"), &venv)?;
    if backup.join("deploy.yaml").exists() {
        fs::copy(backup.join("deploy.yaml"), root.join("config/deploy.yaml"))?;
    }
    let record = root.join(".warship/environment-backup.json");
    if record.exists() {
        fs::remove_file(record)?;
    }
    Ok(())
}

fn pending_environment(root: &Path) -> Option<PathBuf> {
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(".warship/environment-backup.json")).ok()?)
            .ok()?;
    if value["committed"] == true {
        return None;
    }
    let path = PathBuf::from(value["backup"].as_str()?);
    (path.starts_with(root.join(".warship/backups")) && path.join(".venv").is_dir()).then_some(path)
}

fn prepare(
    mut progress: impl FnMut(SplashUpdate),
    cancel: Arc<AtomicBool>,
    skip: bool,
    sources: &Preferences,
    repair: bool,
) -> Result<()> {
    let root = setup::azurpilot_repo_dir();
    crate::launcher::ensure_not_cancelled(&cancel)?;
    ensure_not_running(&root)?;
    if let Some(backup) = pending_environment(&root) {
        restore_environment(&root, &backup)?;
    }
    if repair {
        components::repair(&root, &mut progress, cancel.clone(), sources)?;
    }
    if components::deployment_required(&root) {
        bail!("请先部署运行组件");
    }
    // Existing portable runtimes have a flat .venv/python/python.exe layout;
    // uv-managed runtimes contain cpython-* directories instead.
    let legacy = root.join(".venv/python/python.exe").exists();
    let backup = if repair || legacy {
        backup_environment(&root)?
    } else {
        None
    };
    let result = (|| -> Result<()> {
        let uv = setup::bootstrap_uv_path()?;
        setup::ensure_runtime_tools(&uv, &cancel, &mut progress, sources)?;
        let git = components::available_tool(&root, "git").context("缺少 Git 组件")?;
        let adb = components::available_tool(&root, "adb").context("缺少 ADB 组件")?;
        progress(SplashUpdate::loading(
            "准备舰R助手源码",
            "检查已有安装或获取源码",
            41,
        ));
        let mut bootstrap = command(&root, "bootstrap");
        bootstrap.arg("--git").arg(&git);
        let repositories = [
            "https://ghfast.top/https://github.com/a2893005741/WarshipPilot.git",
            "https://github.com/a2893005741/WarshipPilot.git",
        ];
        let mut repositories = repositories
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        #[cfg(debug_assertions)]
        if let Ok(repository) = std::env::var("WARSHIP_TEST_REPOSITORY") {
            repositories = vec![repository];
        }
        for url in sources.download_source.order_urls(&repositories) {
            bootstrap.arg("--repository").arg(url);
        }
        execute(bootstrap, &root, &cancel)?;
        let mut config = command(&root, "config");
        config
            .arg("--git")
            .arg(&git)
            .arg("--adb")
            .arg(adb)
            .arg("--python")
            .arg(setup::venv_python())
            .arg("--uv")
            .arg(setup::venv_uv());
        execute(config, &root, &cancel)?;
        progress(SplashUpdate::loading(
            "检查源码更新",
            "保留配置与调度状态",
            48,
        ));
        let mut update = command(&root, "update");
        if skip || repair {
            update.arg("--skip-update");
        }
        execute(update, &root, &cancel)?;
        progress(SplashUpdate::loading(
            "同步运行依赖",
            "按 uv.lock 安装并校验 ADB",
            65,
        ));
        execute(command(&root, "sync"), &root, &cancel)?;
        progress(SplashUpdate::loading("部署完成", "等待后台健康检查", 94));
        Ok(())
    })();
    if result.is_err() {
        if let Some(backup) = backup {
            restore_environment(&root, &backup)?;
        }
    }
    result
}

pub(crate) fn setup(
    progress: impl FnMut(SplashUpdate),
    cancel: Arc<AtomicBool>,
    skip: bool,
    sources: &Preferences,
) -> Result<()> {
    prepare(progress, cancel, skip, sources, false)
}
pub(crate) fn repair(
    progress: impl FnMut(SplashUpdate),
    cancel: Arc<AtomicBool>,
    sources: &Preferences,
) -> Result<()> {
    prepare(progress, cancel, true, sources, true)
}
pub(crate) fn commit(cancel: &AtomicBool) -> Result<()> {
    let root = setup::azurpilot_repo_dir();
    execute(command(&root, "commit"), &root, cancel)?;
    let record = root.join(".warship/environment-backup.json");
    if record.exists() {
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&record)?)?;
        value["committed"] = serde_json::json!(true);
        fs::write(record, serde_json::to_vec(&value)?)?;
    }
    Ok(())
}
pub(crate) fn rollback(cancel: &AtomicBool) -> Result<()> {
    let root = setup::azurpilot_repo_dir();
    execute(command(&root, "rollback"), &root, cancel)
}
pub(crate) fn has_pending_update() -> bool {
    setup::azurpilot_repo_dir()
        .join(".warship/transaction.json")
        .exists()
}

pub(crate) fn update_to(target: &str, cancel: &AtomicBool) -> Result<()> {
    let root = setup::azurpilot_repo_dir();
    let mut update = command(&root, "update");
    update.arg("--target").arg(target);
    execute(update, &root, cancel)?;
    execute(command(&root, "sync"), &root, cancel)
}

/// Run the same deployment/backend contract without WebView or game input.
/// Useful for support diagnostics in a separate installation directory.
pub(crate) fn verify_install() -> Result<()> {
    crate::i18n::init();
    setup::setup_environment()?;
    let _guard = crate::initialize_logging()?;
    let root = setup::azurpilot_repo_dir();
    let cancel = Arc::new(AtomicBool::new(false));
    let sources = Preferences::default();
    let progress =
        |p: SplashUpdate| tracing::info!("{}: {} ({:.0}%)", p.title, p.detail, p.progress);
    if std::env::args().any(|a| a == "--repair") {
        repair(progress, cancel.clone(), &sources)?;
    } else {
        if components::deployment_required(&root) {
            components::deploy(
                &root,
                &mut |p| tracing::info!("{}", p.detail),
                cancel.clone(),
                false,
                &sources,
            )?;
        }
        setup(
            progress,
            cancel.clone(),
            std::env::args().any(|a| a == "--skip-update"),
            &sources,
        )?;
    }
    let config =
        crate::backend::WebuiLaunchConfig::from_deploy_config(setup::get_deploy_config().as_ref());
    let (mut backend, recovered) = launch(&config, &cancel)?;
    backend.terminate()?;
    fs::write(
        root.join("log/launcher-verification.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "ok":true, "root":root, "port":config.port, "transactionPending":has_pending_update(), "recovered":recovered
        }))?,
    )?;
    Ok(())
}

pub(crate) fn launch(
    config: &crate::backend::WebuiLaunchConfig,
    cancel: &AtomicBool,
) -> Result<(crate::backend::ManagedBackend, bool)> {
    match crate::backend::ManagedBackend::new_cancellable(config, cancel) {
        Ok(backend) => Ok((backend, false)),
        Err(error) => {
            let root = setup::azurpilot_repo_dir();
            let environment = pending_environment(&root);
            if !has_pending_update() && environment.is_none() {
                return Err(error);
            }
            tracing::warn!("新版本或新环境启动失败，恢复旧安装：{error:#}");
            if has_pending_update() {
                rollback(cancel)?;
            }
            if let Some(backup) = environment {
                restore_environment(&root, &backup)?;
            }
            Ok((
                crate::backend::ManagedBackend::new_cancellable(config, cancel)?,
                true,
            ))
        }
    }
}
