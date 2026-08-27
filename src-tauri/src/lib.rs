mod app_icon;
mod app_resolver;
mod app_uninstaller;
mod disk;
mod folder_browse;
mod large_file_scan;
mod protected_paths;
mod scan_control;
mod tray;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    pub id: String,
    pub path: String,
    pub size_bytes: u64,
    pub category: String,
    pub risk: String,
    pub title: String,
    pub description: String,
    pub last_modified: String,
    /// 可读应用名，如 "Trae"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// 展示标签，如 "Trae 应用"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_label: Option<String>,
    /// 关联 Bundle ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    /// 是否仍在 Applications 中安装
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_installed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub phase: String,
    pub current_path: String,
    pub items_found: u32,
    pub bytes_found: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub success: Vec<String>,
    pub failed: Vec<CleanError>,
    pub total_freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanError {
    pub path: String,
    pub message: String,
}

#[tauri::command]
fn get_disk_usage() -> Result<disk::DiskInfo, String> {
    disk::get_disk_info()
}

#[tauri::command]
fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径不能为空".into());
    }
    if !std::path::Path::new(trimmed).exists() {
        return Err(format!("路径不存在: {trimmed}"));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", trimmed])
            .spawn()
            .map_err(|e| format!("无法在访达中打开: {e}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = trimmed;
        Err("仅支持 macOS".into())
    }
}

#[tauri::command]
fn open_privacy_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // 旧版系统偏好设置优先；macOS 14+ 可再尝试系统设置 URL
        let urls = [
            "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
            "x-apple.systemsettings:com.apple.settings.PrivacySecurity.extension?Privacy_AllFiles",
        ];

        for url in urls {
            match std::process::Command::new("open").arg(url).status() {
                Ok(status) if status.success() => return Ok(()),
                _ => continue,
            }
        }

        Err("无法打开系统隐私设置，请手动前往：系统设置 → 隐私与安全性 → 完全磁盘访问".into())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("仅支持 macOS".into())
    }
}

#[tauri::command]
async fn tauri_message(
    app: AppHandle,
    r#type: String,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let msg_type = r#type.trim();
    if msg_type.is_empty() {
        return Err("消息 type 不能为空".into());
    }

    let payload = match msg_type {
        "scan" => handle_scan(&app, &data).await?,
        "clean" => handle_clean(&data)?,
        "list_apps" => serde_json::to_value(app_uninstaller::list_installed_apps()?)
            .map_err(|e| e.to_string())?,
        "app_scan" => handle_app_scan(&data)?,
        "folder_scan" => handle_folder_browse(&data)?,
        "folder_browse" => handle_folder_browse(&data)?,
        "folder_analyze" => handle_folder_analyze(&data).await?,
        "folder_shortcuts" => serde_json::to_value(folder_browse::list_shortcuts())
            .map_err(|e| e.to_string())?,
        "large_file_scan" => handle_large_file_scan(&app, &data).await?,
        "scan_control" => handle_scan_control(&data)?,
        "large_file_scan_control" => handle_scan_control(&data)?,
        "disk_info" => serde_json::to_value(disk::get_disk_info()?).map_err(|e| e.to_string())?,
        other => return Err(format!("未知消息类型: {other}")),
    };

    Ok(serde_json::json!({
        "type": msg_type,
        "data": payload
    }))
}

async fn handle_scan(app: &AppHandle, data: &serde_json::Value) -> Result<serde_json::Value, String> {
    let mode = data
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("quick");

    let phases = if mode == "deep" {
        vec![
            ("scanning_caches", "正在扫描缓存"),
            ("scanning_logs", "正在扫描日志"),
            ("scanning_trash", "正在扫描废纸篓"),
            ("scanning_leftovers", "正在扫描系统遗留"),
            ("scanning_large", "正在扫描大文件"),
        ]
    } else {
        vec![
            ("scanning_caches", "正在扫描缓存"),
            ("scanning_logs", "正在扫描日志"),
            ("scanning_trash", "正在扫描废纸篓"),
        ]
    };

    scan_control::reset();
    let mut all_items: Vec<ScanItem> = Vec::new();
    let total_phases = phases.len() as f32;
    let mut cancelled = false;

    for (idx, (phase, label)) in phases.iter().enumerate() {
        scan_control::wait_if_paused();
        if scan_control::is_cancelled() {
            cancelled = true;
            break;
        }

        let progress = ScanProgress {
            phase: phase.to_string(),
            current_path: if scan_control::is_paused() {
                "扫描已暂停".into()
            } else {
                label.to_string()
            },
            items_found: all_items.len() as u32,
            bytes_found: all_items.iter().map(|i| i.size_bytes).sum(),
            percent: (idx as f32 / total_phases) * 100.0,
        };
        let _ = app.emit("scan_progress", &progress);
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        if scan_control::is_cancelled() {
            cancelled = true;
            break;
        }

        let items = disk::scan_phase(phase)?;
        all_items.extend(items);
    }

    let complete_progress = ScanProgress {
        phase: "complete".into(),
        current_path: if cancelled {
            "扫描已停止".into()
        } else {
            "扫描完成".into()
        },
        items_found: all_items.len() as u32,
        bytes_found: all_items.iter().map(|i| i.size_bytes).sum(),
        percent: 100.0,
    };
    let _ = app.emit("scan_progress", &complete_progress);
    let _ = app.emit("scan_complete", &all_items);

    Ok(serde_json::json!({
        "ok": true,
        "items": all_items,
        "total_bytes": all_items.iter().map(|i| i.size_bytes).sum::<u64>()
    }))
}

