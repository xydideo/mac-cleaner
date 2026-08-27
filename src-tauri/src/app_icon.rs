use base64::{engine::general_purpose::STANDARD, Engine as _};
use plist::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 读取 .app 图标并转为 data URL（PNG base64）
pub fn load_app_icon(app_path: &Path) -> Option<String> {
    let icns_path = resolve_icns_path(app_path)?;
    png_base64_from_icns(&icns_path)
}

fn resolve_icns_path(app_path: &Path) -> Option<PathBuf> {
    let resources = app_path.join("Contents/Resources");
    if !resources.exists() {
        return None;
    }

    let plist_path = app_path.join("Contents/Info.plist");
    if let Ok(plist_value) = plist::from_file::<_, Value>(&plist_path) {
        if let Some(dict) = plist_value.as_dictionary() {
            if let Some(icon_name) = dict.get("CFBundleIconFile").and_then(|v| v.as_string()) {
                let candidates = if icon_name.ends_with(".icns") {
                    vec![resources.join(&icon_name)]
                } else {
                    vec![
                        resources.join(format!("{icon_name}.icns")),
                        resources.join(&icon_name),
                    ]
                };
                for path in candidates {
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    // 兜底：Resources 下第一个 .icns
    if let Ok(entries) = std::fs::read_dir(&resources) {
        let mut icns_files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("icns"))
            .collect();
        icns_files.sort();
        return icns_files.into_iter().next();
    }

    None
}

fn png_base64_from_icns(icns_path: &Path) -> Option<String> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    let out = std::env::temp_dir().join(format!("mac-cleaner-icon-{ts}.png"));

    let status = Command::new("sips")
        .args([
            "-s",
            "format",
            "png",
            "-z",
            "64",
            "64",
            icns_path.to_str().unwrap_or(""),
            "--out",
        ])
        .arg(out.to_str().unwrap_or(""))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;

    if !status.success() || !out.exists() {
        let _ = std::fs::remove_file(&out);
        return None;
    }

    let bytes = std::fs::read(&out).ok()?;
    let _ = std::fs::remove_file(&out);

    if bytes.is_empty() {
        return None;
    }

    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}
