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

fn user_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
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

pub fn is_protected_path(path: &Path) -> bool {
    if is_home_system_directory(path) {
        return true;
    }

    let Ok(canonical) = path.canonicalize() else {
        return false;
    };

    let protected_prefixes: &[&str] = &[
        "/System",
        "/usr",
        "/bin",
        "/sbin",
        "/var/vm",
        "/private/var/vm",
        "/cores",
    ];

    let path_str = canonical.to_string_lossy();
    for prefix in protected_prefixes {
        if path_str == *prefix || path_str.starts_with(&format!("{prefix}/")) {
            return true;
        }
    }

    if path_str.starts_with("/Applications/")
        && path_str.ends_with(".app")
        && path_str.contains("/Safari.app")
    {
        return true;
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
        return Some("系统保护路径，无法删除".into());
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
