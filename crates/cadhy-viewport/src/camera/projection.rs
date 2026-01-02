use glam::Mat4;

/// Camera projection type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    Perspective { fov: f32, near: f32, far: f32 },
    Orthographic { scale: f32, near: f32, far: f32 },
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective {
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Projection {
    /// Projection matrix (camera -> clip space)
    pub fn matrix(&self, aspect_ratio: f32) -> Mat4 {
        match self {
            Projection::Perspective { fov, near, far } => {
                Mat4::perspective_rh(*fov, aspect_ratio, *near, *far)
            }
            Projection::Orthographic { scale, near, far } => {
                let half_width = scale * aspect_ratio;
                let half_height = *scale;
                Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, *near, *far)
            }
        }
    }
}