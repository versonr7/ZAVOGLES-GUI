#![no_std]
#![warn(missing_docs)]

use core::ops::{Add, Sub, Mul, Div, Neg};

/// 2D vector (f32) - Stack size: 8 bytes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline(always)]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        libm::sqrtf(self.length_sq())
    }

    #[inline(always)]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 { self / len } else { Self::ZERO }
    }

    #[inline(always)]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    #[inline(always)]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline(always)]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs) }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self { Self::new(self.x / rhs, self.y / rhs) }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y) }
}

/// 3D vector (f32) - Stack size: 12 bytes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline(always)]
    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

/// 4x4 Matrix - Column-major (OpenGL) - Stack size: 64 bytes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub const fn from_array(m: [[f32; 4]; 4]) -> Self {
        Self { m }
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let rl = right - left;
        let tb = top - bottom;
        let fn_ = far - near;
        Self::from_array([
            [2.0 / rl, 0.0, 0.0, 0.0],
            [0.0, 2.0 / tb, 0.0, 0.0],
            [0.0, 0.0, -2.0 / fn_, 0.0],
            [-(right + left) / rl, -(top + bottom) / tb, -(far + near) / fn_, 1.0],
        ])
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::IDENTITY;
        m.m[3][0] = x; m.m[3][1] = y; m.m[3][2] = z;
        m
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::IDENTITY;
        m.m[0][0] = x; m.m[1][1] = y; m.m[2][2] = z;
        m
    }

    pub fn mul(self, other: Self) -> Self {
        let mut r = Self::IDENTITY;
        for col in 0..4 {
            for row in 0..4 {
                r.m[col][row] =
                    self.m[0][row] * other.m[col][0] +
                    self.m[1][row] * other.m[col][1] +
                    self.m[2][row] * other.m[col][2] +
                    self.m[3][row] * other.m[col][3];
            }
        }
        r
    }

    pub fn transform_vec2(self, v: Vec2) -> Vec2 {
        let x = self.m[0][0] * v.x + self.m[1][0] * v.y + self.m[3][0];
        let y = self.m[0][1] * v.x + self.m[1][1] * v.y + self.m[3][1];
        let w = self.m[0][3] * v.x + self.m[1][3] * v.y + self.m[3][3];
        if w != 0.0 { Vec2::new(x / w, y / w) } else { Vec2::new(x, y) }
    }
    pub fn to_array(self) -> [f32; 16] {
        [
            self.m[0][0], self.m[0][1], self.m[0][2], self.m[0][3],
            self.m[1][0], self.m[1][1], self.m[1][2], self.m[1][3],
            self.m[2][0], self.m[2][1], self.m[2][2], self.m[2][3],
            self.m[3][0], self.m[3][1], self.m[3][2], self.m[3][3],
        ]
    }

}

/// Axis-aligned rectangle - Stack size: 16 bytes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { min: pos, max: pos + size }
    }

    pub fn from_coords(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self::from_pos_size(Vec2::new(x, y), Vec2::new(w, h))
    }

    pub fn width(self) -> f32 { self.max.x - self.min.x }
    pub fn height(self) -> f32 { self.max.y - self.min.y }
    pub fn size(self) -> Vec2 { Vec2::new(self.width(), self.height()) }
    pub fn center(self) -> Vec2 { (self.min + self.max) * 0.5 }

    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }

    pub fn intersects(self, other: Self) -> bool {
        self.min.x < other.max.x && self.max.x > other.min.x &&
        self.min.y < other.max.y && self.max.y > other.min.y
    }

    pub fn intersection(self, other: Self) -> Self {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        if min.x < max.x && min.y < max.y { Self { min, max } }
        else { Self::from_pos_size(Vec2::ZERO, Vec2::ZERO) }
    }

    pub fn expand(self, padding: f32) -> Self {
        Self {
            min: Vec2::new(self.min.x - padding, self.min.y - padding),
            max: Vec2::new(self.max.x + padding, self.max.y + padding),
        }
    }

    pub fn translate(self, offset: Vec2) -> Self {
        Self { min: self.min + offset, max: self.max + offset }
    }
}

