//! Camera system for 3D viewport navigation
//!
//! Implements orbit camera similar to Blender's viewport navigation.

mod camera;
mod controller;
mod projection;

pub use camera::Camera;
pub use controller::CameraController;
pub use projection::Projection;