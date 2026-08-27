use std::sync::Mutex;

#[cfg(target_os = "macos")]
use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
use objc2_app_kit::NSWindow;
#[cfg(target_os = "macos")]
use objc2_foundation::{NSPoint, NSRect, NSSize};

#[derive(Clone, Copy, Debug)]
struct SavedFrame {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[cfg(target_os = "macos")]
static SAVED_FRAME: Mutex<Option<SavedFrame>> = Mutex::new(None);

#[cfg(target_os = "macos")]
fn rect_from_saved(saved: SavedFrame) -> NSRect {
    NSRect::new(
        NSPoint::new(saved.x, saved.y),
        NSSize::new(saved.width, saved.height),
    )
}

/// 双击标题栏：仅拉高至当前屏幕可用高度，宽度与水平位置不变。
#[cfg(target_os = "macos")]
pub fn toggle_height_maximized(window: &WebviewWindow) -> Result<bool, String> {
    let ns_window_ptr = window
        .ns_window()
        .map_err(|e| format!("无法获取 NSWindow: {e}"))?;
    let ns_window = unsafe { &*(ns_window_ptr as *mut NSWindow) };

    let screen = ns_window
        .screen()
        .ok_or_else(|| "无法获取当前屏幕".to_string())?;
    let visible = screen.visibleFrame();
    let frame = ns_window.frame();

    let mut saved = SAVED_FRAME
        .lock()
        .map_err(|e| format!("窗口状态锁失败: {e}"))?;

    if let Some(restore) = saved.take() {
        ns_window.setFrame_display_animate(rect_from_saved(restore), true, true);
        return Ok(false);
    }

    *saved = Some(SavedFrame {
        x: frame.origin.x,
        y: frame.origin.y,
        width: frame.size.width,
        height: frame.size.height,
    });

    let mut next = frame;
    next.size.height = visible.size.height;
    next.origin.y = visible.origin.y;

    ns_window.setFrame_display_animate(next, true, true);
    Ok(true)
}

#[cfg(target_os = "macos")]
pub fn reset_saved_frame() {
  if let Ok(mut saved) = SAVED_FRAME.lock() {
    *saved = None;
  }
}

#[cfg(not(target_os = "macos"))]
pub fn toggle_height_maximized(_window: &tauri::WebviewWindow) -> Result<bool, String> {
    Err("当前平台不支持高度最大化".into())
}

#[cfg(not(target_os = "macos"))]
pub fn reset_saved_frame() {}
