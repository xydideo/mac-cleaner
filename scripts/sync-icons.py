#!/usr/bin/env python3
"""以 AppIcon.iconset 中的芒果主图为源，同步 Bundle / 托盘 / UI Logo。"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ICONS = ROOT / "src-tauri" / "icons"
ICONSET = ICONS / "AppIcon.iconset"
MASTER = ICONS / "mango-source.png"
PUBLIC_LOGO = ROOT / "public" / "image.png"
PUBLIC_FAVICON = ROOT / "public" / "icon.png"

# Dock：芒果占画布比例（去白边后再缩放）
ICON_FILL = 0.76
# Dock 圆角底板占整图标比例（留出透明边距，避免 Dock 里显得偏大）
DOCK_PLATE_FILL = 0.80
# macOS Dock 圆角（相对边长比例，近似系统 squircle）
SQUIRCLE_RADIUS_RATIO = 0.2237
# 菜单栏托盘：在 22/44pt 画布内尽可能放大芒果剪影
TRAY_FILL = 0.90
# 客户端侧边栏 Logo：透明底，仅芒果主体
UI_LOGO_FILL = 0.92

TRAY_1X = 22
TRAY_2X = 44
UI_LOGO = 128

ICONSET_SIZES: dict[str, int] = {
    "icon_16x16.png": 16,
    "icon_16x16@2x.png": 32,
    "icon_32x32.png": 32,
    "icon_32x32@2x.png": 64,
    "icon_128x128.png": 128,
    "icon_128x128@2x.png": 256,
    "icon_256x256.png": 256,
    "icon_256x256@2x.png": 512,
    "icon_512x512.png": 512,
    "icon_512x512@2x.png": 1024,
}


def ensure_pillow():
    try:
        from PIL import Image  # noqa: F401
    except ImportError:
        print("需要 Pillow：pip install pillow", file=sys.stderr)
        sys.exit(1)


def master_source() -> Path:
    if MASTER.exists():
        return MASTER
    preferred = ICONSET / "icon_512x512@2x.png"
    fallback = ICONSET / "icon_512x512.png"
    if preferred.exists():
        return preferred
    if fallback.exists():
        return fallback
    print("缺少 AppIcon.iconset 芒果主图", file=sys.stderr)
    sys.exit(1)


def remove_white_background(image, threshold: int = 245):
    from PIL import Image

    rgba = image.convert("RGBA")
    pixels = rgba.load()
    width, height = rgba.size
    for y in range(height):
        for x in range(width):
            red, green, blue, alpha = pixels[x, y]
            if alpha == 0:
                continue
            if red >= threshold and green >= threshold and blue >= threshold:
                pixels[x, y] = (255, 255, 255, 0)
    return rgba


def extract_artwork(image):
    from PIL import Image

    rgba = remove_white_background(image)
    bbox = rgba.getbbox()
    if bbox:
        return rgba.crop(bbox)
    return rgba


def fit_artwork_on_canvas(
    artwork,
    canvas_size: int,
    fill_ratio: float,
    background=(255, 255, 255, 255),
):
    from PIL import Image

    art_w, art_h = artwork.size
    max_inner = max(1, int(round(canvas_size * fill_ratio)))
    scale = min(max_inner / art_w, max_inner / art_h)
    new_w = max(1, int(round(art_w * scale)))
    new_h = max(1, int(round(art_h * scale)))
    resized = artwork.resize((new_w, new_h), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (canvas_size, canvas_size), background)
    offset_x = (canvas_size - new_w) // 2
    offset_y = (canvas_size - new_h) // 2
    canvas.paste(resized, (offset_x, offset_y), resized)
    return canvas


def squircle_mask(size: int):
    from PIL import Image, ImageDraw

    radius = max(1, int(round(size * SQUIRCLE_RADIUS_RATIO)))
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle((0, 0, size - 1, size - 1), radius=radius, fill=255)
    return mask


def apply_squircle_mask(image):
    from PIL import Image

    size = image.size[0]
    mask = squircle_mask(size)
    clipped = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    clipped.paste(image, (0, 0), mask)
    return clipped


def make_dock_icon(artwork, canvas_size: int) -> "Image.Image":
    from PIL import Image

    plate_size = max(1, int(round(canvas_size * DOCK_PLATE_FILL)))
    square = fit_artwork_on_canvas(
        artwork,
        plate_size,
        ICON_FILL,
        background=(255, 255, 255, 255),
    )
    plate = apply_squircle_mask(square)

    canvas = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    offset = (canvas_size - plate_size) // 2
    canvas.paste(plate, (offset, offset), plate)
    return canvas


def make_ui_logo(artwork, canvas_size: int) -> "Image.Image":
    return fit_artwork_on_canvas(
        artwork,
        canvas_size,
        UI_LOGO_FILL,
        background=(0, 0, 0, 0),
    )


def make_tray_template(artwork, dest: Path, size: int) -> None:
    from PIL import Image

    tray = fit_artwork_on_canvas(
        artwork,
        size,
        TRAY_FILL,
        background=(0, 0, 0, 0),
    )
    black = Image.new("L", tray.size, 0)
    _, _, _, alpha = tray.split()
    template = Image.merge("RGBA", (black, black, black, alpha))
    template.save(dest, optimize=True)
    print(f"✓ {dest.relative_to(ROOT)} ({size}x{size}, fill={TRAY_FILL})")


def save_png(image, dest: Path) -> None:
    image.save(dest, optimize=True)
    print(f"✓ {dest.relative_to(ROOT)} ({image.size[0]}x{image.size[1]})")


def rebuild_icns() -> None:
    icns = ICONS / "icon.icns"
    subprocess.run(
        ["iconutil", "-c", "icns", str(ICONSET), "-o", str(icns)],
        check=True,
    )
    print(f"✓ {icns.relative_to(ROOT)}")


def main() -> None:
    ensure_pillow()
    from PIL import Image

    source_path = master_source()
    master = Image.open(source_path).convert("RGBA")
    print(f"源图: {source_path.relative_to(ROOT)}")

    if not MASTER.exists() and source_path != MASTER:
        save_png(master.copy(), MASTER)
        print(f"已保存主图源: {MASTER.relative_to(ROOT)}")

    artwork = extract_artwork(master)
    print(f"芒果主体: {artwork.size[0]}x{artwork.size[1]}")

    ICONSET.mkdir(parents=True, exist_ok=True)
    for filename, size in ICONSET_SIZES.items():
        icon = make_dock_icon(artwork, size)
        save_png(icon, ICONSET / filename)

    save_png(make_dock_icon(artwork, 512), ICONS / "icon.png")
    save_png(make_dock_icon(artwork, 32), ICONS / "32x32.png")
    save_png(make_dock_icon(artwork, 128), ICONS / "128x128.png")
    save_png(make_dock_icon(artwork, 256), ICONS / "128x128@2x.png")
    save_png(make_dock_icon(artwork, 64), ICONS / "64x64.png")

    ui_logo = make_ui_logo(artwork, UI_LOGO)
    save_png(ui_logo, PUBLIC_LOGO)
    save_png(make_dock_icon(artwork, 32), PUBLIC_FAVICON)

    make_tray_template(artwork, ICONS / "tray-template.png", TRAY_1X)
    make_tray_template(artwork, ICONS / "tray-template@2x.png", TRAY_2X)

    if sys.platform == "darwin":
        try:
            rebuild_icns()
        except (subprocess.CalledProcessError, FileNotFoundError) as error:
            print(f"跳过 icns 重建: {error}")

    print(
        f"芒果图标同步完成（Dock plate={DOCK_PLATE_FILL}, fill={ICON_FILL}, "
        f"圆角={SQUIRCLE_RADIUS_RATIO}, Tray fill={TRAY_FILL}）"
    )


if __name__ == "__main__":
    main()
