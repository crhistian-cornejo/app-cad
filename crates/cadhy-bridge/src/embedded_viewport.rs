//! Embedded Viewport - Native wgpu rendering as child window
//!
//! Creates a native child window embedded in the main Tauri window for
//! high-performance GPU rendering. The child window automatically moves
//! with the parent, eliminating position sync issues.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Main Tauri Window                        │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │                  React WebView UI                     │   │
//! │  │  ┌─────────────┐ ┌────────────────────┐ ┌──────────┐ │   │
//! │  │  │ Left Panel  │ │   Viewport Area    │ │  Right   │ │   │
//! │  │  │             │ │   (transparent)    │ │  Panel   │ │   │
//! │  │  └─────────────┘ └────────────────────┘ └──────────┘ │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │                              │                               │
//! │         Child Window (wgpu)  ▼                               │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │           wgpu Surface → 60+ FPS                      │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Runtime, WebviewWindow};
use tauri::window::{Window, WindowBuilder};

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

/// Embedded viewport that renders wgpu content in a child window
pub struct EmbeddedViewport<R: Runtime> {
    /// The child window for wgpu rendering
    window: Window<R>,
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
}

impl<R: Runtime> EmbeddedViewport<R> {
    /// Create a new embedded viewport as a child of the main window
    ///
    /// # Arguments
    /// * `app` - The Tauri app handle
    /// * `main_window` - The main window to embed into (parent)
    /// * `x` - X position relative to main window's inner area
    /// * `y` - Y position relative to main window's inner area
    /// * `width` - Viewport width
    /// * `height` - Viewport height
    pub fn new(
        app: &AppHandle<R>,
        main_window: &WebviewWindow<R>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        tracing::info!(
            "Creating embedded viewport {}x{} at ({}, {})",
            width, height, x, y
        );

        // Get main window's inner position to calculate absolute screen position
        let main_inner_pos = main_window
            .inner_position()
            .map_err(|e| format!("Failed to get main window position: {}", e))?;

        let abs_x = main_inner_pos.x + x;
        let abs_y = main_inner_pos.y + y;

        tracing::info!(
            "Absolute position: ({}, {}) - main inner: ({}, {})",
            abs_x, abs_y, main_inner_pos.x, main_inner_pos.y
        );

        // Get the raw Window from WebviewWindow for parent relationship
        let parent_window = main_window.as_ref().window();

        // Create a child window (raw Window, not WebviewWindow)
        // This window has no webview, just a native surface for wgpu
        let window = WindowBuilder::new(app, "viewport-native")
            .parent(&parent_window)
            .map_err(|e| format!("Failed to set parent window: {}", e))?
            .position(abs_x as f64, abs_y as f64)
            .inner_size(width as f64, height as f64)
            .decorations(false)
            .transparent(false)
            .visible(true)
            .focused(false)
            .skip_taskbar(true)
            .resizable(false)
            .build()
            .map_err(|e| format!("Failed to create viewport window: {}", e))?;

        tracing::info!("Child window created successfully");

        Ok(Self {
            window,
            sender: None,
            running: Arc::new(AtomicBool::new(false)),
            size: Arc::new(RwLock::new((width, height))),
            scene: Arc::new(RwLock::new(Scene::new())),
            view_mode: Arc::new(RwLock::new(ViewMode::Solid)),
            render_thread: None,
        })
    }

    /// Start the render loop
    ///
    /// IMPORTANT: Must be called from the main thread on macOS (Metal requirement)
    pub fn start_render_loop(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Render loop already running".to_string());
        }

        let (width, height) = *self.size.read().map_err(|e| e.to_string())?;

        // Create wgpu viewport on THIS thread (must be main thread on macOS)
        tracing::info!("Creating NativeViewport on main thread...");

        let window_arc = Arc::new(self.window.clone());
        let viewport = pollster::block_on(async {
            NativeViewport::new(window_arc, width, height).await
        })
        .map_err(|e| format!("Failed to create NativeViewport: {:?}", e))?;

        tracing::info!("NativeViewport created, starting render thread...");

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

            let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);
            let mut frame_count = 0u64;
            let mut last_fps_log = Instant::now();

            // Main render loop
            while running.load(Ordering::SeqCst) {
                let frame_start = Instant::now();

                // Process all pending messages
                while let Ok(msg) = receiver.try_recv() {
                    match msg {
                        RenderMessage::Resize { width, height } => {
                            if width > 0 && height > 0 {
                                viewport.resize(width, height);
                                camera.aspect_ratio = width as f32 / height.max(1) as f32;
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

                // FPS logging every 5 seconds
                frame_count += 1;
                if last_fps_log.elapsed() >= Duration::from_secs(5) {
                    let fps = frame_count as f64 / last_fps_log.elapsed().as_secs_f64();
                    tracing::info!("Embedded viewport FPS: {:.1}", fps);
                    frame_count = 0;
                    last_fps_log = Instant::now();
                }

                // Frame timing
                let elapsed = frame_start.elapsed();
                if elapsed < target_frame_time {
                    thread::sleep(target_frame_time - elapsed);
                }
            }

            tracing::info!("Render thread stopped");
        });

        self.render_thread = Some(handle);
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

    /// Check if the render loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get shared scene reference
    pub fn scene(&self) -> Arc<RwLock<Scene>> {
        self.scene.clone()
    }

    /// Update window position and size
    pub fn update_bounds(&self, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
        // Update window position
        let _ = self.window.set_position(tauri::Position::Physical(
            tauri::PhysicalPosition::new(x, y),
        ));

        // Update window size
        let _ = self.window.set_size(tauri::Size::Physical(
            tauri::PhysicalSize::new(width, height),
        ));

        // Notify render thread
        if let Some(sender) = &self.sender {
            let _ = sender.send(RenderMessage::Resize { width, height });
        }

        // Update stored size
        if let Ok(mut size) = self.size.write() {
            *size = (width, height);
        }

        Ok(())
    }

    /// Get current view mode
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
            .read()
            .map(|v| *v)
            .unwrap_or(ViewMode::Solid)
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

    /// Get the window reference
    pub fn window(&self) -> &Window<R> {
        &self.window
    }
}

impl<R: Runtime> Drop for EmbeddedViewport<R> {
    fn drop(&mut self) {
        self.stop();
    }
}
