use crate::app_resolver::AppIndex;
use crate::folder_browse::{BrowseItem, FolderBrowseSummary};
use crate::protected_paths;
use crate::scan_control;
use chrono::{DateTime, Local};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_RESULTS: usize = 800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LargeScanKind {
    Folders,
    Files,
}

impl LargeScanKind {
    pub fn parse(value: &str) -> Self {
        match value {
            "files" => Self::Files,
            _ => Self::Folders,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LargeFileScanProgress {
    pub current_path: String,
    pub files_scanned: u64,
    pub items_found: u32,
}

pub fn scan_large_items(
    min_file_bytes: u64,
    min_folder_bytes: u64,
    kind: LargeScanKind,
    mut on_progress: impl FnMut(LargeFileScanProgress),
) -> Result<(Vec<BrowseItem>, FolderBrowseSummary, u64), String> {
    let roots = scan_roots();
    if roots.is_empty() {
        return Err("无法确定扫描根目录".into());
    }

    let min_file_mb = min_file_bytes / 1024 / 1024;
    let min_folder_mb = min_folder_bytes / 1024 / 1024;
    let app_index = AppIndex::build();
    let mut file_items: Vec<BrowseItem> = Vec::new();
    let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();
    let mut files_scanned = 0u64;
    let mut current_path = String::new();
    let mut cancelled = false;

    'root_loop: for root in &roots {
        let walker = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !protected_paths::should_prune_scan_entry(entry.path()));

        for entry in walker.filter_map(|e| e.ok()) {
            if scan_control::checkpoint(files_scanned) {
                cancelled = true;
                break 'root_loop;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            files_scanned += 1;
            current_path = path.to_string_lossy().to_string();

            if files_scanned % 400 == 0 {
                let items_found = match kind {
                    LargeScanKind::Files => file_items.len() as u32,
                    LargeScanKind::Folders => count_large_dirs(&dir_sizes, min_folder_bytes),
                };
                on_progress(LargeFileScanProgress {
                    current_path: current_path.clone(),
                    files_scanned,
                    items_found,
                });
            }

            let size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
            if size == 0 {
                continue;
            }

            if kind == LargeScanKind::Folders {
                let mut parent = path.parent();
                while let Some(p) = parent {
                    if !is_under_roots(p, &roots) {
                        break;
                    }
                    *dir_sizes.entry(p.to_path_buf()).or_default() += size;
                    parent = p.parent();
                }
            } else if size >= min_file_bytes && !protected_paths::is_protected_path(path) {
                file_items.push(build_file_item(path, size, min_file_mb, &app_index));
            }
        }
    }

    let items = match kind {
        LargeScanKind::Folders => build_folder_items(
            dir_sizes,
            min_folder_bytes,
            min_folder_mb,
            &app_index,
        ),
        LargeScanKind::Files => {
            file_items.sort_by(|a, b| {
                b.size_bytes
                    .cmp(&a.size_bytes)
                    .then_with(|| a.path.cmp(&b.path))
            });
            file_items.truncate(MAX_RESULTS);
            file_items
        }
    };

    on_progress(LargeFileScanProgress {
        current_path: if cancelled {
            "扫描已停止".into()
        } else {
            "扫描完成".into()
        },
        files_scanned,
        items_found: items.len() as u32,
    });

    let scan_kind_label = match kind {
        LargeScanKind::Folders => "folders",
        LargeScanKind::Files => "files",
    };
    let summary = FolderBrowseSummary {
        path: format!(
            "large-file-scan://{scan_kind_label}?file≥{min_file_mb}MB,folder≥{min_folder_mb}MB"
        ),
        total_bytes: items.iter().map(|i| i.size_bytes).sum(),
        folder_count: items.iter().filter(|i| i.is_directory).count() as u32,
        file_count: items.iter().filter(|i| !i.is_directory).count() as u32,
        junk_bytes: 0,
        junk_count: 0,
        pending_count: 0,
    };

    Ok((items, summary, files_scanned))
}

fn count_large_dirs(dir_sizes: &HashMap<PathBuf, u64>, min_folder_bytes: u64) -> u32 {
    dir_sizes
        .values()
        .filter(|size| **size >= min_folder_bytes)
        .count() as u32
}

fn build_folder_items(
    dir_sizes: HashMap<PathBuf, u64>,
    min_folder_bytes: u64,
    min_folder_mb: u64,
    app_index: &AppIndex,
) -> Vec<BrowseItem> {
    let mut folder_items: Vec<BrowseItem> = Vec::new();
    let mut dir_candidates: Vec<(PathBuf, u64)> = dir_sizes
        .into_iter()
        .filter(|(_, size)| *size >= min_folder_bytes)
        .collect();
    dir_candidates.sort_by(|a, b| b.1.cmp(&a.1));

    for (dir_path, size) in dir_candidates {
        if !dir_path.is_dir() {
            continue;
        }
        if protected_paths::is_protected_path(&dir_path) {
            continue;
        }
        if is_redundant_folder(&dir_path, &folder_items) {
            continue;
        }
        folder_items.push(build_folder_item(&dir_path, size, min_folder_mb, app_index));
    }

    folder_items.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.path.cmp(&b.path))
    });
    folder_items.truncate(MAX_RESULTS);
    folder_items
}

