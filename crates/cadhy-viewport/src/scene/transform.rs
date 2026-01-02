use glam::{Mat4, Vec3};

/// Transform component for scene objects
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: glam::Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Create transform matrix
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Create from Euler angles (radians)
    pub fn from_euler(position: Vec3, euler: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation: glam::Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z),
            scale,
        }
    }
}