fn handle_clean(data: &serde_json::Value) -> Result<serde_json::Value, String> {
    let paths: Vec<String> = data
        .get("paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if paths.is_empty() {
        return Err("未选择任何文件".into());
    }

    let mode = data
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("trash");

    let mut success = Vec::new();
    let mut failed = Vec::new();
    let mut total_freed = 0u64;

    let mut size_index = 0usize;

    for path in paths {
        let size = data
            .get("sizes")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(size_index))
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| quick_path_size(&path));
        size_index += 1;

        let result = if mode == "permanent" {
            disk::delete_permanent(&path)
        } else {
            disk::move_to_trash(&path)
        };

        match result {
            Ok(_) => {
                total_freed += size;
                success.push(path);
            }
            Err(e) => failed.push(CleanError {
                path,
                message: e,
            }),
        }
    }

    let result = CleanResult {
        success,
        failed,
        total_freed_bytes: total_freed,
    };

    Ok(serde_json::json!(result))
}

/// 仅读取单文件体积，目录不递归（避免删除前统计卡住）
fn quick_path_size(path: &str) -> u64 {
    let p = std::path::Path::new(path);
    if p.is_file() {
        p.metadata().map(|m| m.len()).unwrap_or(0)
    } else {
        0
    }
}

fn handle_folder_browse(data: &serde_json::Value) -> Result<serde_json::Value, String> {
    let path = data
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    if path.is_empty() {
        return Err("path 不能为空".into());
    }

    let show_hidden = data
        .get("show_hidden")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let (items, summary) = folder_browse::list_directory(path, show_hidden)?;

    Ok(serde_json::json!({
        "ok": true,
        "items": items,
        "summary": summary
    }))
}

async fn handle_folder_analyze(data: &serde_json::Value) -> Result<serde_json::Value, String> {
    let paths: Vec<String> = data
        .get("paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    if paths.is_empty() {
        return Err("paths 不能为空".into());
    }

    let exact = data
        .get("exact")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let items = if exact {
        let scan_paths = paths.clone();
        tauri::async_runtime::spawn_blocking(move || folder_browse::analyze_items_exact(&scan_paths))
            .await
            .map_err(|e| format!("精确统计任务异常: {e}"))?
    } else {
        folder_browse::analyze_items(&paths)
    };

    Ok(serde_json::json!({
        "ok": true,
        "items": items
    }))
}

async fn handle_large_file_scan(
    app: &AppHandle,
    data: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let min_file_bytes = data
        .get("min_file_bytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(200 * 1024 * 1024);
    let min_folder_bytes = data
        .get("min_folder_bytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(1024 * 1024 * 1024);

    let app_handle = app.clone();
    scan_control::reset();
    let (items, summary, files_scanned) = tauri::async_runtime::spawn_blocking(move || {
        large_file_scan::scan_large_items(min_file_bytes, min_folder_bytes, |progress| {
            let _ = app_handle.emit("large_file_scan_progress", &progress);
        })
    })
    .await
    .map_err(|e| format!("扫描任务异常: {e}"))??;

    Ok(serde_json::json!({
        "ok": true,
        "items": items,
        "summary": summary,
        "files_scanned": files_scanned
    }))
}

fn handle_scan_control(data: &serde_json::Value) -> Result<serde_json::Value, String> {
    let action = data
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    scan_control::apply_action(action)?;
    Ok(serde_json::json!({ "ok": true }))
}

fn handle_app_scan(data: &serde_json::Value) -> Result<serde_json::Value, String> {
    let app_path = data
        .get("app_path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let bundle_id = data
        .get("bundle_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let app_name = data
        .get("app_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    if app_path.is_empty() || bundle_id.is_empty() {
        return Err("app_path 与 bundle_id 不能为空".into());
    }

    scan_control::reset();
    let items = app_uninstaller::scan_app_related(app_path, bundle_id, app_name)?;
    let total_bytes = items.iter().map(|i| i.size_bytes).sum::<u64>();

    Ok(serde_json::json!({
        "ok": true,
        "items": items,
        "total_bytes": total_bytes
    }))
}

const WINDOW_CORNER_RADIUS: f64 = 14.0;

#[cfg(target_os = "macos")]
fn apply_native_window_radius(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSColor, NSWindow};

    let Ok(ns_window_ptr) = window.ns_window() else {
        return;
    };

    let ns_window = unsafe { &*(ns_window_ptr as *mut NSWindow) };
    ns_window.setOpaque(false);
    ns_window.setBackgroundColor(Some(&NSColor::clearColor()));

    if let Some(content_view) = ns_window.contentView() {
        content_view.setWantsLayer(true);
        if let Some(layer) = content_view.layer() {
            layer.setCornerRadius(WINDOW_CORNER_RADIUS);
            layer.setMasksToBounds(true);
        }
    }

    let _ = window.set_shadow(true);
}

#[cfg(not(target_os = "macos"))]
fn apply_native_window_radius(_window: &tauri::WebviewWindow) {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_disk_usage, exit_app, reveal_in_finder, open_privacy_settings, tauri_message])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                apply_native_window_radius(&window);
            }
            #[cfg(desktop)]
            {
                let _ = tray::setup_tray(app.handle());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
