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

/// FPS statistics from the render thread
#[derive(Debug, Clone, Copy, Default)]
pub struct FpsStats {
    pub fps: f64,
    pub frame_time_ms: f64,
}

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
    /// Current FPS stats (updated by render thread)
    fps_stats: Arc<RwLock<FpsStats>>,
}

impl<R: Runtime> EmbeddedViewport<R> {
    /// Create a new embedded viewport as a child of the main window
    ///
    /// # Arguments
    /// * `app` - The Tauri app handle
    /// * `main_window` - The main window to embed into (parent)
    /// * `x` - X position relative to main window's content area (CSS pixels)
    /// * `y` - Y position relative to main window's content area (CSS pixels)
    /// * `width` - Viewport width (CSS pixels)
    /// * `height` - Viewport height (CSS pixels)
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

        // Get the raw Window from WebviewWindow for parent relationship
        let parent_window = main_window.as_ref().window();

        // With parent() on macOS, the child window positions are relative to the parent window's
        // content area (the area below the title bar).
        //
        // The frontend sends CSS (logical) pixel coordinates relative to the webview's viewport,
        // which corresponds directly to the parent window's content area when using
        // titleBarStyle: "Overlay" with hiddenTitle: true.
        //
        // IMPORTANT: macOS uses a flipped coordinate system where Y increases upward from the
        // bottom of the parent window. We need to convert from top-down to bottom-up coordinates.

        let scale_factor = main_window
            .scale_factor()
            .map_err(|e| format!("Failed to get scale factor: {}", e))?;

        // Get parent window's inner size for coordinate conversion
        let inner_size = main_window
            .inner_size()
            .map_err(|e| format!("Failed to get inner size: {}", e))?;

        // Convert from logical to physical pixels for inner size
        let parent_height_logical = inner_size.height as f64 / scale_factor;

        // macOS child windows with parent() use bottom-left origin
        // Convert from top-down (CSS) to bottom-up (macOS) coordinates
        let child_x = x;
        // For macOS: y_macos = parent_height - (y_css + child_height)
        let child_y = (parent_height_logical as i32) - y - (height as i32);

        tracing::info!(
            "Child window positioning - frontend: ({}, {}) -> macOS: ({}, {}) - parent_height: {} - scale: {}",
            x, y, child_x, child_y, parent_height_logical, scale_factor
        );

        // Create a child window (raw Window, not WebviewWindow)
        // This window has no webview, just a native surface for wgpu
        //
        // STRATEGY: Position wgpu window ON TOP with click-through
        // 1. Use ignore_cursor_events to let mouse clicks pass through to webview
        // 2. Webview receives mouse events and forwards them via IPC
        // 3. wgpu renders at 60+ FPS in the child window
        let window = WindowBuilder::new(app, "viewport-native")
            .parent(&parent_window)
            .map_err(|e| format!("Failed to set parent window: {}", e))?
            .position(child_x as f64, child_y as f64)
            .inner_size(width as f64, height as f64)
            .decorations(false)
            .transparent(false)
            .visible(true)
            .focused(false)
            .skip_taskbar(true)
            .resizable(false)
            .build()
            .map_err(|e| format!("Failed to create viewport window: {}", e))?;

        // Enable cursor event passthrough so mouse events go to the webview behind
        if let Err(e) = window.set_ignore_cursor_events(true) {
            tracing::warn!("Failed to set ignore_cursor_events: {}", e);
        }

        tracing::info!("Child window created successfully (ignore_cursor_events=true)");

        Ok(Self {
            window,
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
    /// IMPORTANT: Must be called from the main thread on macOS (Metal requirement)
    pub fn start_render_loop(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Render loop already running".to_string());
        }

        let (width, height) = *self.size.read().map_err(|e| e.to_string())?;

        // Create wgpu viewport on THIS thread (must be main thread on macOS)
        tracing::info!("Creating NativeViewport on main thread...");
        tracing::info!("  Window label: viewport-native");
        tracing::info!("  Requested size: {}x{}", width, height);

        // Verify window handle is valid before proceeding
        use raw_window_handle::HasWindowHandle;
        match self.window.window_handle() {
            Ok(handle) => {
                tracing::info!("  Window handle acquired successfully");
                tracing::info!("  Handle type: {:?}", handle.as_raw());
            }
            Err(e) => {
                tracing::error!("  Failed to get window handle: {:?}", e);
                return Err(format!("Window handle error: {:?}", e));
            }
        }

        // On macOS, give the Metal layer time to initialize
        #[cfg(target_os = "macos")]
        {
            tracing::info!("  macOS detected - waiting 50ms for Metal layer initialization...");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let window_arc = Arc::new(self.window.clone());
        tracing::info!("  Creating wgpu surface and device...");

        let viewport = pollster::block_on(async {
            NativeViewport::new(window_arc, width, height).await
        })
        .map_err(|e| {
            tracing::error!("  NativeViewport creation failed: {:?}", e);
            format!("Failed to create NativeViewport: {:?}", e)
        })?;

        tracing::info!("NativeViewport created successfully!");
        tracing::info!("  Surface format: {:?}", viewport.surface_format());
        tracing::info!("  Actual size: {}x{}", viewport.size().0, viewport.size().1);
        tracing::info!("Starting render thread...");

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

                // FPS tracking - update every second for UI, log every 5 seconds
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
                    
                    // Log every 5 updates (~5 seconds)
                    static mut LOG_COUNTER: u32 = 0;
                    unsafe {
                        LOG_COUNTER += 1;
                        if LOG_COUNTER >= 5 {
                            tracing::info!("Embedded viewport FPS: {:.1}", fps);
                            LOG_COUNTER = 0;
                        }
                    }
                    
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
    /// Note: x, y should be in logical (CSS) screen coordinates
    pub fn update_bounds(&self, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
        // Update window position using logical coordinates
        let _ = self.window.set_position(tauri::Position::Logical(
            tauri::LogicalPosition::new(x as f64, y as f64),
        ));

        // Update window size using logical coordinates
        let _ = self.window.set_size(tauri::Size::Logical(
            tauri::LogicalSize::new(width as f64, height as f64),
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
