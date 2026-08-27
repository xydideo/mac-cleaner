import { convertFileSrc } from "@tauri-apps/api/core"
import { isTauriRuntime } from "@/utils/tauriPlatform"

const IMAGE_EXTENSIONS = new Set([
  "jpg",
  "jpeg",
  "png",
  "gif",
  "webp",
  "bmp",
  "svg",
  "heic",
  "heif",
  "tif",
  "tiff",
  "ico",
  "avif",
])

export function isImageFile(name: string): boolean {
  const dot = name.lastIndexOf(".")
  if (dot <= 0 || dot === name.length - 1) return false
  return IMAGE_EXTENSIONS.has(name.slice(dot + 1).toLowerCase())
}

export function imagePreviewSrc(path: string): string {
  if (!isTauriRuntime() || !path) return ""
  return convertFileSrc(path)
}
