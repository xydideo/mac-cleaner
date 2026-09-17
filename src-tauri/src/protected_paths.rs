use std::path::{Path, PathBuf};

/// macOS 用户主目录下系统自带的顶层文件夹（整夹不可删）
static HOME_SYSTEM_DIR_NAMES: &[&str] = &[
    "Desktop",
    "Documents",
    "Downloads",
    "Library",
    "Movies",
    "Music",
    "Pictures",
    "Public",
    "Sites",
];

/// 系统卷路径：禁止删除；文件树遍历时剪枝（垃圾清理默认不会扫这些根目录）
static SYSTEM_VOLUME_PREFIXES: &[&str] = &[
    "/System",
    "/Library",
    "/usr",
    "/bin",
    "/sbin",
    "/etc",
    "/opt",
    "/dev",
    "/var/vm",
    "/private/var/vm",
    "/private/etc",
    "/private/var/db",
    "/private/var/folders",
    "/cores",
];

/// ~/Library 下与系统配置强相关、整目录不参与清理扫描且不可删
static CRITICAL_LIBRARY_DIRS: &[&str] = &[
    "Keychains",
    "SystemConfiguration",
    "SyncedPreferences",
];

/// 垃圾清理扫描时：这些 ~/Library 子目录下的 com.apple.* 等仍排除
static APPLE_SCOPED_LIBRARY_DIRS: &[&str] = &[
    "Preferences",
    "Caches",
    "Logs",
    "Application Support",
    "LaunchAgents",
    "LaunchDaemons",
    "WebKit",
    "HTTPStorages",
    "Saved Application State",
    "Cookies",
];

fn user_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Apple / macOS 系统 Bundle ID、目录名、plist 主文件名等
pub fn is_apple_system_identifier(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("com.apple.") || lower.starts_with("apple.") {
        return true;
    }
    if lower.starts_with("group.com.apple.") || lower.starts_with("group.apple.") {
        return true;
    }
    if lower.starts_with("org.apple.") {
        return true;
    }
    matches!(
        lower.as_str(),
        "globalpreferences"
            | ".globalpreferences"
            | "loginwindow"
            | "systempreferences"
            | "com.apple"
            | "icloud"
            | "mobileme"
    ) || lower.starts_with(".globalpreferences")
        || lower.starts_with("loginwindow")
        || lower.starts_with("systempreferences")
}

/// 系统偏好 plist（键盘、鼠标、辅助功能等），误删会导致重启后要重新设置
pub fn is_system_preference_plist_stem(stem: &str) -> bool {
    if is_apple_system_identifier(stem) {
        return true;
    }
    let lower = stem.to_lowercase();
    lower.contains("hitoolbox")
        || lower.contains("keyboard")
        || lower.contains("mouse")
        || lower.contains("trackpad")
        || lower.contains("bluetooth")
        || lower.contains("universalaccess")
        || lower.contains("symbolichotkeys")
        || lower.contains("inputsource")
        || lower.contains("inputmethod")
        || lower.contains("driver.apple")
        || lower.contains("multitouch")
        || lower.contains("pointing")
        || lower.contains("spelling")
        || lower.contains("locale")
        || lower.contains("timezone")
        || lower.contains("networkservices")
        || lower.contains("loginwindow")
        || lower.contains("dock")
        || lower.contains("finder")
        || lower.contains("controlcenter")
        || lower.contains("menuextras")
}

fn path_matches_prefix(path_str: &str, prefix: &str) -> bool {
    path_str == prefix || path_str.starts_with(&format!("{prefix}/"))
}

fn is_system_volume_path(path_str: &str) -> bool {
    SYSTEM_VOLUME_PREFIXES
        .iter()
        .any(|prefix| path_matches_prefix(path_str, prefix))
}

fn home_library_relative(path: &Path) -> Option<Vec<String>> {
    let home = user_home()?;
    let library = home.join("Library");
    let Ok(rel) = path.strip_prefix(&library) else {
        return None;
    };
    if rel.as_os_str().is_empty() {
        return Some(Vec::new());
    }
    Some(
        rel.components()
            .filter_map(|c| c.as_os_str().to_str().map(String::from))
            .collect(),
    )
}

fn is_critical_library_path(parts: &[String]) -> bool {
    if parts.is_empty() {
        return false;
    }
    CRITICAL_LIBRARY_DIRS
        .iter()
        .any(|name| parts[0].eq_ignore_ascii_case(name))
}

fn is_apple_scoped_library_path(parts: &[String]) -> bool {
    if parts.is_empty() {
        return false;
    }
    let top = &parts[0];
    if is_apple_system_identifier(top) {
        return true;
    }
    if !APPLE_SCOPED_LIBRARY_DIRS
        .iter()
        .any(|name| top.eq_ignore_ascii_case(name))
    {
        return false;
    }
    if parts.len() < 2 {
        return top.eq_ignore_ascii_case("Preferences");
    }
    let child = &parts[1];
    let stem = Path::new(child)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(child.as_str());
    is_apple_system_identifier(child)
        || is_apple_system_identifier(stem)
        || (top.eq_ignore_ascii_case("Preferences")
            && is_system_preference_plist_stem(stem))
}

