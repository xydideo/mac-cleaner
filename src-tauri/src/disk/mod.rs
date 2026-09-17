use crate::app_resolver::AppIndex;
use crate::protected_paths;
use crate::ScanItem;
use crate::scan_control;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

fn include_in_junk_scan(path: &Path) -> bool {
    !protected_paths::is_excluded_from_junk_scan(path)
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

pub fn scan_phase(phase: &str, deep: bool) -> Result<Vec<ScanItem>, String> {
    let home = dirs_home();
    let mut items = Vec::new();

    match phase {
        "scanning_caches" => {
            let app_index = AppIndex::build();
            let caches = home.join("Library/Caches");
            if caches.exists() {
                for (path, name, size) in scan_subdirs_sorted(&caches, true) {
                    if !include_in_junk_scan(&path) {
                        continue;
                    }
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
            if deep {
                items.extend(scan_update_cache(&home, &app_index));
            }
        }
        "scanning_logs" => {
            let logs = home.join("Library/Logs");
            if logs.exists() {
                // 按子目录拆分，避免一个巨大「日志」项且结果不稳定
                let subdirs = scan_subdirs_sorted(&logs, true);
                if subdirs.is_empty() {
                    let size = safe_dir_size(&logs);
                    if size > 0 && include_in_junk_scan(&logs) {
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
                        if !include_in_junk_scan(&path) {
                            continue;
                        }
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
                    let resolved = app_index.resolve(&name);
                    let installed = resolved.installed;
                    if !protected_paths::is_eligible_third_party_leftover_preference(&name, installed)
                    {
                        continue;
                    }
                    if !include_in_junk_scan(&path) {
                        continue;
                    }
                    let path_str = path.to_string_lossy().to_string();
                    items.push(ScanItem {
                        id: stable_item_id("leftover", &path_str),
                        path: path_str,
                        size_bytes: size,
                        category: "leftover".into(),
                        risk: "low".into(),
                        title: format!("第三方遗留: {name}"),
                        description: "已卸载第三方应用的偏好设置，删除一般不影响系统与在装应用".into(),
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
                    if !include_in_junk_scan(&path) {
                        continue;
                    }
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

/// 深度扫描：Application Support 内应用更新缓存与更新包（并入「应用缓存」阶段，无独立进度）
const MIN_UPDATE_ITEM_BYTES: u64 = 1024 * 1024;
const UPDATE_SCAN_MAX_DEPTH: usize = 12;
/// 路径中任一层目录名命中即视为更新相关（不区分大小写）
const UPDATE_PATH_KEYWORDS_EXACT: &[&str] = &[
    "update",
    "updates",
    "pending",
    "download",
    "downloads",
    "staging",
    "patch",
    "upgrade",
    "sparkle",
];
const UPDATE_PATH_KEYWORDS_PARTIAL: &[&str] = &[
    "updater",
    "googleupdater",
    "autoupdate",
    "auto-update",
    "crx_cache",
    "installer",
    "mau",
    "cryptex",
];
/// 整目录作为「更新缓存」上报的文件夹名
const UPDATE_CACHE_DIR_NAMES: &[&str] = &["crx_cache", "pending"];
const FIXED_UPDATE_CACHE_PATHS: &[&str] = &[
    "Library/Application Support/Google/GoogleUpdater/crx_cache",
    "Library/Application Support/Microsoft/MAU2.0",
];
const UPDATE_PACKAGE_EXTS: &[&str] = &["dmg", "pkg", "zip", "crx", "ipa"];

fn scan_update_cache(home: &Path, app_index: &AppIndex) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut cache_dir_roots = HashSet::new();

    for rel in FIXED_UPDATE_CACHE_PATHS {
        let path = home.join(rel);
        push_update_cache_dir(
            &mut items,
            &mut seen_paths,
            &mut cache_dir_roots,
            &path,
            app_index,
            "更新缓存",
        );
    }

    let app_support = home.join("Library/Application Support");
    if !app_support.exists() {
        items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then_with(|| a.title.cmp(&b.title)));
        return items;
    }

    let mut index = 0u64;
    for entry in WalkDir::new(&app_support)
        .max_depth(UPDATE_SCAN_MAX_DEPTH)
        .into_iter()
        .filter_entry(|entry| !protected_paths::should_prune_scan_entry(entry.path()))
        .filter_map(|e| e.ok())
    {
        if scan_control::checkpoint(index) {
            break;
        }
        index += 1;

        let path = entry.path();
        if path == app_support.as_path() {
            continue;
        }
        if !include_in_junk_scan(path) {
            continue;
        }

        if entry.file_type().is_dir() {
            if is_update_cache_dir(path) {
                push_update_cache_dir(
                    &mut items,
                    &mut seen_paths,
                    &mut cache_dir_roots,
                    path,
                    app_index,
                    "更新缓存",
                );
            }
            continue;
        }

        if !entry.file_type().is_file() {
            continue;
        }
        if is_inside_any(path, &cache_dir_roots) {
            continue;
        }
        if !is_update_package_file(path) {
            continue;
        }
        if !is_update_related_path(path, &app_support) {
            continue;
        }

        push_update_package_file(&mut items, &mut seen_paths, path, app_index);
    }

    items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then_with(|| a.title.cmp(&b.title)));
    items
}

fn is_update_cache_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| {
            let lower = name.to_lowercase();
            UPDATE_CACHE_DIR_NAMES
                .iter()
                .any(|n| lower == *n || lower.contains(n))
        })
        .unwrap_or(false)
}

fn path_component_matches_update_keyword(component: &str) -> bool {
    let lower = component.to_lowercase();
    if UPDATE_PATH_KEYWORDS_EXACT.iter().any(|kw| lower == *kw) {
        return true;
    }
    UPDATE_PATH_KEYWORDS_PARTIAL
        .iter()
        .any(|kw| lower.contains(kw))
}

fn is_update_related_path(path: &Path, app_support: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(app_support) else {
        return false;
    };
    rel.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .map(path_component_matches_update_keyword)
            .unwrap_or(false)
    })
}

fn is_update_package_file(path: &Path) -> bool {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext.to_lowercase(),
        None => return false,
    };
    if !UPDATE_PACKAGE_EXTS.contains(&ext.as_str()) {
        return false;
    }
    path.metadata()
        .map(|m| m.len() >= MIN_UPDATE_ITEM_BYTES)
        .unwrap_or(false)
}

fn is_inside_any(path: &Path, roots: &HashSet<String>) -> bool {
    let path_str = path.to_string_lossy();
    roots.iter().any(|root| path_str.starts_with(root))
}

fn push_update_cache_dir(
    items: &mut Vec<ScanItem>,
    seen_paths: &mut HashSet<String>,
    cache_dir_roots: &mut HashSet<String>,
    path: &Path,
    app_index: &AppIndex,
    kind_label: &str,
) {
    if !path.is_dir() {
        return;
    }
    if !include_in_junk_scan(path) {
        return;
    }
    let path_str = path.to_string_lossy().to_string();
    if !seen_paths.insert(path_str.clone()) {
        return;
    }

    let size = safe_dir_size(path);
    if size < MIN_UPDATE_ITEM_BYTES {
        return;
    }

    cache_dir_roots.insert(format!("{path_str}/"));

    let resolved = app_index.resolve_for_path(path);
    items.push(ScanItem {
        id: stable_item_id("update_cache", &path_str),
        path: path_str,
        size_bytes: size,
        category: "cache".into(),
        risk: "low".into(),
        title: format!("{} {}", resolved.app_name, kind_label),
        description: "应用更新下载缓存，删除后不影响正常使用，下次更新时会重新下载".into(),
        last_modified: modified_str(path),
        app_name: Some(resolved.app_name),
        app_label: Some(resolved.app_label),
        bundle_id: resolved.bundle_id,
        app_installed: Some(resolved.installed),
    });
}

fn push_update_package_file(
    items: &mut Vec<ScanItem>,
    seen_paths: &mut HashSet<String>,
    path: &Path,
    app_index: &AppIndex,
) {
    if !include_in_junk_scan(path) {
        return;
    }
    let path_str = path.to_string_lossy().to_string();
    if !seen_paths.insert(path_str.clone()) {
        return;
    }

    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
    if size < MIN_UPDATE_ITEM_BYTES {
        return;
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "更新包".into());
    let resolved = app_index.resolve_for_path(path);
    items.push(ScanItem {
        id: stable_item_id("update_package", &path_str),
        path: path_str,
        size_bytes: size,
        category: "cache".into(),
        risk: "low".into(),
        title: format!("{} 更新包 · {name}", resolved.app_name),
        description: "Application Support 中的应用更新安装包，删除后不影响已安装版本".into(),
        last_modified: modified_str(path),
        app_name: Some(resolved.app_name),
        app_label: Some(resolved.app_label),
        bundle_id: resolved.bundle_id,
        app_installed: Some(resolved.installed),
    });
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
