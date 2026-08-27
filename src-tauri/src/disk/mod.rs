use crate::app_resolver::AppIndex;
use crate::ScanItem;
use crate::scan_control;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sysinfo::Disks;
use walkdir::WalkDir;

pub mod delete;

pub use delete::{delete_permanent, move_to_trash};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub mount_point: String,
}

pub fn get_disk_info() -> Result<DiskInfo, String> {
    let disks = Disks::new_with_refreshed_list();
    let home = dirs_home();

    let disk = disks
        .iter()
        .find(|d| home.starts_with(d.mount_point()))
        .or_else(|| disks.iter().find(|d| d.mount_point() == "/"))
        .ok_or_else(|| "无法获取磁盘信息".to_string())?;

    let total = disk.total_space();
    let available = disk.available_space();
    let used = total.saturating_sub(available);
    let usage_percent = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    Ok(DiskInfo {
        total_bytes: total,
        used_bytes: used,
        available_bytes: available,
        usage_percent,
        mount_point: disk.mount_point().to_string_lossy().to_string(),
    })
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
}

pub fn path_size(path: &str) -> Result<u64, String> {
    let p = Path::new(path);
    if p.is_file() {
        return Ok(p.metadata().map_err(|e| e.to_string())?.len());
    }
    if p.is_dir() {
        return dir_size(p);
    }
    Ok(0)
}

fn dir_size(path: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    let mut index = 0u64;
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if scan_control::checkpoint(index) {
            break;
        }
        index += 1;
        if let Ok(meta) = entry.metadata() {
            total += meta.len();
        }
    }
    Ok(total)
}

fn safe_dir_size(path: &Path) -> u64 {
    dir_size(path).unwrap_or(0)
}

fn stable_item_id(prefix: &str, path: &str) -> String {
    format!("{prefix}_{}", path.replace('/', "_"))
}

/// 扫描目录下所有子文件夹，按体积从大到小排序（结果稳定、可预期）
fn scan_subdirs_sorted(dir: &Path, skip_hidden: bool) -> Vec<(PathBuf, String, u64)> {
    let entries: Vec<(PathBuf, String)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().to_string();
            if skip_hidden && name.starts_with('.') {
                return None;
            }
            Some((path, name))
        })
        .collect();

    let mut with_sizes: Vec<(PathBuf, String, u64)> = Vec::new();
    for (index, (path, name)) in entries.iter().enumerate() {
        if scan_control::checkpoint(index as u64) {
            break;
        }
        let size = safe_dir_size(path);
        if size > 0 {
            with_sizes.push((path.clone(), name.clone(), size));
        }
    }

    with_sizes.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));
    with_sizes
}

