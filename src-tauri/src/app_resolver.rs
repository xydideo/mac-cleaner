use plist::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 已安装应用索引：Bundle ID → 显示名称
pub struct AppIndex {
    by_bundle_id: HashMap<String, String>,
    by_folder_name: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AppResolveResult {
    pub app_name: String,
    pub app_label: String,
    pub bundle_id: Option<String>,
    pub installed: bool,
}

impl AppIndex {
    pub fn build() -> Self {
        let mut by_bundle_id = HashMap::new();
        let mut by_folder_name = HashMap::new();

        let app_dirs = [
            PathBuf::from("/Applications"),
            dirs_home().join("Applications"),
            PathBuf::from("/System/Applications"),
        ];

        for dir in app_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("app") {
                        continue;
                    }
                    if let Some((bundle_id, display_name)) = read_app_info(&path) {
                        let folder_name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        by_bundle_id
                            .entry(bundle_id)
                            .or_insert_with(|| display_name.clone());
                        by_folder_name
                            .entry(folder_name.to_lowercase())
                            .or_insert(display_name);
                    }
                }
            }
        }

        Self {
            by_bundle_id,
            by_folder_name,
        }
    }

    /// 根据缓存目录名 / Bundle ID 解析应用信息
    pub fn resolve(&self, raw_name: &str) -> AppResolveResult {
        let trimmed = raw_name.trim();
        if trimmed.is_empty() {
            return unknown_result(raw_name, None);
        }

        // 1. 精确匹配 Bundle ID
        if let Some(name) = self.by_bundle_id.get(trimmed) {
            return AppResolveResult {
                app_name: name.clone(),
                app_label: format!("{name} 应用"),
                bundle_id: Some(trimmed.to_string()),
                installed: true,
            };
        }

        // 2. 目录名匹配（如 "Google Chrome" 缓存文件夹）
        if let Some(name) = self.by_folder_name.get(&trimmed.to_lowercase()) {
            return AppResolveResult {
                app_name: name.clone(),
                app_label: format!("{name} 应用"),
                bundle_id: if trimmed.contains('.') {
                    Some(trimmed.to_string())
                } else {
                    None
                },
                installed: true,
            };
        }

        // 3. Bundle ID 形态但未安装：从 ID 推断可读名称
        if trimmed.contains('.') {
            let inferred = infer_name_from_bundle_id(trimmed);
            return AppResolveResult {
                app_label: format!("{inferred} 应用"),
                app_name: inferred,
                bundle_id: Some(trimmed.to_string()),
                installed: false,
            };
        }

        // 4. 普通文件夹名
        let name = humanize_token(trimmed);
        AppResolveResult {
            app_label: format!("{name} 应用"),
            app_name: name,
            bundle_id: None,
            installed: false,
        }
    }

    /// 根据文件/文件夹路径推断所属应用
    pub fn resolve_for_path(&self, path: &Path) -> AppResolveResult {
        if let Some(name) = extract_app_bundle_name(path) {
            return self.resolve(&name);
        }

        for subdir in [
            "Containers",
            "Group Containers",
            "Caches",
            "Application Support",
            "WebKit",
            "HTTPStorages",
            "Logs",
        ] {
            if let Some(token) = extract_after_library(path, subdir) {
                return self.resolve(sanitize_app_token(&token));
            }
        }

        if let Some(token) = extract_after_library(path, "Saved Application State") {
            return self.resolve(sanitize_app_token(&token));
        }

        if is_preferences_path(path) {
            if let Some(file_name) = path.file_stem().and_then(|n| n.to_str()) {
                return self.resolve(file_name);
            }
        }

        if is_cookies_path(path) {
            if let Some(file_name) = path.file_stem().and_then(|n| n.to_str()) {
                return self.resolve(file_name);
            }
        }

        let path_str = path.to_string_lossy();
        if path_str == "/System" || path_str.starts_with("/System/") {
            return system_result();
        }

        if path_str.starts_with("/Volumes/") {
            return volume_result();
        }

        if let Some(home) = user_home() {
            if path.starts_with(&home) {
                if let Ok(rel) = path.strip_prefix(&home) {
                    let mut components = rel.components();
                    if let Some(first) = components.next().and_then(|c| c.as_os_str().to_str()) {
                        if first != "Library" {
                            let label = home_folder_label(first);
                            return AppResolveResult {
                                app_name: label.clone(),
                                app_label: label,
                                bundle_id: None,
                                installed: true,
                            };
                        }
                    }
                }
            }
        }

        AppResolveResult {
            app_name: "用户文件".into(),
            app_label: "用户文件".into(),
            bundle_id: None,
            installed: true,
        }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
}

