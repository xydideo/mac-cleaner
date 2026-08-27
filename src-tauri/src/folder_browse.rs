use chrono::{DateTime, Local};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::protected_paths;
use crate::scan_control;

const LARGE_FILE_BYTES: u64 = 100 * 1024 * 1024;
const STALE_DAYS: i64 = 90;
const MAX_ENTRIES: usize = 600;
/// 深扫单个文件夹时最多统计的文件数，避免 /Users/xxx 这类目录卡死
const MAX_ANALYZE_FILES: u32 = 200_000;
/// 深扫体积上限：超过 5GB 立即停止，前端显示 >5G
const MAX_ANALYZE_BYTES: u64 = 5 * 1024 * 1024 * 1024;

static BUILD_DIR_NAMES: &[&str] = &[
    "node_modules",
    "dist",
    "build",
    "target",
    ".next",
    "DerivedData",
    ".gradle",
    "__pycache__",
    ".turbo",
    "coverage",
];

static INSTALLER_EXTS: &[&str] = &["dmg", "pkg", "iso", "zip", "rar", "7z", "apk", "ipa"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseItem {
    pub id: String,
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    /// 文件恒为 true；文件夹在快速列表阶段为 false，后台分析完成后为 true
    pub size_ready: bool,
    /// 深扫因超过 5GB 提前终止时为 true，体积展示为 >5G
    pub size_capped: bool,
    pub modified: String,
    pub risk: String,
    pub risk_label: String,
    pub description: String,
    pub junk_bytes: u64,
    pub junk_count: u32,
    pub child_count: u32,
    /// 系统保护路径，不可删除
    pub protected: bool,
    /// 所属应用展示标签，如「Chrome 应用」
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_installed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderBrowseSummary {
    pub path: String,
    pub total_bytes: u64,
    pub folder_count: u32,
    pub file_count: u32,
    pub junk_bytes: u64,
    pub junk_count: u32,
    pub pending_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderShortcut {
    pub id: String,
    pub label: String,
    pub path: String,
    pub description: String,
}

/// 自定义目录空状态快捷入口
pub fn list_shortcuts() -> Vec<FolderShortcut> {
    let home = match std::env::var("HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => return Vec::new(),
    };

    let username = home
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "用户".into());

    let mut shortcuts = Vec::new();

    let push_dir = |shortcuts: &mut Vec<FolderShortcut>, id: &str, label: &str, path: PathBuf, desc: &str| {
        if path.is_dir() {
            shortcuts.push(FolderShortcut {
                id: id.into(),
                label: label.into(),
                path: path.to_string_lossy().to_string(),
                description: desc.into(),
            });
        }
    };

    push_dir(
        &mut shortcuts,
        "home",
        &username,
        home.clone(),
        "用户主目录",
    );
    push_dir(
        &mut shortcuts,
        "downloads",
        "下载",
        home.join("Downloads"),
        "安装包、压缩文件与大文件",
    );
    push_dir(
        &mut shortcuts,
        "documents",
        "文稿",
        home.join("Documents"),
        "文档、归档与长期未动文件",
    );
    push_dir(
        &mut shortcuts,
        "library",
        "资源库",
        home.join("Library"),
        "缓存、日志与应用数据",
    );

    shortcuts
}

struct FileRisk {
    risk: String,
    label: String,
    description: String,
}

struct DirAnalysis {
    total_bytes: u64,
    junk_bytes: u64,
    junk_count: u32,
    risk: String,
    risk_label: String,
    description: String,
    size_capped: bool,
    files_capped: bool,
}

/// 快速列出当前层：不递归子树，毫秒级返回
pub fn list_directory(path: &str, show_hidden: bool) -> Result<(Vec<BrowseItem>, FolderBrowseSummary), String> {
    let input = Path::new(path.trim());
    if !input.exists() {
        return Err(format!("路径不存在: {path}"));
    }
    if !input.is_dir() {
        return Err("请选择一个文件夹".into());
    }

    let root = input
        .canonicalize()
        .map_err(|e| format!("无法解析路径: {path} ({e})"))?;

    let Ok(entries) = std::fs::read_dir(&root) else {
        return Err(format!("无法读取目录: {path}"));
    };

    let mut raw: Vec<(PathBuf, String, bool)> = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if !show_hidden && name.starts_with('.') && name != ".DS_Store" {
            continue;
        }
        raw.push((path, name, file_type.is_dir()));
    }

    if raw.len() > MAX_ENTRIES {
        raw.truncate(MAX_ENTRIES);
    }

    let items: Vec<BrowseItem> = raw
        .par_iter()
        .filter_map(|(path, name, is_dir)| build_browse_item_fast(path, name, *is_dir))
        .collect();

    Ok((items.clone(), build_summary(&root, &items)))
}

/// 后台深扫：仅对指定文件夹路径递归统计大小与风险
pub fn analyze_items(paths: &[String]) -> Vec<BrowseItem> {
    analyze_items_with_mode(paths, false)
}

/// 精确统计：不受 5GB / 20 万文件上限
pub fn analyze_items_exact(paths: &[String]) -> Vec<BrowseItem> {
    analyze_items_with_mode(paths, true)
}

fn analyze_items_with_mode(paths: &[String], exact: bool) -> Vec<BrowseItem> {
    paths
        .par_iter()
        .filter_map(|path_str| {
            let path = Path::new(path_str);
            if !path.is_dir() {
                return None;
            }
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            build_browse_item_analyzed(path, &name, exact)
        })
        .collect()
}

fn build_summary(root: &Path, items: &[BrowseItem]) -> FolderBrowseSummary {
    FolderBrowseSummary {
        path: root.to_string_lossy().to_string(),
        total_bytes: items
            .iter()
            .filter(|i| i.size_ready)
            .map(|i| i.size_bytes)
            .sum(),
        folder_count: items.iter().filter(|i| i.is_directory).count() as u32,
        file_count: items.iter().filter(|i| !i.is_directory).count() as u32,
        junk_bytes: items
            .iter()
            .filter(|i| i.size_ready)
            .map(|i| i.junk_bytes)
            .sum(),
        junk_count: items
            .iter()
            .filter(|i| i.size_ready)
            .map(|i| i.junk_count)
            .sum(),
        pending_count: items
            .iter()
            .filter(|i| i.is_directory && !i.size_ready)
            .count() as u32,
    }
}

fn build_browse_item_fast(path: &Path, name: &str, is_dir: bool) -> Option<BrowseItem> {
    let path_str = path.to_string_lossy().to_string();
    let modified = modified_str(path);

    if is_dir {
        let child_count = count_immediate_children(path);

        if crate::protected_paths::is_home_system_directory(path) {
            return Some(build_system_home_item(
                &path_str,
                name,
                &modified,
                child_count,
            ));
        }

        if is_empty_dir(path) {
            return Some(BrowseItem {
                id: stable_id(&path_str),
                path: path_str,
                name: name.to_string(),
                is_directory: true,
                size_bytes: 0,
                size_ready: true,
                size_capped: false,
                modified,
                risk: "low".into(),
                risk_label: "空文件夹".into(),
                description: "空目录".into(),
                junk_bytes: 0,
                junk_count: 0,
                child_count,
                protected: false,
                app_label: None,
                bundle_id: None,
                app_installed: None,
            });
        }

        if is_build_dir_name(name) {
            return Some(BrowseItem {
                id: stable_id(&path_str),
                path: path_str,
                name: name.to_string(),
                is_directory: true,
                size_bytes: 0,
                size_ready: false,
                size_capped: false,
                modified,
                risk: "low".into(),
                risk_label: "构建目录".into(),
                description: "常见构建/缓存目录，正在计算大小…".into(),
                junk_bytes: 0,
                junk_count: 0,
                child_count,
                protected: false,
                app_label: None,
                bundle_id: None,
                app_installed: None,
            });
        }

        Some(BrowseItem {
            id: stable_id(&path_str),
            path: path_str,
            name: name.to_string(),
            is_directory: true,
            size_bytes: 0,
            size_ready: false,
            size_capped: false,
            modified,
            risk: "medium".into(),
            risk_label: "待分析".into(),
            description: if child_count > 0 {
                format!("含 {child_count} 项，正在计算大小与风险…")
            } else {
                "正在计算大小与风险…".into()
            },
            junk_bytes: 0,
            junk_count: 0,
            child_count,
            protected: false,
            app_label: None,
            bundle_id: None,
            app_installed: None,
        })
    } else {
        let meta = std::fs::symlink_metadata(path).ok()?;
        if meta.is_symlink() {
            return None;
        }
        let size = meta.len();
        let file_risk = classify_file(path, name, size);
        let junk_bytes = if file_risk.risk == "low" { size } else { 0 };
        let junk_count = if file_risk.risk == "low" { 1 } else { 0 };
        Some(BrowseItem {
            id: stable_id(&path_str),
            path: path_str,
            name: name.to_string(),
            is_directory: false,
            size_bytes: size,
            size_ready: true,
            size_capped: false,
            modified,
            risk: file_risk.risk,
            risk_label: file_risk.label,
            description: file_risk.description,
            junk_bytes,
            junk_count,
            child_count: 0,
            protected: false,
            app_label: None,
            bundle_id: None,
            app_installed: None,
        })
    }
}

fn build_system_home_item(
    path_str: &str,
    name: &str,
    modified: &str,
    child_count: u32,
) -> BrowseItem {
    BrowseItem {
        id: stable_id(path_str),
        path: path_str.to_string(),
        name: name.to_string(),
        is_directory: true,
        size_bytes: 0,
        size_ready: true,
        size_capped: false,
        modified: modified.to_string(),
        risk: "high".into(),
        risk_label: "系统目录".into(),
        description: "macOS 用户主目录下的系统自带文件夹，不可删除".into(),
        junk_bytes: 0,
        junk_count: 0,
        child_count,
        protected: true,
        app_label: None,
        bundle_id: None,
        app_installed: None,
    }
}

fn build_browse_item_analyzed(path: &Path, name: &str, exact: bool) -> Option<BrowseItem> {
    let path_str = path.to_string_lossy().to_string();
    let modified = modified_str(path);
    let child_count = count_immediate_children(path);

    if crate::protected_paths::is_home_system_directory(path) {
        return Some(build_system_home_item(
            &path_str,
            name,
            &modified,
            child_count,
        ));
    }

    let analysis = analyze_directory(path, name, exact);

    Some(BrowseItem {
        id: stable_id(&path_str),
        path: path_str,
        name: name.to_string(),
        is_directory: true,
        size_bytes: analysis.total_bytes,
        size_ready: true,
        size_capped: analysis.size_capped,
        modified,
        risk: analysis.risk,
        risk_label: analysis.risk_label,
        description: analysis.description,
        junk_bytes: analysis.junk_bytes,
        junk_count: analysis.junk_count,
        child_count,
        protected: false,
        app_label: None,
        bundle_id: None,
        app_installed: None,
    })
}

fn analyze_directory(path: &Path, name: &str, exact: bool) -> DirAnalysis {
    if is_build_dir_name(name) {
        let (size, size_capped) = if exact {
            (dir_size_uncapped(path), false)
        } else {
            dir_size_no_follow(path)
        };
        let size_hint = if !exact && size_capped {
            "，已超过 5GB 停止扫描（显示 >5G）"
        } else {
            ""
        };
        return DirAnalysis {
            total_bytes: if size_capped { MAX_ANALYZE_BYTES } else { size },
            junk_bytes: if size_capped { 0 } else { size },
            junk_count: if size > 0 && !size_capped { 1 } else { 0 },
            risk: "low".into(),
            risk_label: "构建目录".into(),
            description: format!("常见构建/缓存目录，删除前请自行确认{size_hint}"),
            size_capped,
            files_capped: false,
        };
    }

    if is_empty_dir(path) {
        return DirAnalysis {
            total_bytes: 0,
            junk_bytes: 0,
            junk_count: 0,
            risk: "low".into(),
            risk_label: "空文件夹".into(),
            description: "空目录".into(),
            size_capped: false,
            files_capped: false,
        };
    }

    let mut total_bytes = 0u64;
    let mut file_count = 0u32;
    let mut junk_bytes = 0u64;
    let mut junk_count = 0u32;
    let mut has_medium = false;
    let mut has_high = false;
    let mut size_capped = false;
    let mut files_capped = false;

    for entry in WalkDir::new(path).follow_links(false).into_iter().flatten() {
        if scan_control::checkpoint(file_count as u64) {
            break;
        }
        if entry.file_type().is_symlink() {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        file_count += 1;
        if !exact && file_count > MAX_ANALYZE_FILES {
            files_capped = true;
            break;
        }

        let file_path = entry.path();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        total_bytes = total_bytes.saturating_add(size);

        if !exact && total_bytes > MAX_ANALYZE_BYTES {
            size_capped = true;
            total_bytes = MAX_ANALYZE_BYTES;
            break;
        }

        let fname = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let risk = classify_file(file_path, &fname, size);

        match risk.risk.as_str() {
            "low" => {
                junk_bytes += size;
                junk_count += 1;
            }
            "medium" => has_medium = true,
            "high" => has_high = true,
            _ => has_medium = true,
        }
    }

    if file_count == 0 {
        return DirAnalysis {
            total_bytes: 0,
            junk_bytes: 0,
            junk_count: 0,
            risk: "low".into(),
            risk_label: "空文件夹".into(),
            description: "无文件的目录".into(),
            size_capped: false,
            files_capped: false,
        };
    }

    if !exact && size_capped {
        return DirAnalysis {
            total_bytes: MAX_ANALYZE_BYTES,
            junk_bytes: 0,
            junk_count: 0,
            risk: "high".into(),
            risk_label: "超大目录".into(),
            description: format!(
                "已超过 5GB（已扫描 {file_count} 个文件），停止继续扫描，可点「具体」计算真实大小"
            ),
            size_capped: true,
            files_capped,
        };
    }

    let cap_hint = if !exact && files_capped {
        "（已统计 20万个文件，目录可能更大）".into()
    } else {
        String::new()
    };

    let (risk, risk_label, description) = if has_high {
        (
            "high",
            "混合内容",
            format!("内含 {file_count} 个文件，含未匹配常见可清理规则的文件，不建议整夹删除{cap_hint}"),
        )
    } else if has_medium {
        (
            "medium",
            "混合内容",
            format!(
                "内含 {file_count} 个文件，其中 {junk_count} 个匹配常见可清理规则（{:.1} MB）{cap_hint}",
                (junk_bytes as f64 / 1024.0 / 1024.0).max(0.01)
            ),
        )
    } else if junk_count == file_count {
        (
            "low",
            "目录",
            format!("内含 {file_count} 个文件，删除前请自行确认{cap_hint}"),
        )
    } else {
        (
            "medium",
            "混合内容",
            format!("内含 {file_count} 个文件，请进入查看详情{cap_hint}"),
        )
    };

    DirAnalysis {
        total_bytes,
        junk_bytes,
        junk_count,
        risk: risk.into(),
        risk_label: risk_label.into(),
        description,
        size_capped: false,
        files_capped,
    }
}

fn classify_file(path: &Path, name: &str, size: u64) -> FileRisk {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if name == ".DS_Store" {
        return FileRisk {
            risk: "low".into(),
            label: "系统元数据".into(),
            description: "macOS 目录元数据（.DS_Store）".into(),
        };
    }

    if INSTALLER_EXTS.contains(&ext.as_str()) {
        return FileRisk {
            risk: "medium".into(),
            label: "安装包".into(),
            description: "可能是安装包或压缩文件，删除前请自行确认".into(),
        };
    }

    if size >= LARGE_FILE_BYTES {
        return FileRisk {
            risk: "medium".into(),
            label: "大文件".into(),
            description: "体积超过 100MB，删除前请自行确认".into(),
        };
    }

    if is_stale(path) {
        return FileRisk {
            risk: "low".into(),
            label: "长期未修改".into(),
            description: "超过 90 天未修改".into(),
        };
    }

    FileRisk {
        risk: "high".into(),
        label: "普通文件".into(),
        description: "可能是正常使用的文件，删除前请自行确认".into(),
    }
}

fn is_build_dir_name(name: &str) -> bool {
    BUILD_DIR_NAMES
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
}

fn dir_size_uncapped(path: &Path) -> u64 {
    let mut total = 0u64;
    let mut index = 0u64;
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if scan_control::checkpoint(index) {
            break;
        }
        index += 1;
        total = total.saturating_add(entry.metadata().ok().map(|m| m.len()).unwrap_or(0));
    }
    total
}

fn dir_size_no_follow(path: &Path) -> (u64, bool) {
    let mut total = 0u64;
    let mut index = 0u64;
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if scan_control::checkpoint(index) {
            break;
        }
        index += 1;
        let size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
        total = total.saturating_add(size);
        if total > MAX_ANALYZE_BYTES {
            return (MAX_ANALYZE_BYTES, true);
        }
    }
    (total, false)
}

fn is_empty_dir(path: &Path) -> bool {
    std::fs::read_dir(path)
        .map(|mut it| it.next().is_none())
        .unwrap_or(false)
}

fn count_immediate_children(path: &Path) -> u32 {
    std::fs::read_dir(path)
        .map(|it| it.flatten().count() as u32)
        .unwrap_or(0)
}

fn is_stale(path: &Path) -> bool {
    let Ok(meta) = path.metadata() else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let modified: DateTime<Local> = modified.into();
    (Local::now() - modified).num_days() >= STALE_DAYS
}

fn modified_str(path: &Path) -> String {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| DateTime::<Local>::from(t).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".into())
}

fn stable_id(path: &str) -> String {
    format!("browse_{}", path.replace('/', "_"))
}
