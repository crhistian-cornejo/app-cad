//! GPU Picking system
//!
//! Uses a separate render pass with unique colors per object
//! to identify which object is under the mouse cursor.

mod buffer;
mod color;

pub use buffer::PickingBuffer;
pub use color::{color_to_id, id_to_color};