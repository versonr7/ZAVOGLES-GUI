//! Simple math utilities (no_std compatible)

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp value between min and max
pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else if v > max { max } else { v }
}

/// Smooth step function (Hermite interpolation)
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sine wave with time
pub fn wave(t: f32, frequency: f32, amplitude: f32) -> f32 {
    (t * frequency).sin() * amplitude
}

/// 2D transformation matrix (simplified)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Mat3 {
    pub m: [f32; 9],
}

impl Mat3 {
    pub const fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0,
            ]
        }
    }
    
    pub const fn translation(x: f32, y: f32) -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                x,   y,   1.0,
            ]
        }
    }
    
    pub const fn scale(x: f32, y: f32) -> Self {
        Self {
            m: [
                x,   0.0, 0.0,
                0.0, y,   0.0,
                0.0, 0.0, 1.0,
            ]
        }
    }
}
