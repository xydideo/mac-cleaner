use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static SCAN_PAUSED: AtomicBool = AtomicBool::new(false);
static SCAN_CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn reset() {
    SCAN_PAUSED.store(false, Ordering::SeqCst);
    SCAN_CANCELLED.store(false, Ordering::SeqCst);
}

pub fn set_paused(paused: bool) {
    SCAN_PAUSED.store(paused, Ordering::SeqCst);
}

pub fn set_cancelled(cancelled: bool) {
    SCAN_CANCELLED.store(cancelled, Ordering::SeqCst);
}

pub fn is_paused() -> bool {
    SCAN_PAUSED.load(Ordering::SeqCst)
}

pub fn is_cancelled() -> bool {
    SCAN_CANCELLED.load(Ordering::SeqCst)
}

pub fn wait_if_paused() {
    while is_paused() && !is_cancelled() {
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// 长循环内周期性调用；返回 true 表示应中断扫描
pub fn checkpoint(index: u64) -> bool {
    if index % 200 == 0 {
        wait_if_paused();
        return is_cancelled();
    }
    false
}

pub fn apply_action(action: &str) -> Result<(), String> {
    match action {
        "pause" => set_paused(true),
        "resume" => set_paused(false),
        "cancel" => {
            set_cancelled(true);
            set_paused(false);
        }
        "reset" => reset(),
        _ => return Err(format!("未知扫描控制指令: {action}")),
    }
    Ok(())
}
