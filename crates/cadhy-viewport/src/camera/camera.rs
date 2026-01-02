use super::projection::Projection;
use glam::{Mat4, Vec3};

/// 3D Camera with position, target, and projection
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub projection: Projection,
    pub aspect_ratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: Projection::default(),
            aspect_ratio: 16.0 / 9.0,
        }
    }
}

impl Camera {
    /// Create a new camera at the given position looking at target
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            ..Default::default()
        }
    }

    /// View matrix (world -> camera space)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Projection matrix (camera -> clip space)
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection.matrix(self.aspect_ratio)
    }

    /// Combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Distance from camera to target
    pub fn distance(&self) -> f32 {
        (self.position - self.target).length()
    }

    /// Set aspect ratio (call on resize)
    pub fn set_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height.max(1) as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_default() {
        let camera = Camera::default();
        assert!((camera.distance() - 8.66).abs() < 0.1); // sqrt(5^2 + 5^2 + 5^2)
    }

    #[test]
    fn test_view_projection() {
        let camera = Camera::default();
        let vp = camera.view_projection_matrix();
        assert!(!vp.is_nan());
    }
}