fn user_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

fn extract_app_bundle_name(path: &Path) -> Option<String> {
    for ancestor in path.ancestors() {
        if ancestor.extension().and_then(|e| e.to_str()) == Some("app") {
            return ancestor
                .file_stem()
                .map(|s| s.to_string_lossy().to_string());
        }
    }
    None
}

fn extract_after_library(path: &Path, subdir: &str) -> Option<String> {
    let components: Vec<_> = path.components().collect();
    for (index, component) in components.iter().enumerate() {
        if component.as_os_str() != "Library" {
            continue;
        }
        let next = components.get(index + 1)?;
        if next.as_os_str().to_str()? != subdir {
            continue;
        }
        return components
            .get(index + 2)
            .and_then(|c| c.as_os_str().to_str())
            .map(|s| s.to_string());
    }
    None
}

fn sanitize_app_token(token: &str) -> &str {
    token
        .trim_end_matches(".savedState")
        .trim_end_matches(".plist")
        .trim_end_matches(".binarycookies")
}

fn is_preferences_path(path: &Path) -> bool {
    path.components()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| w[0].as_os_str() == "Library" && w[1].as_os_str() == "Preferences")
}

fn is_cookies_path(path: &Path) -> bool {
    path.components()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| w[0].as_os_str() == "Library" && w[1].as_os_str() == "Cookies")
}

fn system_result() -> AppResolveResult {
    AppResolveResult {
        app_name: "系统".into(),
        app_label: "系统".into(),
        bundle_id: None,
        installed: true,
    }
}

fn volume_result() -> AppResolveResult {
    AppResolveResult {
        app_name: "外部磁盘".into(),
        app_label: "外部磁盘".into(),
        bundle_id: None,
        installed: true,
    }
}

fn home_folder_label(name: &str) -> String {
    match name {
        "Desktop" => "桌面".into(),
        "Documents" => "文稿".into(),
        "Downloads" => "下载".into(),
        "Movies" => "影片".into(),
        "Music" => "音乐".into(),
        "Pictures" => "图片".into(),
        "Public" => "公共".into(),
        _ => humanize_token(name),
    }
}

fn read_app_info(app_path: &Path) -> Option<(String, String)> {
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

    Some((bundle_id, display_name))
}

fn infer_name_from_bundle_id(bundle_id: &str) -> String {
    let skip = [
        "com", "org", "net", "io", "cn", "app", "dev", "www", "mac", "osx", "helper", "agent",
        "desktop", "service", "plugin",
    ];
    let parts: Vec<&str> = bundle_id.split('.').collect();

    for part in parts.iter().rev() {
        let lower = part.to_lowercase();
        if skip.contains(&lower.as_str()) || part.len() < 2 {
            continue;
        }
        if part.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        return humanize_token(part);
    }

    humanize_token(bundle_id)
}

fn humanize_token(raw: &str) -> String {
    let mut s = raw.replace(['_', '-'], " ");
    if s.chars().all(|c| c.is_ascii_uppercase() || !c.is_alphabetic()) {
        s = s.to_ascii_lowercase();
    }
    let mut chars = s.chars();
    match chars.next() {
        None => raw.to_string(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

fn unknown_result(raw_name: &str, bundle_id: Option<String>) -> AppResolveResult {
    let name = humanize_token(raw_name);
    AppResolveResult {
        app_label: format!("{name} 应用"),
        app_name: name,
        bundle_id,
        installed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn infer_trae_bundle_id() {
        let name = infer_name_from_bundle_id("com.trae.app");
        assert_eq!(name, "Trae");
    }

    #[test]
    fn infer_chrome_bundle_id() {
        let name = infer_name_from_bundle_id("com.google.Chrome");
        assert_eq!(name, "Chrome");
    }

    #[test]
    fn resolve_cache_path() {
        let index = AppIndex { by_bundle_id: HashMap::new(), by_folder_name: HashMap::new() };
        let path = PathBuf::from("/Users/test/Library/Caches/com.google.Chrome/Default");
        let resolved = index.resolve_for_path(&path);
        assert_eq!(resolved.app_name, "Chrome");
        assert!(resolved.app_label.contains("Chrome"));
    }
}