/// 是否为 ~/Pictures、~/Public 等系统自带用户目录
pub fn is_home_system_directory(path: &Path) -> bool {
    let Ok(canonical) = path.canonicalize() else {
        return false;
    };

    let Some(home) = user_home() else {
        return false;
    };
    let Ok(home_canon) = home.canonicalize() else {
        return false;
    };

    if canonical.parent() != Some(home_canon.as_path()) {
        return false;
    }

    let Some(name) = canonical.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    HOME_SYSTEM_DIR_NAMES
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
        || name.eq_ignore_ascii_case("Picture")
}

/// 不可删除的路径（删除接口、清理兜底）
pub fn is_protected_path(path: &Path) -> bool {
    if is_home_system_directory(path) {
        return true;
    }

    let lossy = path.to_string_lossy();
    if is_system_volume_path(&lossy) {
        return true;
    }

    if let Ok(canonical) = path.canonicalize() {
        let canon = canonical.to_string_lossy();
        if is_system_volume_path(&canon) {
            return true;
        }
    }

    if let Some(parts) = home_library_relative(path) {
        if is_critical_library_path(&parts) {
            return true;
        }
        if is_apple_scoped_library_path(&parts) {
            return true;
        }
    }

    if lossy.starts_with("/Applications/")
        && lossy.ends_with(".app")
        && (lossy.contains("/Safari.app") || lossy.contains("/System/Applications/"))
    {
        return true;
    }

    false
}

/// 深度扫描「遗留配置」：仅允许已卸载的第三方偏好 plist
pub fn is_eligible_third_party_leftover_preference(stem: &str, app_installed: bool) -> bool {
    if app_installed {
        return false;
    }
    if is_system_preference_plist_stem(stem) {
        return false;
    }
    if is_apple_system_identifier(stem) {
        return false;
    }
    true
}

/// 垃圾清理扫描是否跳过（比删除保护略宽：含 com.apple 缓存/日志等）
pub fn is_excluded_from_junk_scan(path: &Path) -> bool {
    if is_protected_path(path) {
        return true;
    }

    if let Some(parts) = home_library_relative(path) {
        if parts.len() >= 2 {
            let top = &parts[0];
            let child = &parts[1];
            let stem = Path::new(child)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(child.as_str());
            if top.eq_ignore_ascii_case("Caches") || top.eq_ignore_ascii_case("Logs") {
                if is_apple_system_identifier(child) || is_apple_system_identifier(stem) {
                    return true;
                }
            }
        }
    }

    false
}

/// 文件树遍历时剪枝：系统卷 + 关键配置目录 + 废纸篓
pub fn should_prune_scan_entry(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if name == ".Trash" || name == ".Trashes" || name == " .Trashes" {
        return true;
    }

    let lossy = path.to_string_lossy();
    if is_system_volume_path(&lossy) {
        return true;
    }

    if let Some(parts) = home_library_relative(path) {
        if is_critical_library_path(&parts) {
            return true;
        }
    }

    false
}

pub fn protection_message(path: &Path) -> Option<String> {
    if is_home_system_directory(path) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "该文件夹".into());
        return Some(format!(
            "「{name}」为 macOS 用户主目录下的系统自带文件夹，不可删除"
        ));
    }

    if is_protected_path(path) {
        return Some("系统或受保护路径，无法删除".into());
    }

    None
}

pub fn validate_deletable(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }

    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("路径不存在或已被删除".into());
    }

    if let Some(msg) = protection_message(p) {
        return Err(msg);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn apple_identifier_detects_system_bundle_ids() {
        assert!(is_apple_system_identifier("com.apple.finder"));
        assert!(!is_apple_system_identifier("com.google.Chrome"));
    }

    #[test]
    fn system_preference_stems_include_input_devices() {
        assert!(is_system_preference_plist_stem("com.apple.HIToolbox"));
        assert!(is_system_preference_plist_stem(".GlobalPreferences"));
    }

    #[test]
    fn leftover_only_third_party_uninstalled() {
        assert!(!is_eligible_third_party_leftover_preference(
            "com.google.Chrome",
            true
        ));
        assert!(is_eligible_third_party_leftover_preference(
            "com.example.removed",
            false
        ));
        assert!(!is_eligible_third_party_leftover_preference(
            "com.apple.HIToolbox",
            false
        ));
    }

    #[test]
    fn third_party_cache_not_protected_by_default() {
        let home = user_home().expect("HOME");
        let cache = home.join("Library/Caches/com.google.Chrome");
        assert!(!is_excluded_from_junk_scan(&cache));
    }

    #[test]
    fn mail_dir_not_fully_protected_for_scan() {
        let home = user_home().expect("HOME");
        let mail = home.join("Library/Mail");
        assert!(!should_prune_scan_entry(&mail));
    }
}
