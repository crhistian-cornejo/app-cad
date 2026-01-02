use super::camera::Camera;
use glam::Vec3;

/// Camera controller for orbit/pan/zoom navigation (Blender-style)
#[derive(Debug, Clone)]
pub struct CameraController {
    /// Rotation speed in radians per pixel
    pub rotate_speed: f32,
    /// Pan speed in units per pixel
    pub pan_speed: f32,
    /// Zoom speed factor
    pub zoom_speed: f32,
    /// Minimum distance to target
    pub min_distance: f32,
    /// Maximum distance to target
    pub max_distance: f32,

    // Internal state
    is_rotating: bool,
    is_panning: bool,
    last_mouse_pos: (f32, f32),
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            rotate_speed: 0.005,
            pan_speed: 0.01,
            zoom_speed: 0.1,
            min_distance: 0.5,
            max_distance: 500.0,
            is_rotating: false,
            is_panning: false,
            last_mouse_pos: (0.0, 0.0),
        }
    }
}

impl CameraController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start rotation mode (middle mouse button)
    pub fn start_rotate(&mut self, x: f32, y: f32) {
        self.is_rotating = true;
        self.last_mouse_pos = (x, y);
    }

    /// Start pan mode (shift + middle mouse button)
    pub fn start_pan(&mut self, x: f32, y: f32) {
        self.is_panning = true;
        self.last_mouse_pos = (x, y);
    }

    /// Stop rotation/pan
    pub fn stop(&mut self) {
        self.is_rotating = false;
        self.is_panning = false;
    }

    /// Process mouse movement
    pub fn mouse_move(&mut self, camera: &mut Camera, x: f32, y: f32) {
        let dx = x - self.last_mouse_pos.0;
        let dy = y - self.last_mouse_pos.1;
        self.last_mouse_pos = (x, y);

        if self.is_rotating {
            self.orbit(camera, dx, dy);
        } else if self.is_panning {
            self.pan(camera, dx, dy);
        }
    }

    /// Orbit camera around target
    fn orbit(&self, camera: &mut Camera, dx: f32, dy: f32) {
        let distance = camera.distance();
        let offset = camera.position - camera.target;

        // Convert to spherical coordinates
        let radius = offset.length();
        let mut theta = offset.z.atan2(offset.x); // Horizontal angle
        let mut phi = (offset.y / radius).acos(); // Vertical angle

        // Apply rotation
        theta -= dx * self.rotate_speed;
        phi = (phi - dy * self.rotate_speed).clamp(0.01, std::f32::consts::PI - 0.01);

        // Convert back to Cartesian
        let new_offset = Vec3::new(
            distance * phi.sin() * theta.cos(),
            distance * phi.cos(),
            distance * phi.sin() * theta.sin(),
        );

        camera.position = camera.target + new_offset;
    }

    /// Pan camera (move both position and target)
    fn pan(&self, camera: &mut Camera, dx: f32, dy: f32) {
        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward);

        let pan_offset =
            right * (-dx * self.pan_speed * camera.distance())
            + up * (dy * self.pan_speed * camera.distance());

        camera.position += pan_offset;
        camera.target += pan_offset;
    }

    /// Zoom in/out (scroll wheel)
    pub fn zoom(&self, camera: &mut Camera, delta: f32) {
        let direction = (camera.target - camera.position).normalize();
        let distance = camera.distance();
        let zoom_amount = distance * self.zoom_speed * delta;
        let new_distance = (distance - zoom_amount).clamp(self.min_distance, self.max_distance);

        camera.position = camera.target - direction * new_distance;
    }

    /// Frame selected objects (focus camera on bounds)
    pub fn frame(&self, camera: &mut Camera, center: Vec3, radius: f32) {
        let direction = (camera.position - camera.target).normalize();
        let ideal_distance = radius * 2.5; // Give some margin

        camera.target = center;
        camera.position = center + direction * ideal_distance;
    }
}