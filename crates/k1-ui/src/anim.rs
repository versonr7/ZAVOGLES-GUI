//! Animation system - simple tweening

use k1_core::math::{lerp, smoothstep};

#[derive(Clone, Copy, Debug)]
pub struct AnimationState {
    pub column_offset: f32,
    pub target_offset: f32,
    pub animating: bool,
    pub progress: f32,
    pub speed: f32,
}

impl AnimationState {
    pub const fn new() -> Self {
        Self {
            column_offset: 0.0,
            target_offset: 0.0,
            animating: false,
            progress: 0.0,
            speed: 8.0, // Animation speed
        }
    }
    
    pub fn start_column_transition(&mut self, from: f32, to: f32) {
        self.column_offset = from;
        self.target_offset = to;
        self.animating = true;
        self.progress = 0.0;
    }
    
    pub fn update(&mut self, dt: f32) {
        if !self.animating {
            return;
        }
        
        self.progress += dt * self.speed;
        
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.animating = false;
            self.column_offset = self.target_offset;
        } else {
            let t = smoothstep(0.0, 1.0, self.progress);
            self.column_offset = lerp(self.column_offset, self.target_offset, t);
        }
    }
    
    pub const fn current_column_offset(&self) -> f32 {
        self.column_offset
    }
}