pub fn scan_phase(phase: &str) -> Result<Vec<ScanItem>, String> {
    let home = dirs_home();
    let mut items = Vec::new();

    match phase {
        "scanning_caches" => {
            let app_index = AppIndex::build();
            let caches = home.join("Library/Caches");
            if caches.exists() {
                for (path, name, size) in scan_subdirs_sorted(&caches, true) {
                    let path_str = path.to_string_lossy().to_string();
                    let resolved = app_index.resolve(&name);
                    let install_hint = if resolved.installed {
                        "已安装"
                    } else {
                        "可能已卸载"
                    };
                    items.push(ScanItem {
                        id: stable_item_id("cache", &path_str),
                        path: path_str,
                        size_bytes: size,
                        category: "cache".into(),
                        risk: "low".into(),
                        title: format!("{name} 缓存"),
                        description: format!(
                            "{} · 应用缓存，删除后会在需要时重建",
                            install_hint
                        ),
                        last_modified: modified_str(&path),
                        app_name: Some(resolved.app_name),
                        app_label: Some(resolved.app_label),
                        bundle_id: resolved.bundle_id,
                        app_installed: Some(resolved.installed),
                    });
                }
            }
        }
        "scanning_logs" => {
            let logs = home.join("Library/Logs");
            if logs.exists() {
                // 按子目录拆分，避免一个巨大「日志」项且结果不稳定
                let subdirs = scan_subdirs_sorted(&logs, true);
                if subdirs.is_empty() {
                    let size = safe_dir_size(&logs);
                    if size > 0 {
                        let path_str = logs.to_string_lossy().to_string();
                        items.push(ScanItem {
                            id: stable_item_id("log", &path_str),
                            path: path_str,
                            size_bytes: size,
                            category: "log".into(),
                            risk: "low".into(),
                            title: "系统与应用日志".into(),
                            description: "历史运行日志，删除不影响系统运行".into(),
                            last_modified: modified_str(&logs),
                            app_name: None,
                            app_label: None,
                            bundle_id: None,
                            app_installed: None,
                        });
                    }
                } else {
                    for (path, name, size) in subdirs {
                        let path_str = path.to_string_lossy().to_string();
                        items.push(ScanItem {
                            id: stable_item_id("log", &path_str),
                            path: path_str,
                            size_bytes: size,
                            category: "log".into(),
                            risk: "low".into(),
                            title: format!("{name} 日志"),
                            description: "应用运行日志，删除后会自动重建".into(),
                            last_modified: modified_str(&path),
                            app_name: None,
                            app_label: None,
                            bundle_id: None,
                            app_installed: None,
                        });
                    }
                }
            }
        }
        "scanning_trash" => {
            let trash = home.join(".Trash");
            if trash.exists() {
                let size = safe_dir_size(&trash);
                if size > 0 {
                    items.push(ScanItem {
                        id: "trash".into(),
                        path: trash.to_string_lossy().to_string(),
                        size_bytes: size,
                        category: "trash".into(),
                        risk: "low".into(),
                        title: "废纸篓".into(),
                        description: "已删除但未清空的文件".into(),
                        last_modified: modified_str(&trash),
                        app_name: None,
                        app_label: None,
                        bundle_id: None,
                        app_installed: None,
                    });
                }
            }
        }
        "scanning_leftovers" => {
            let app_index = AppIndex::build();
            let prefs = home.join("Library/Preferences");
            if prefs.exists() {
                let mut plist_items: Vec<(PathBuf, String, u64)> = std::fs::read_dir(&prefs)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) != Some("plist") {
                            return None;
                        }
                        let size = path.metadata().ok()?.len();
                        if size == 0 {
                            return None;
                        }
                        let name = path.file_stem()?.to_string_lossy().to_string();
                        Some((path, name, size))
                    })
                    .collect();
                plist_items.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));

                for (path, name, size) in plist_items {
                    let path_str = path.to_string_lossy().to_string();
                    let resolved = app_index.resolve(&name);
                    items.push(ScanItem {
                        id: stable_item_id("leftover", &path_str),
                        path: path_str,
                        size_bytes: size,
                        category: "leftover".into(),
                        risk: "low".into(),
                        title: format!("遗留配置: {name}"),
                        description: if resolved.installed {
                            "已安装应用的偏好设置文件".into()
                        } else {
                            "可能是已卸载应用的偏好设置文件".into()
                        },
                        last_modified: modified_str(&path),
                        app_name: Some(resolved.app_name),
                        app_label: Some(resolved.app_label),
                        bundle_id: resolved.bundle_id,
                        app_installed: Some(resolved.installed),
                    });
                }
            }
        }
        "scanning_large" => {
            let downloads = home.join("Downloads");
            if downloads.exists() {
                let mut large_files: Vec<(PathBuf, String, u64)> = std::fs::read_dir(&downloads)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter_map(|entry| {
                        let path = entry.path();
                        if !path.is_file() {
                            return None;
                        }
                        let size = path.metadata().ok()?.len();
                        if size < 100 * 1024 * 1024 {
                            return None;
                        }
                        let name = path.file_name()?.to_string_lossy().to_string();
                        Some((path, name, size))
                    })
                    .collect();
                large_files.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));

                for (path, name, size) in large_files {
                    let path_str = path.to_string_lossy().to_string();
                    items.push(ScanItem {
                        id: stable_item_id("large", &path_str),
                        path: path_str,
                        size_bytes: size,
                        category: "large_file".into(),
                        risk: "medium".into(),
                        title: name,
                        description: "下载文件夹中的大文件（>100MB）".into(),
                        last_modified: modified_str(&path),
                        app_name: None,
                        app_label: None,
                        bundle_id: None,
                        app_installed: None,
                    });
                }
            }
        }
        _ => {}
    }

    Ok(items)
}

fn modified_str(path: &Path) -> String {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            chrono::DateTime::<chrono::Local>::from(t)
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|| "-".into())
}
