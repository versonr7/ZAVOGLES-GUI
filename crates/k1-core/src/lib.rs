//! K1-Core: Core types and utilities for K1 Platform
//! Zero-allocation friendly, no_std compatible

#![cfg_attr(not(feature = "std"), no_std)]

pub mod math;
pub mod memory;

#[cfg(feature = "std")]
extern crate std;

/// Color representation (RGBA8)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
    
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    /// Convert to normalized float array [r, g, b, a]
    pub const fn to_f32_array(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

/// 2D position
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Rectangle
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    
    pub const fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x 
            && point.x <= self.x + self.w
            && point.y >= self.y 
            && point.y <= self.y + self.h
    }
}

/// Vertex for UI rendering
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self { pos, uv, color }
    }
}

/// Result type for K1 operations
pub type K1Result<T> = Result<T, K1Error>;

/// Error types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum K1Error {
    OutOfMemory,
    InvalidParameter,
    NotSupported,
    DeviceLost,
    SurfaceLost,
    ShaderCompileError,
    OutOfBounds,
    NotInitialized,
    AlreadyInitialized,
}

impl core::fmt::Display for K1Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::InvalidParameter => write!(f, "Invalid parameter"),
            Self::NotSupported => write!(f, "Not supported"),
            Self::DeviceLost => write!(f, "Device lost"),
            Self::SurfaceLost => write!(f, "Surface lost"),
            Self::ShaderCompileError => write!(f, "Shader compile error"),
            Self::OutOfBounds => write!(f, "Out of bounds"),
            Self::NotInitialized => write!(f, "Not initialized"),
            Self::AlreadyInitialized => write!(f, "Already initialized"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for K1Error {}