fn scan_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            roots.push(PathBuf::from(home));
        }
    }

    let applications = PathBuf::from("/Applications");
    if applications.is_dir() {
        roots.push(applications);
    }

    let volumes = PathBuf::from("/Volumes");
    if let Ok(entries) = std::fs::read_dir(volumes) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if name.starts_with('.') || name == "Macintosh HD" {
                continue;
            }
            roots.push(path);
        }
    }

    roots
}

fn is_under_roots(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn is_redundant_folder(path: &Path, existing: &[BrowseItem]) -> bool {
    existing.iter().any(|item| {
        item.is_directory && path.starts_with(Path::new(&item.path)) && path != Path::new(&item.path)
    })
}

fn build_file_item(path: &Path, size: u64, min_mb: u64, app_index: &AppIndex) -> BrowseItem {
    let path_str = path.to_string_lossy().to_string();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.clone());
    let protected = protected_paths::is_protected_path(path);
    let app = resolve_item_app(path, protected, app_index);

    BrowseItem {
        id: stable_id(&path_str),
        path: path_str,
        name,
        is_directory: false,
        size_bytes: size,
        size_ready: true,
        size_capped: false,
        modified: modified_str(path),
        risk: if protected { "high".into() } else { "medium".into() },
        risk_label: app.app_name.clone(),
        description: if protected {
            "系统保护路径内的文件，不可删除".into()
        } else {
            format!(
                "{} · 文件大小 ≥ {}，删除前请自行确认",
                app.app_name,
                format_threshold_mb(min_mb)
            )
        },
        junk_bytes: 0,
        junk_count: 0,
        child_count: 0,
        protected,
        app_label: Some(app.app_label),
        bundle_id: app.bundle_id,
        app_installed: Some(app.installed),
    }
}

fn build_folder_item(path: &Path, size: u64, min_mb: u64, app_index: &AppIndex) -> BrowseItem {
    let path_str = path.to_string_lossy().to_string();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.clone());
    let protected = protected_paths::is_protected_path(path);
    let child_count = std::fs::read_dir(path)
        .map(|it| it.flatten().count() as u32)
        .unwrap_or(0);
    let app = resolve_item_app(path, protected, app_index);

    BrowseItem {
        id: stable_id(&path_str),
        path: path_str,
        name,
        is_directory: true,
        size_bytes: size,
        size_ready: true,
        size_capped: false,
        modified: modified_str(path),
        risk: if protected { "high".into() } else { "medium".into() },
        risk_label: app.app_name.clone(),
        description: if protected {
            "文件夹总大小超过阈值，系统保护路径不可删除".into()
        } else {
            format!(
                "{} · 文件夹总大小 ≥ {}，删除前请自行确认",
                app.app_name,
                format_threshold_mb(min_mb)
            )
        },
        junk_bytes: 0,
        junk_count: 0,
        child_count,
        protected,
        app_label: Some(app.app_label),
        bundle_id: app.bundle_id,
        app_installed: Some(app.installed),
    }
}

fn resolve_item_app(
    path: &Path,
    protected: bool,
    app_index: &AppIndex,
) -> crate::app_resolver::AppResolveResult {
    if protected {
        return crate::app_resolver::AppResolveResult {
            app_name: "系统".into(),
            app_label: "系统".into(),
            bundle_id: None,
            installed: true,
        };
    }
    app_index.resolve_for_path(path)
}

fn modified_str(path: &Path) -> String {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| DateTime::<Local>::from(t).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".into())
}

fn stable_id(path: &str) -> String {
    format!("large_{}", path.replace('/', "_"))
}

fn format_threshold_mb(mb: u64) -> String {
    if mb >= 1024 && mb % 1024 == 0 {
        format!("{} GB", mb / 1024)
    } else if mb >= 1024 {
        format!("{:.1} GB", mb as f64 / 1024.0)
    } else {
        format!("{mb} MB")
    }
}