/// RGBA Color - Stack size: 16 bytes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const XMB_BLUE: Self = Self { r: 0.0, g: 0.4, b: 0.8, a: 1.0 };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    pub fn premultiply(self) -> Self {
        Self::new(self.r * self.a, self.g * self.a, self.b * self.a, self.a)
    }

    pub fn to_u8(self) -> [u8; 4] {
        [(self.r * 255.0 + 0.5) as u8, (self.g * 255.0 + 0.5) as u8, (self.b * 255.0 + 0.5) as u8, (self.a * 255.0 + 0.5) as u8]
    }
}

/// Animation easing functions
pub mod easing {
    #[inline(always)]
    pub fn linear(t: f32) -> f32 { t }

    #[inline(always)]
    pub fn ease_out_cubic(t: f32) -> f32 {
        let t = 1.0 - t;
        1.0 - t * t * t
    }

    #[inline(always)]
    pub fn ease_out_expo(t: f32) -> f32 {
        if t >= 1.0 { 1.0 } else { 1.0 - libm::powf(2.0, -10.0 * t) }
    }

    #[inline(always)]
    pub fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 { 2.0 * t * t } else { 1.0 - { let v = -2.0 * t + 2.0; v * v } / 2.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Vec2 Tests ==========

    #[test]
    fn vec2_new() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn vec2_add() {
        let c = Vec2::new(1.0, 2.0) + Vec2::new(3.0, 4.0);
        assert_eq!(c, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn vec2_sub() {
        let c = Vec2::new(5.0, 5.0) - Vec2::new(2.0, 3.0);
        assert_eq!(c, Vec2::new(3.0, 2.0));
    }

    #[test]
    fn vec2_mul_scalar() {
        let r = Vec2::new(2.0, 3.0) * 2.0;
        assert_eq!(r, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn vec2_div_scalar() {
        let r = Vec2::new(4.0, 6.0) / 2.0;
        assert_eq!(r, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn vec2_neg() {
        assert_eq!(-Vec2::new(1.0, -2.0), Vec2::new(-1.0, 2.0));
    }

    #[test]
    fn vec2_dot() {
        assert_eq!(Vec2::new(1.0, 2.0).dot(Vec2::new(3.0, 4.0)), 11.0);
    }

    #[test]
    fn vec2_length() {
        assert_eq!(Vec2::new(3.0, 4.0).length(), 5.0);
    }

    #[test]
    fn vec2_normalize() {
        let n = Vec2::new(3.0, 4.0).normalize();
        assert!((n.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn vec2_normalize_zero() {
        assert_eq!(Vec2::ZERO.normalize(), Vec2::ZERO);
    }

    #[test]
    fn vec2_lerp() {
        assert_eq!(Vec2::ZERO.lerp(Vec2::new(10.0, 20.0), 0.5), Vec2::new(5.0, 10.0));
    }

    #[test]
    fn vec2_min_max() {
        let a = Vec2::new(1.0, 5.0);
        let b = Vec2::new(3.0, 2.0);
        assert_eq!(a.min(b), Vec2::new(1.0, 2.0));
        assert_eq!(a.max(b), Vec2::new(3.0, 5.0));
    }

    // ========== Vec3 Tests ==========

    #[test]
    fn vec3_mul_scalar() {
        let r = Vec3::new(1.0, 2.0, 3.0) * 2.0;
        assert_eq!(r, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn vec3_xy() {
        assert_eq!(Vec3::new(1.0, 2.0, 3.0).xy(), Vec2::new(1.0, 2.0));
    }

    // ========== Mat4 Tests ==========

    #[test]
    fn mat4_identity() {
        let m = Mat4::IDENTITY;
        for i in 0..4 {
            for j in 0..4 {
                assert_eq!(m.m[i][j], if i == j { 1.0 } else { 0.0 });
            }
        }
    }

    #[test]
    fn mat4_translate() {
        let r = Mat4::translate(5.0, 10.0, 0.0).transform_vec2(Vec2::new(1.0, 2.0));
        assert_eq!(r, Vec2::new(6.0, 12.0));
    }

    #[test]
    fn mat4_scale() {
        let r = Mat4::scale(2.0, 3.0, 1.0).transform_vec2(Vec2::new(1.0, 1.0));
        assert_eq!(r, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn mat4_ortho() {
        let m = Mat4::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
        let tl = m.transform_vec2(Vec2::new(0.0, 0.0));
        assert!((tl.x + 1.0).abs() < 0.001);
        assert!((tl.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn mat4_translate_then_scale() {
        let m = Mat4::translate(10.0, 0.0, 0.0).mul(Mat4::scale(2.0, 1.0, 1.0));
        let r = m.transform_vec2(Vec2::new(1.0, 0.0));
        assert_eq!(r, Vec2::new(12.0, 0.0));
    }

    // ========== Rect Tests ==========

    #[test]
    fn rect_from_pos_size() {
        let r = Rect::from_pos_size(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));
        assert_eq!(r.min, Vec2::new(10.0, 20.0));
        assert_eq!(r.max, Vec2::new(110.0, 70.0));
    }

    #[test]
    fn rect_contains() {
        let r = Rect::from_coords(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Vec2::new(50.0, 50.0)));
        assert!(!r.contains(Vec2::new(101.0, 50.0)));
    }

    #[test]
    fn rect_intersects() {
        let a = Rect::from_coords(0.0, 0.0, 100.0, 100.0);
        let b = Rect::from_coords(50.0, 50.0, 100.0, 100.0);
        assert!(a.intersects(b));
        assert!(!a.intersects(Rect::from_coords(200.0, 200.0, 50.0, 50.0)));
    }

    #[test]
    fn rect_intersection() {
        let i = Rect::from_coords(0.0, 0.0, 100.0, 100.0)
            .intersection(Rect::from_coords(50.0, 50.0, 100.0, 100.0));
        assert_eq!(i, Rect::from_coords(50.0, 50.0, 50.0, 50.0));
    }

    #[test]
    fn rect_expand() {
        let e = Rect::from_coords(10.0, 10.0, 80.0, 80.0).expand(5.0);
        assert_eq!(e, Rect::from_coords(5.0, 5.0, 90.0, 90.0));
    }

    #[test]
    fn rect_translate() {
        let t = Rect::from_coords(0.0, 0.0, 100.0, 100.0).translate(Vec2::new(10.0, 20.0));
        assert_eq!(t, Rect::from_coords(10.0, 20.0, 100.0, 100.0));
    }

    // ========== Color Tests ==========

    #[test]
    fn color_from_u8() {
        let c = Color::from_u8(255, 128, 0, 255);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.5).abs() < 0.01);
    }

    #[test]
    fn color_lerp() {
        let mid = Color::BLACK.lerp(Color::WHITE, 0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
    }

    #[test]
    fn color_premultiply() {
        let p = Color::new(1.0, 0.5, 0.25, 0.5).premultiply();
        assert_eq!(p, Color::new(0.5, 0.25, 0.125, 0.5));
    }

    #[test]
    fn color_to_u8() {
        assert_eq!(Color::new(1.0, 0.5, 0.0, 1.0).to_u8(), [255, 128, 0, 255]);
    }

    // ========== Easing Tests ==========

    #[test]
    fn easing_linear() {
        assert_eq!(easing::linear(0.5), 0.5);
    }

    #[test]
    fn easing_ease_out_cubic() {
        assert!((easing::ease_out_cubic(0.5) - 0.875).abs() < 0.001);
    }

    #[test]
    fn easing_ease_out_expo() {
        assert_eq!(easing::ease_out_expo(0.0), 0.0);
        assert_eq!(easing::ease_out_expo(1.0), 1.0);
    }

    #[test]
    fn easing_ease_in_out_quad() {
        assert_eq!(easing::ease_in_out_quad(0.0), 0.0);
        assert_eq!(easing::ease_in_out_quad(0.5), 0.5);
        assert_eq!(easing::ease_in_out_quad(1.0), 1.0);
    }

    // ========== Memory Tests ==========

    #[test]
    fn verify_stack_sizes() {
        assert_eq!(core::mem::size_of::<Vec2>(), 8);
        assert_eq!(core::mem::size_of::<Vec3>(), 12);
        assert_eq!(core::mem::size_of::<Mat4>(), 64);
        assert_eq!(core::mem::size_of::<Rect>(), 16);
        assert_eq!(core::mem::size_of::<Color>(), 16);
    }
}