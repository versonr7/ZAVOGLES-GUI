#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont
import os

FONT_PATH = "tools/DejaVuSans.ttf"
if not os.path.exists(FONT_PATH):
    FONT_PATH = "/usr/share/fonts/dejavu/DejaVuSans.ttf"

ATLAS_SIZE = 512
FONT_SIZE = 32
OUTPUT_DIR = "assets"
GLYPHS_PER_ROW = 16
START_CHAR = 32
END_CHAR = 127

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    if not os.path.exists(FONT_PATH):
        print(f"Font not found: {FONT_PATH}")
        return

    font = ImageFont.truetype(FONT_PATH, FONT_SIZE)
    atlas = Image.new("RGBA", (ATLAS_SIZE, ATLAS_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(atlas)

    cell_w = ATLAS_SIZE // GLYPHS_PER_ROW
    cell_h = ATLAS_SIZE // GLYPHS_PER_ROW

    glyph_data = []

    for i, code in enumerate(range(START_CHAR, END_CHAR)):
        ch = chr(code)
        col = i % GLYPHS_PER_ROW
        row = i // GLYPHS_PER_ROW
        cell_x = col * cell_w
        cell_y = row * cell_h

        bbox = draw.textbbox((0, 0), ch, font=font)
        tw = bbox[2] - bbox[0]
        th = bbox[3] - bbox[1]

        if tw <= 0 or th <= 0:
            # مسافة أو حرف غير مرئي
            draw.text((cell_x, cell_y), ch, font=font, fill=(255, 255, 255, 255))
            uv_x, uv_y, uv_w, uv_h = 0.0, 0.0, 0.0, 0.0
            width, height = 0.0, 0.0
            advance = float(cell_w * 0.5) if ch == ' ' else float(cell_w)
        else:
            # توسيط الحرف داخل الخلية
            draw_x = cell_x + (cell_w - tw) // 2 - bbox[0]
            draw_y = cell_y + (cell_h - th) // 2 - bbox[1]
            draw.text((draw_x, draw_y), ch, font=font, fill=(255, 255, 255, 255))

            # حساب إحداثيات UV الدقيقة للحرف فقط
            actual_x = cell_x + (cell_w - tw) // 2
            actual_y = cell_y + (cell_h - th) // 2

            uv_x = actual_x / ATLAS_SIZE
            uv_y = actual_y / ATLAS_SIZE
            uv_w = tw / ATLAS_SIZE
            uv_h = th / ATLAS_SIZE
            
            width = float(tw)
            height = float(th)
            
            # المسافة بين الحروف = عرض الحرف + 2 بكسل
            advance = float(tw + 2.0)

        glyph_data.append({
            'char': ch,
            'uv_x': uv_x,
            'uv_y': uv_y,
            'uv_w': uv_w,
            'uv_h': uv_h,
            'width': width,
            'height': height,
            'advance': advance,
            'x_offset': 0.0,
            'y_offset': 0.0,
        })

    with open(os.path.join(OUTPUT_DIR, "font_atlas.rgba"), "wb") as f:
        f.write(atlas.tobytes())
    atlas.save(os.path.join(OUTPUT_DIR, "font_atlas.png"))
    print(f"Saved atlas: {ATLAS_SIZE}x{ATLAS_SIZE}")

    with open(os.path.join(OUTPUT_DIR, "font_glyphs.rs"), "w") as f:
        f.write("// Auto-generated\n")
        f.write("use za_gles::font::Glyph;\n\n")
        f.write(f"pub const FONT_GLYPHS: [Option<Glyph>; {len(glyph_data)}] = [\n")
        for g in glyph_data:
            esc = repr(g['char'])
            f.write(f"    // {esc}\n    Some(Glyph {{\n")
            f.write(f"        uv_x: {g['uv_x']:.6}f32,\n")
            f.write(f"        uv_y: {g['uv_y']:.6}f32,\n")
            f.write(f"        uv_w: {g['uv_w']:.6}f32,\n")
            f.write(f"        uv_h: {g['uv_h']:.6}f32,\n")
            f.write(f"        width: {g['width']:.1}f32,\n")
            f.write(f"        height: {g['height']:.1}f32,\n")
            f.write(f"        advance: {g['advance']:.1}f32,\n")
            f.write(f"        x_offset: {g['x_offset']:.1}f32,\n")
            f.write(f"        y_offset: {g['y_offset']:.1}f32,\n")
            f.write("    }),\n")
        f.write("];\n")
    print("Saved font_glyphs.rs")

if __name__ == "__main__":
    main()
