use crate::disk;
use crate::scan_control;
use plist::Value;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub bundle_id: String,
    pub version: String,
    pub size_bytes: u64,
    pub is_system_app: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRelatedItem {
    pub id: String,
    pub path: String,
    pub size_bytes: u64,
    pub category: String,
    pub risk: String,
    pub title: String,
    pub description: String,
    pub match_type: String,
}

pub fn list_installed_apps() -> Result<Vec<InstalledApp>, String> {
    let home = dirs_home();
    let app_dirs = [
        (PathBuf::from("/Applications"), false),
        (home.join("Applications"), false),
        (PathBuf::from("/System/Applications"), true),
    ];

    let mut apps: Vec<InstalledApp> = Vec::new();
    let mut seen_bundle_ids = std::collections::HashSet::new();

    for (dir, force_system) in app_dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("app") {
                continue;
            }
            let Some(details) = read_app_details(&path) else {
                continue;
            };
            if seen_bundle_ids.contains(&details.bundle_id) {
                continue;
            }
            seen_bundle_ids.insert(details.bundle_id.clone());

            let path_str = path.to_string_lossy().to_string();
            let is_system = force_system || details.bundle_id.starts_with("com.apple.");

            apps.push(InstalledApp {
                name: details.display_name,
                path: path_str.clone(),
                bundle_id: details.bundle_id,
                version: details.version,
                size_bytes: disk::path_size(&path_str).unwrap_or(0),
                is_system_app: is_system,
                icon_base64: crate::app_icon::load_app_icon(&path),
            });
        }
    }

    let mut user_apps: Vec<InstalledApp> = apps
        .iter()
        .filter(|a| !a.is_system_app)
        .cloned()
        .collect();
    let mut system_apps: Vec<InstalledApp> = apps
        .iter()
        .filter(|a| a.is_system_app)
        .cloned()
        .collect();

    user_apps.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then_with(|| a.name.cmp(&b.name)));
    system_apps.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then_with(|| a.name.cmp(&b.name)));

    user_apps.extend(system_apps);
    Ok(user_apps)
}

pub fn scan_app_related(app_path: &str, bundle_id: &str, app_name: &str) -> Result<Vec<AppRelatedItem>, String> {
    let home = dirs_home();
    let mut items = Vec::new();

    let candidates: Vec<(&str, PathBuf, &str, &str, &str)> = vec![
        ("main", PathBuf::from(app_path), "主程序", "low", "应用程序本体"),
        (
            "cache",
            home.join(format!("Library/Caches/{bundle_id}")),
            "应用缓存",
            "low",
            "缓存文件，删除后会重建",
        ),
        (
            "cache",
            home.join(format!("Library/Caches/{app_name}")),
            "应用缓存",
            "low",
            "缓存文件，删除后会重建",
        ),
        (
            "support",
            home.join(format!("Library/Application Support/{app_name}")),
            "应用数据",
            "high",
            "可能含用户数据或配置",
        ),
        (
            "support",
            home.join(format!("Library/Application Support/{bundle_id}")),
            "应用数据",
            "high",
            "可能含用户数据或配置",
        ),
        (
            "container",
            home.join(format!("Library/Containers/{bundle_id}")),
            "沙盒数据",
            "high",
            "沙盒容器，可能含聊天记录或文档",
        ),
        (
            "preferences",
            home.join(format!("Library/Preferences/{bundle_id}.plist")),
            "偏好设置",
            "medium",
            "用户偏好，删除后需重新配置",
        ),
        (
            "saved_state",
            home.join(format!(
                "Library/Saved Application State/{bundle_id}.savedState"
            )),
            "窗口状态",
            "low",
            "窗口恢复数据",
        ),
        (
            "logs",
            home.join(format!("Library/Logs/{app_name}")),
            "运行日志",
            "low",
            "历史运行日志",
        ),
        (
            "webkit",
            home.join(format!("Library/WebKit/{bundle_id}")),
            "Web 数据",
            "medium",
            "WebKit 缓存与数据",
        ),
        (
            "http",
            home.join(format!("Library/HTTPStorages/{bundle_id}")),
            "HTTP 缓存",
            "low",
            "网络请求缓存",
        ),
        (
            "launch_agent",
            home.join(format!("Library/LaunchAgents/{bundle_id}.plist")),
            "启动项",
            "medium",
            "用户级后台启动项",
        ),
        (
            "cookies",
            home.join(format!("Library/Cookies/{bundle_id}.binarycookies")),
            "Cookie",
            "medium",
            "应用 Cookie 数据",
        ),
    ];

    let mut seen_paths = std::collections::HashSet::new();

    for (index, (category, path, title, risk, desc)) in candidates.iter().enumerate() {
        if scan_control::checkpoint(index as u64) {
            break;
        }
        let path_str = path.to_string_lossy().to_string();
        if !path.exists() || seen_paths.contains(&path_str) {
            continue;
        }
        seen_paths.insert(path_str.clone());

        let size = disk::path_size(&path_str).unwrap_or(0);
        if size == 0 && *category != "main" {
            continue;
        }

        items.push(AppRelatedItem {
            id: format!("app_rel_{}", path_str.replace('/', "_")),
            path: path_str,
            size_bytes: size,
            category: (*category).into(),
            risk: (*risk).into(),
            title: (*title).into(),
            description: (*desc).into(),
            match_type: "bundle_id".into(),
        });
    }

    items.sort_by(|a, b| {
        category_order(&a.category)
            .cmp(&category_order(&b.category))
            .then(b.size_bytes.cmp(&a.size_bytes))
    });

    Ok(items)
}

fn category_order(category: &str) -> u8 {
    match category {
        "main" => 0,
        "cache" => 1,
        "logs" => 2,
        "preferences" => 3,
        "saved_state" => 4,
        "launch_agent" => 5,
        "support" => 6,
        "container" => 7,
        _ => 8,
    }
}

struct AppDetails {
    bundle_id: String,
    display_name: String,
    version: String,
}

fn read_app_details(app_path: &Path) -> Option<AppDetails> {
    let plist_path = app_path.join("Contents/Info.plist");
    let plist_value: Value = plist::from_file(&plist_path).ok()?;
    let dict = plist_value.as_dictionary()?;

    let bundle_id = dict
        .get("CFBundleIdentifier")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())?;

    let display_name = dict
        .get("CFBundleDisplayName")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .or_else(|| {
            dict.get("CFBundleName")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            app_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })?;

    let version = dict
        .get("CFBundleShortVersionString")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "-".into());

    Some(AppDetails {
        bundle_id,
        display_name,
        version,
    })
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
}
