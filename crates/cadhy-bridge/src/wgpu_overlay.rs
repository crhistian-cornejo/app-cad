//! WGPU Overlay - Direct wgpu rendering to WebviewWindow surface
//!
//! This module implements the tauri-wgpu-cam approach where:
//! - wgpu surface is created directly from the WebviewWindow
//! - wgpu renders to the ENTIRE window (no coordinate conversion needed)
//! - React UI components overlay on top with z-index and transparent backgrounds
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    WebviewWindow                             │
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              wgpu Surface (full window)                 ││
//! │  │           renders 3D content at 100+ FPS                ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │                          ▲                                   │
//! │                          │ z-index layering                  │
//! │                          ▼                                   │
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │         React UI (transparent background)               ││
//! │  │    ┌──────────┐                    ┌──────────┐         ││
//! │  │    │ Toolbar  │    transparent     │ Toolbar  │         ││
//! │  │    └──────────┘      center        └──────────┘         ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! Benefits:
//! - No coordinate conversion between CSS and native pixels
//! - No child window management
//! - Simpler codebase
//! - Works on all platforms

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tauri::{WebviewWindow, Runtime};

use cadhy_viewport::{Camera, NativeViewport, Projection, Scene, ViewMode};

/// Messages sent to the render thread
#[derive(Debug)]
pub enum RenderMessage {
    Resize { width: u32, height: u32 },
    Orbit { delta_x: f32, delta_y: f32 },
    Pan { delta_x: f32, delta_y: f32 },
    Zoom { delta: f32 },
    FrameAll,
    ResetCamera,
    SetViewMode(ViewMode),
    UpdateScene,
    Stop,
}

/// Thread-safe channel sender for render messages
pub type RenderSender = crossbeam_channel::Sender<RenderMessage>;

/// FPS statistics from the render thread
#[derive(Debug, Clone, Copy, Default)]
pub struct FpsStats {
    pub fps: f64,
    pub frame_time_ms: f64,
}

/// WGPU Overlay that renders directly to the WebviewWindow surface
///
/// This approach creates the wgpu surface from the WebviewWindow itself,
/// so wgpu renders to the entire window area. The React UI overlays on top
/// using CSS z-index and transparent backgrounds.
pub struct WgpuOverlay {
    /// Channel to send commands to render thread
    sender: Option<RenderSender>,
    /// Whether the render loop is running
    running: Arc<AtomicBool>,
    /// Current viewport size
    size: Arc<RwLock<(u32, u32)>>,
    /// Shared scene for rendering
    scene: Arc<RwLock<Scene>>,
    /// Current view mode
    view_mode: Arc<RwLock<ViewMode>>,
    /// Render thread handle
    render_thread: Option<JoinHandle<()>>,
    /// Current FPS stats (updated by render thread)
    fps_stats: Arc<RwLock<FpsStats>>,
}

impl WgpuOverlay {
    /// Create a new WGPU overlay from a WebviewWindow
    ///
    /// # Arguments
    /// * `window` - The WebviewWindow to render into
    /// * `width` - Initial width
    /// * `height` - Initial height
    ///
    /// # Important
    /// On macOS, this MUST be called from the main thread (Metal requirement)
    pub fn new<R: Runtime>(
        _window: &WebviewWindow<R>,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        tracing::info!(
            "Creating WgpuOverlay {}x{} on WebviewWindow",
            width, height
        );

        Ok(Self {
            sender: None,
            running: Arc::new(AtomicBool::new(false)),
            size: Arc::new(RwLock::new((width, height))),
            scene: Arc::new(RwLock::new(Scene::new())),
            view_mode: Arc::new(RwLock::new(ViewMode::Solid)),
            render_thread: None,
            fps_stats: Arc::new(RwLock::new(FpsStats::default())),
        })
    }

