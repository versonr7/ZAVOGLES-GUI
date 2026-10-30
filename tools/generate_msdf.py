#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont
import numpy as np
import os

FONT_PATH = "/usr/share/fonts/dejavu/DejaVuSans.ttf"
ATLAS_SIZE = 512
FONT_SIZE = 32          
GLYPHS_PER_ROW = 16
START_CHAR, END_CHAR = 32, 127
OUTPUT_DIR = "assets"
DISTANCE_RANGE = 4

def generate_msdf_for_glyph(char, font, cell_w, cell_h):
    img = Image.new("L", (cell_w, cell_h), 0)
    draw = ImageDraw.Draw(img)
    bbox = draw.textbbox((0, 0), char, font=font)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    
    if tw <= 0 or th <= 0:
        return np.zeros((cell_h, cell_w, 4), dtype=np.uint8), 0, 0
        
    ox = (cell_w - tw) // 2 - bbox[0]
    oy = (cell_h - th) // 2 - bbox[1]
    draw.text((ox, oy), char, font=font, fill=255)

    arr = np.array(img, dtype=np.float32) / 255.0
    h, w = arr.shape
    y, x = np.mgrid[0:h, 0:w]
    inside = arr > 0.5
    
    dist_in = np.where(inside, 1.0, 0.0) * DISTANCE_RANGE
    dist_out = np.where(inside, 0.0, 1.0) * DISTANCE_RANGE
    sdf = (dist_in - dist_out) / (2 * DISTANCE_RANGE) + 0.5
    
    # 4 قنوات: RGB للمسافة، و A للشفافية
    msdf = np.stack([sdf, sdf, sdf, np.ones_like(sdf)], axis=-1)
    msdf_u8 = np.clip(msdf * 255, 0, 255).astype(np.uint8)
    return msdf_u8, tw, th

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    os.makedirs(os.path.join(OUTPUT_DIR, "..", "za-app", "src"), exist_ok=True)
    
    font = ImageFont.truetype(FONT_PATH, FONT_SIZE)
    cell_w = ATLAS_SIZE // GLYPHS_PER_ROW
    cell_h = ATLAS_SIZE // GLYPHS_PER_ROW

    atlas = np.zeros((ATLAS_SIZE, ATLAS_SIZE, 4), dtype=np.uint8)
    glyph_data = []

    for i, code in enumerate(range(START_CHAR, END_CHAR)):
        ch = chr(code)
        col, row = i % GLYPHS_PER_ROW, i // GLYPHS_PER_ROW
        cx, cy = col * cell_w, row * cell_h

        msdf_cell, tw, th = generate_msdf_for_glyph(ch, font, cell_w, cell_h)
        atlas[cy:cy+cell_h, cx:cx+cell_w] = msdf_cell

        if tw > 0:
            uv_x, uv_y = cx / ATLAS_SIZE, cy / ATLAS_SIZE
            uv_w, uv_h = tw / ATLAS_SIZE, th / ATLAS_SIZE
        else:
            uv_x, uv_y, uv_w, uv_h = 0.0, 0.0, 0.0, 0.0

        advance = float(tw + 2.0) if tw > 0 else cell_w * 0.5

        glyph_data.append({
            'char': ch, 'uv_x': uv_x, 'uv_y': uv_y, 'uv_w': uv_w, 'uv_h': uv_h,
            'width': float(tw), 'height': float(th), 'advance': advance
        })

    # 1. حفظ الصورة الخام (RGBA)
    with open(os.path.join(OUTPUT_DIR, "font_msdf.rgba"), "wb") as f:
        f.write(atlas.tobytes())
    
    # 2. توليد ملف Rust مباشرة
    rust_path = os.path.join(OUTPUT_DIR, "..", "za-app", "src", "glyphs_msdf.rs")
    with open(rust_path, "w") as f:
        f.write("// Auto-generated MSDF Glyphs\nuse za_gles::font::Glyph;\n\n")
        f.write(f"pub const FONT_GLYPHS: [Option<Glyph>; {len(glyph_data)}] = [\n")
        for g in glyph_data:
            f.write(f"    // {repr(g['char'])}\n    Some(Glyph {{\n")
            f.write(f"        uv_x: {g['uv_x']:.6}f32, uv_y: {g['uv_y']:.6}f32, uv_w: {g['uv_w']:.6}f32, uv_h: {g['uv_h']:.6}f32,\n")
            f.write(f"        width: {g['width']:.1f}f32, height: {g['height']:.1f}f32,\n")
            f.write(f"        advance: {g['advance']:.1f}f32, x_offset: 0.0, y_offset: 0.0,\n")
            f.write("    }),\n")
        f.write("];\n")

    print("✅ Saved font_msdf.rgba and za-app/src/glyphs_msdf.rs")

if __name__ == "__main__":
    main()