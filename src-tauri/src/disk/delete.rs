use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const DELETE_TIMEOUT: Duration = Duration::from_secs(20);

/// 移到废纸篓（带超时，避免系统弹窗导致界面一直卡住）
pub fn move_to_trash(path: &str) -> Result<(), String> {
    validate_deletable(path)?;
    let path = path.to_string();
    run_with_timeout(DELETE_TIMEOUT, move || {
        trash::delete(&path).map_err(|e| friendly_error(&e.to_string(), &path))
    })
}

/// 永久删除（带超时）
pub fn delete_permanent(path: &str) -> Result<(), String> {
    validate_deletable(path)?;
    let path = path.to_string();
    run_with_timeout(DELETE_TIMEOUT, move || {
        let p = Path::new(&path);
        if p.is_dir() {
            std::fs::remove_dir_all(p).map_err(|e| friendly_error(&e.to_string(), &path))
        } else if p.is_file() {
            std::fs::remove_file(p).map_err(|e| friendly_error(&e.to_string(), &path))
        } else if p.exists() {
            std::fs::remove_file(p).map_err(|e| friendly_error(&e.to_string(), &path))
        } else {
            Err("路径不存在或已被删除".into())
        }
    })
}

fn run_with_timeout<F>(timeout: Duration, op: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String> + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(op());
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(
            "操作超时：可能被系统拦截、文件正在使用，或需要权限确认。请检查系统提示后重试".into(),
        ),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err("删除线程异常结束，请重试".into())
        }
    }
}

fn validate_deletable(path: &str) -> Result<(), String> {
    crate::protected_paths::validate_deletable(path)
}

fn friendly_error(raw: &str, path: &str) -> String {
    let lower = raw.to_lowercase();
    let name = PathBuf::from(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    if lower.contains("permission")
        || lower.contains("denied")
        || lower.contains("not authorized")
        || lower.contains("operation not permitted")
    {
        return format!("「{name}」权限不足，可能需要在系统设置中授予完全磁盘访问权限");
    }

    if lower.contains("busy")
        || lower.contains("in use")
        || lower.contains("locked")
        || lower.contains("resource busy")
    {
        return format!("「{name}」正在使用中，无法删除");
    }

    if lower.contains("no such file") || lower.contains("not found") {
        return format!("「{name}」不存在或已被删除");
    }

    if lower.contains("directory not empty") {
        return format!("「{name}」目录非空且无法完整删除");
    }

    if raw.trim().is_empty() {
        return format!("「{name}」无法删除，可能被系统保护");
    }

    format!("「{name}」无法删除: {raw}")
}