    /// Start the render loop
    ///
    /// # Important
    /// On macOS, this MUST be called from the main thread (Metal requirement)
    pub fn start_render_loop<R: Runtime>(
        &mut self,
        window: WebviewWindow<R>,
    ) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Render loop already running".to_string());
        }

        let (width, height) = *self.size.read().map_err(|e| e.to_string())?;

        tracing::info!("Creating NativeViewport on main thread...");
        tracing::info!("  Requested size: {}x{}", width, height);

        // Create wgpu viewport from WebviewWindow
        // The WebviewWindow implements HasWindowHandle and HasDisplayHandle
        let window_arc = Arc::new(window);

        let viewport = pollster::block_on(async {
            NativeViewport::new(window_arc, width, height).await
        })
        .map_err(|e| {
            tracing::error!("NativeViewport creation failed: {:?}", e);
            format!("Failed to create NativeViewport: {:?}", e)
        })?;

        tracing::info!("NativeViewport created successfully!");
        tracing::info!("  Surface format: {:?}", viewport.surface_format());
        tracing::info!("  Actual size: {}x{}", viewport.size().0, viewport.size().1);

        // Wrap viewport for thread transfer
        let viewport = Arc::new(Mutex::new(Some(viewport)));

        // Create message channel
        let (sender, receiver) = crossbeam_channel::unbounded::<RenderMessage>();
        self.sender = Some(sender);

        // Clone references for render thread
        let running = self.running.clone();
        let scene = self.scene.clone();
        let view_mode = self.view_mode.clone();
        let viewport_for_thread = viewport.clone();
        let fps_stats = self.fps_stats.clone();

        running.store(true, Ordering::SeqCst);

        // Spawn render thread
        let handle = thread::spawn(move || {
            // Take ownership of viewport
            let mut viewport = {
                let mut guard = match viewport_for_thread.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        tracing::error!("Failed to acquire viewport lock: {}", e);
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                };
                match guard.take() {
                    Some(v) => v,
                    None => {
                        tracing::error!("Viewport was already taken");
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                }
            };

            tracing::info!(
                "Render thread started, viewport size: {}x{}",
                viewport.size().0,
                viewport.size().1
            );

            // Initialize camera
            let (w, h) = viewport.size();
            let mut camera = Camera {
                position: glam::Vec3::new(5.0, 5.0, 5.0),
                target: glam::Vec3::ZERO,
                up: glam::Vec3::Y,
                projection: Projection::Perspective {
                    fov: 45.0_f32.to_radians(),
                    near: 0.1,
                    far: 1000.0,
                },
                aspect_ratio: w as f32 / h as f32,
            };

            // Use Immediate present mode for uncapped FPS
            let mut frame_count = 0u64;
            let mut last_fps_log = Instant::now();

            // Main render loop - runs as fast as possible
            while running.load(Ordering::SeqCst) {
                let frame_start = Instant::now();

                // Process all pending messages (non-blocking)
                while let Ok(msg) = receiver.try_recv() {
                    match msg {
                        RenderMessage::Resize { width, height } => {
                            if width > 0 && height > 0 {
                                viewport.resize(width, height);
                                camera.aspect_ratio = width as f32 / height.max(1) as f32;
                                tracing::debug!("Viewport resized to {}x{}", width, height);
                            }
                        }
                        RenderMessage::Orbit { delta_x, delta_y } => {
                            let sensitivity = 0.005;
                            let offset = camera.position - camera.target;
                            let distance = offset.length();

                            let mut theta = offset.z.atan2(offset.x);
                            let mut phi = (offset.y / distance).acos();

                            theta -= delta_x * sensitivity;
                            phi = (phi + delta_y * sensitivity)
                                .clamp(0.01, std::f32::consts::PI - 0.01);

                            camera.position = camera.target
                                + glam::Vec3::new(
                                    distance * phi.sin() * theta.cos(),
                                    distance * phi.cos(),
                                    distance * phi.sin() * theta.sin(),
                                );
                        }
                        RenderMessage::Pan { delta_x, delta_y } => {
                            let sensitivity = 0.01;
                            let forward = (camera.target - camera.position).normalize();
                            let right = forward.cross(camera.up).normalize();
                            let up = right.cross(forward);

                            let offset =
                                right * (-delta_x * sensitivity) + up * (delta_y * sensitivity);
                            camera.position += offset;
                            camera.target += offset;
                        }
                        RenderMessage::Zoom { delta } => {
                            let direction = (camera.target - camera.position).normalize();
                            let distance = (camera.position - camera.target).length();
                            let zoom_speed = distance * 0.001;
                            let new_distance = (distance - delta * zoom_speed).max(0.5);

                            camera.position = camera.target - direction * new_distance;
                        }
                        RenderMessage::FrameAll => {
                            camera.position = glam::Vec3::new(5.0, 5.0, 5.0);
                            camera.target = glam::Vec3::ZERO;
                        }
                        RenderMessage::ResetCamera => {
                            camera.position = glam::Vec3::new(5.0, 5.0, 5.0);
                            camera.target = glam::Vec3::ZERO;
                            camera.up = glam::Vec3::Y;
                        }
                        RenderMessage::SetViewMode(mode) => {
                            if let Ok(mut vm) = view_mode.write() {
                                *vm = mode;
                            }
                            viewport.render_context_mut().set_view_mode(mode);
                        }
                        RenderMessage::UpdateScene => {
                            // Scene update - will render next frame
                        }
                        RenderMessage::Stop => {
                            running.store(false, Ordering::SeqCst);
                        }
                    }
                }

                // Render frame
                if let Ok(scene_guard) = scene.read() {
                    if let Err(e) = viewport.render(&scene_guard, &camera) {
                        tracing::error!("Render error: {:?}", e);
                    }
                }

                // FPS tracking - update every second
                frame_count += 1;
                let elapsed_since_fps_update = last_fps_log.elapsed();
                if elapsed_since_fps_update >= Duration::from_secs(1) {
                    let fps = frame_count as f64 / elapsed_since_fps_update.as_secs_f64();
                    let frame_time_ms = 1000.0 / fps;

                    // Update FPS stats for UI
                    if let Ok(mut stats) = fps_stats.write() {
                        stats.fps = fps;
                        stats.frame_time_ms = frame_time_ms;
                    }

                    // Log periodically
                    static mut LOG_COUNTER: u32 = 0;
                    unsafe {
                        LOG_COUNTER += 1;
                        if LOG_COUNTER >= 5 {
                            tracing::info!("WgpuOverlay FPS: {:.1}", fps);
                            LOG_COUNTER = 0;
                        }
                    }

                    frame_count = 0;
                    last_fps_log = Instant::now();
                }

                // Small yield to prevent CPU spinning when VSync is off
                // This still allows 500+ FPS while being CPU-friendly
                let elapsed = frame_start.elapsed();
                if elapsed < Duration::from_micros(500) {
                    thread::sleep(Duration::from_micros(100));
                }
            }

            tracing::info!("Render thread stopped");
        });

        self.render_thread = Some(handle);
        tracing::info!("WgpuOverlay render loop started");
        Ok(())
    }

    /// Send a message to the render thread
    pub fn send(&self, msg: RenderMessage) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender.send(msg).map_err(|e| e.to_string())
        } else {
            Err("Render loop not started".to_string())
        }
    }

    /// Resize the viewport
    pub fn resize(&self, width: u32, height: u32) -> Result<(), String> {
        if let Ok(mut size) = self.size.write() {
            *size = (width, height);
        }
        self.send(RenderMessage::Resize { width, height })
    }

    /// Check if the render loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get shared scene reference
    pub fn scene(&self) -> Arc<RwLock<Scene>> {
        self.scene.clone()
    }

    /// Get current view mode
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
            .read()
            .map(|v| *v)
            .unwrap_or(ViewMode::Solid)
    }

    /// Get current FPS statistics
    pub fn fps_stats(&self) -> FpsStats {
        self.fps_stats
            .read()
            .map(|s| *s)
            .unwrap_or_default()
    }

    /// Stop the render loop
    pub fn stop(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            self.running.store(false, Ordering::SeqCst);
            if let Some(sender) = &self.sender {
                let _ = sender.send(RenderMessage::Stop);
            }
            if let Some(handle) = self.render_thread.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for WgpuOverlay {
    fn drop(&mut self) {
        self.stop();
    }
}
