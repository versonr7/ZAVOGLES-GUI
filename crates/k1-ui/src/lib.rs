//! K1-UI: XMB-style UI system
//! Zero-allocation, retained mode

pub mod xmb;
pub mod text;
pub mod anim;

use k1_core::{Color, Vec2, Rect, Vertex, Pool, PoolHandle, K1Result, K1Error};

/// UI renderer context
pub struct UIRenderer {
    // Pre-allocated vertex buffer for UI quads
    max_quads: usize,
}

/// XMB Menu structure
pub struct XMBMenu {
    columns: [XMBColumn; 8],  // Max 8 columns
    column_count: usize,
    selected_column: usize,
    selected_row: usize,
    
    // Animation state
    anim: anim::AnimationState,
}

pub struct XMBColumn {
    pub id: u32,
    pub label: [u8; 32],      // Fixed-size UTF-8 label
    pub icon_id: u16,          // Icon atlas index
    pub items: [XMBItem; 32],  // Max 32 items per column
    pub item_count: usize,
    pub position: Vec2,
}

pub struct XMBItem {
    pub id: u32,
    pub label: [u8; 48],       // Fixed-size UTF-8 label
    pub icon_id: u16,
    pub app_path: [u8; 128],   // Path to emulator/game
    pub position: Vec2,
}

impl XMBMenu {
    pub const fn new() -> Self {
        Self {
            columns: [XMBColumn::new(); 8],
            column_count: 0,
            selected_column: 0,
            selected_row: 0,
            anim: anim::AnimationState::new(),
        }
    }
    
    pub fn add_column(&mut self, label: &str, icon_id: u16) -> K1Result<&mut XMBColumn> {
        if self.column_count >= 8 {
            return Err(K1Error::OutOfBounds);
        }
        
        let col = &mut self.columns[self.column_count];
        col.set_label(label);
        col.icon_id = icon_id;
        self.column_count += 1;
        
        Ok(col)
    }
    
    pub fn navigate_right(&mut self) {
        if self.selected_column + 1 < self.column_count {
            self.anim.start_column_transition(self.selected_column as f32, (self.selected_column + 1) as f32);
            self.selected_column += 1;
            self.selected_row = 0;
        }
    }
    
    pub fn navigate_left(&mut self) {
        if self.selected_column > 0 {
            self.anim.start_column_transition(self.selected_column as f32, (self.selected_column - 1) as f32);
            self.selected_column -= 1;
            self.selected_row = 0;
        }
    }
    
    pub fn navigate_down(&mut self) {
        let col = &self.columns[self.selected_column];
        if self.selected_row + 1 < col.item_count {
            self.selected_row += 1;
        }
    }
    
    pub fn navigate_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
    }
    
    /// Generate vertices for rendering
    pub fn render(&self, screen_w: f32, screen_h: f32, vertices: &mut [Vertex], vertex_count: &mut usize) {
        let base = *vertex_count;
        let mut count = 0;
        
        // Background wave (simplified)
        self.render_wave(screen_w, screen_h, vertices, &mut count);
        
        // Columns
        let offset_x = self.anim.current_column_offset();
        for (i, col) in self.columns.iter().enumerate().take(self.column_count) {
            let x = (i as f32 - offset_x) * 200.0 + screen_w * 0.3;
            let y = screen_h * 0.3;
            
            let selected = i == self.selected_column;
            let alpha = if selected { 1.0f32 } else { 0.6f32 };
            
            self.render_column(col, x, y, alpha, &mut vertices[base + count..], &mut count);
        }
        
        *vertex_count += count;
    }
    
    fn render_wave(&self, _w: f32, _h: f32, _verts: &mut [Vertex], _count: &mut usize) {
        // TODO: Wave shader background
    }
    
    fn render_column(&self, col: &XMBColumn, x: f32, y: f32, alpha: f32, verts: &mut [Vertex], count: &mut usize) {
        // Column header
        self.render_quad(x, y, 160.0, 40.0, Color::new(0, 120, 215, (alpha * 255.0) as u8), verts, count);
        
        // Items
        for (i, item) in col.items.iter().enumerate().take(col.item_count) {
            let item_y = y + 60.0 + i as f32 * 50.0;
            let selected = i == self.selected_row;
            let color = if selected {
                Color::new(255, 255, 255, (alpha * 255.0) as u8)
            } else {
                Color::new(180, 180, 180, (alpha * 200.0) as u8)
            };
            
            self.render_quad(x + 10.0, item_y, 140.0, 40.0, color, verts, count);
        }
    }
    
    fn render_quad(&self, x: f32, y: f32, w: f32, h: f32, color: Color, verts: &mut [Vertex], count: &mut usize) {
        let c = color.to_f32_array();
        
        let base = *count;
        verts[base + 0] = Vertex::new([x, y], [0.0, 0.0], c);
        verts[base + 1] = Vertex::new([x + w, y], [1.0, 0.0], c);
        verts[base + 2] = Vertex::new([x, y + h], [0.0, 1.0], c);
        verts[base + 3] = Vertex::new([x + w, y], [1.0, 0.0], c);
        verts[base + 4] = Vertex::new([x + w, y + h], [1.0, 1.0], c);
        verts[base + 5] = Vertex::new([x, y + h], [0.0, 1.0], c);
        
        *count += 6;
    }
}

impl XMBColumn {
    pub const fn new() -> Self {
        Self {
            id: 0,
            label: [0; 32],
            icon_id: 0,
            items: [XMBItem::new(); 32],
            item_count: 0,
            position: Vec2::zero(),
        }
    }
    
    pub fn set_label(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(31);
        self.label[..len].copy_from_slice(&bytes[..len]);
        self.label[len] = 0;
    }
    
    pub fn add_item(&mut self, label: &str, icon_id: u16, app_path: &str) -> K1Result<()> {
        if self.item_count >= 32 {
            return Err(K1Error::OutOfBounds);
        }
        
        let item = &mut self.items[self.item_count];
        item.set_label(label);
        item.icon_id = icon_id;
        item.set_app_path(app_path);
        self.item_count += 1;
        
        Ok(())
    }
}

impl XMBItem {
    pub const fn new() -> Self {
        Self {
            id: 0,
            label: [0; 48],
            icon_id: 0,
            app_path: [0; 128],
            position: Vec2::zero(),
        }
    }
    
    pub fn set_label(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(47);
        self.label[..len].copy_from_slice(&bytes[..len]);
        self.label[len] = 0;
    }
    
    pub fn set_app_path(&mut self, path: &str) {
        let bytes = path.as_bytes();
        let len = bytes.len().min(127);
        self.app_path[..len].copy_from_slice(&bytes[..len]);
        self.app_path[len] = 0;
    }
}
