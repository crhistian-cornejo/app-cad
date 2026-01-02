//! CADHY I/O - File import/export (stub)
//!
//! Handles STL, OBJ, and other file format I/O.

mod error;

pub use error::{IoError, Result};

/// Export mesh to STL format
pub fn export_stl(_vertices: &[[f32; 3]], _normals: &[[f32; 3]], _indices: &[u32], _path: &std::path::Path) -> Result<()> {
    Err(IoError::NotImplemented("STL export".into()))
}

/// Export mesh to OBJ format  
pub fn export_obj(_vertices: &[[f32; 3]], _normals: &[[f32; 3]], _indices: &[u32], _path: &std::path::Path) -> Result<()> {
    Err(IoError::NotImplemented("OBJ export".into()))
}
