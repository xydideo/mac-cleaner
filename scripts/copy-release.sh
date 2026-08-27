#!/usr/bin/env bash
# 将 Tauri 打包产物复制为 apps/芒果清理.dmg（仅保留 dmg，不含 .app 目录）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APPS="$ROOT/apps"

find_dmg_dir() {
  local dir
  for dir in \
    "$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/dmg" \
    "${CARGO_TARGET_DIR:+$CARGO_TARGET_DIR/universal-apple-darwin/release/bundle/dmg}" \
    "${CARGO_BUILD_TARGET_DIR:+$CARGO_BUILD_TARGET_DIR/universal-apple-darwin/release/bundle/dmg}"
  do
    if [ -n "$dir" ] && [ -d "$dir" ]; then
      echo "$dir"
      return 0
    fi
  done
  return 1
}

TARGET="$(find_dmg_dir || true)"
if [ -z "$TARGET" ]; then
  echo "未找到打包目录，请先执行: pnpm build" >&2
  exit 1
fi

BUNDLE_DMG=""
for candidate in "$TARGET"/*.dmg; do
  if [ -f "$candidate" ]; then
    BUNDLE_DMG="$candidate"
    break
  fi
done

if [ -z "$BUNDLE_DMG" ]; then
  echo "未找到 .dmg 文件: $TARGET" >&2
  exit 1
fi

mkdir -p "$APPS"
rm -f "$APPS"/*.dmg
rm -rf "$APPS"/*.app

cp "$BUNDLE_DMG" "$APPS/芒果清理.dmg"
echo "✓ $APPS/芒果清理.dmg ($(du -h "$APPS/芒果清理.dmg" | cut -f1))"
