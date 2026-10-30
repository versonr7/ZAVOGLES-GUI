#![no_std]

use crate::{BatchRenderer, Texture};
use za_math::{Color, Rect};

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_w: f32,
    pub uv_h: f32,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

pub struct BitmapFont {
    pub atlas: Texture,
    pub line_height: f32,
    pub glyphs: [Option<Glyph>; 95],
}

impl BitmapFont {
    pub fn from_atlas_data(
        atlas_data: &[u8],
        atlas_w: i32,
        atlas_h: i32,
        glyphs: [Option<Glyph>; 95],
        line_height: f32,
    ) -> Result<Self, i32> {
        let atlas = Texture::new()?;
        atlas
            .upload_rgba(atlas_w, atlas_h, atlas_data)
            .map_err(|_| 0x9991)?;
        Ok(Self {
            atlas,
            line_height,
            glyphs,
        })
    }

    pub fn glyph_for(&self, c: char) -> Option<&Glyph> {
        let idx = c as usize;
        if idx >= 32 && idx < 127 {
            self.glyphs[idx - 32].as_ref()
        } else {
            None
        }
    }

    pub fn measure_text(&self, text: &str, scale: f32) -> f32 {
        let mut x = 0.0;
        for c in text.chars() {
            if let Some(g) = self.glyph_for(c) {
                x += g.advance * scale;
            }
        }
        x
    }
}

pub fn draw_text<const MAX_VERTICES: usize, const MAX_INDICES: usize>(
    batch: &mut BatchRenderer<MAX_VERTICES, MAX_INDICES>,
    font: &BitmapFont,
    text: &str,
    mut x: f32,
    y: f32,
    scale: f32,
    color: Color,
) {
    batch.set_texture(&font.atlas);
    for c in text.chars() {
        if let Some(g) = font.glyph_for(c) {
            // استخدام الإزاحات الصحيحة
            let draw_x = x + g.x_offset * scale;
            let draw_y = y + g.y_offset * scale;
            
            let rect = Rect::from_coords(draw_x, draw_y, g.width * scale, g.height * scale);
            
            // ✅ إزالة القلب نهائياً (استخدام g.uv_y مباشرة)
            let uv = Rect::from_coords(g.uv_x, g.uv_y, g.uv_w, g.uv_h);
            
            batch.draw_quad(rect, uv, color);
            x += g.advance * scale;
        }
    }
}
