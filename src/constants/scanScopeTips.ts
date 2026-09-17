/** 概览 / 垃圾清理「智能扫描（快速）」范围说明 */
export const QUICK_SCAN_SCOPE_TIP = `扫描范围：
• ~/Library/Caches（第三方应用缓存）
• ~/Library/Logs（应用日志）
• 废纸篓`

/** 概览 / 垃圾清理「深度扫描」额外范围说明 */
export const DEEP_SCAN_SCOPE_TIP = `在智能扫描基础上增加：
• 仅「已卸载」的第三方应用 Preferences/*.plist（遗留配置）
• Application Support 内更新包与更新缓存（并入应用缓存扫描）
• ~/Downloads 中 >100MB 的文件`
