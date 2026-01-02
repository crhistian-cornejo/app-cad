//! CADHY Desktop - Tauri application with wgpu rendering
//!
//! Architecture: Raw Window + wgpu surface + WebView child overlay
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Raw Window (WindowBuilder)                      │
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              wgpu Surface (full window)                 ││
//! │  │           renders 3D content at 100+ FPS                ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │                          ▲                                   │
//! │                          │ z-index layering                  │
//! │                          ▼                                   │
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │      Child WebView (transparent background)             ││
//! │  │      React UI renders on top of wgpu content            ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```

use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{Sender, unbounded};
use tauri::{
    Manager, RunEvent, WindowEvent,
    window::WindowBuilder,
    webview::WebviewBuilder,
    WebviewUrl, LogicalPosition, LogicalSize,
};

use cadhy_viewport::{Camera, NativeViewport, Projection, Scene, ViewMode};

/// Global state for the wgpu viewport
pub struct WgpuViewportState {
    pub running: Arc<AtomicBool>,
    pub scene: Arc<RwLock<Scene>>,
    pub sender: Mutex<Option<Sender<ViewportMessage>>>,
    pub fps: Arc<AtomicU32>,
    pub frame_time_us: Arc<AtomicU32>,
}

impl WgpuViewportState {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            scene: Arc::new(RwLock::new(Scene::new())),
            sender: Mutex::new(None),
            fps: Arc::new(AtomicU32::new(0)),
            frame_time_us: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn send(&self, msg: ViewportMessage) -> Result<(), String> {
        if let Ok(guard) = self.sender.lock() {
            if let Some(sender) = guard.as_ref() {
                sender.send(msg).map_err(|e: crossbeam_channel::SendError<ViewportMessage>| e.to_string())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ViewportMessage {
    Resize { width: u32, height: u32 },
    Orbit { delta_x: f32, delta_y: f32 },
    Pan { delta_x: f32, delta_y: f32 },
    Zoom { delta: f32 },
    ResetCamera,
    SetViewMode(ViewMode),
    AddCube { name: String, size: f32 },
    Stop,
}

// ============================================================================
// TAURI COMMANDS - Viewport Control
// ============================================================================

#[tauri::command]
fn viewport_orbit(
    state: tauri::State<Arc<WgpuViewportState>>,
    delta_x: f32,
    delta_y: f32,
) -> Result<(), String> {
    println!("[CADHY] Orbit command: dx={}, dy={}", delta_x, delta_y);
    state.send(ViewportMessage::Orbit { delta_x, delta_y })
}

#[tauri::command]
fn viewport_pan(
    state: tauri::State<Arc<WgpuViewportState>>,
    delta_x: f32,
    delta_y: f32,
) -> Result<(), String> {
    state.send(ViewportMessage::Pan { delta_x, delta_y })
}

#[tauri::command]
fn viewport_zoom(
    state: tauri::State<Arc<WgpuViewportState>>,
    delta: f32,
) -> Result<(), String> {
    state.send(ViewportMessage::Zoom { delta })
}

#[tauri::command]
fn viewport_reset_camera(
    state: tauri::State<Arc<WgpuViewportState>>,
) -> Result<(), String> {
    state.send(ViewportMessage::ResetCamera)
}

#[tauri::command]
fn viewport_set_view_mode(
    state: tauri::State<Arc<WgpuViewportState>>,
    mode: String,
) -> Result<(), String> {
    let view_mode = match mode.as_str() {
        "wireframe" => ViewMode::Wireframe,
        _ => ViewMode::Solid,
    };
    state.send(ViewportMessage::SetViewMode(view_mode))
}

#[tauri::command]
fn viewport_get_fps(
    state: tauri::State<Arc<WgpuViewportState>>,
) -> Result<serde_json::Value, String> {
    let fps = state.fps.load(Ordering::Relaxed);
    let frame_time_us = state.frame_time_us.load(Ordering::Relaxed);
    Ok(serde_json::json!({
        "fps": fps,
        "frame_time_ms": frame_time_us as f64 / 1000.0
    }))
}

#[tauri::command]
fn scene_add_cube(
    state: tauri::State<Arc<WgpuViewportState>>,
    name: String,
    size: f32,
) -> Result<String, String> {
    // Send message to render thread which has GPU context for mesh creation
    state.send(ViewportMessage::AddCube { name: name.clone(), size })?;
    println!("[CADHY] Requested cube: {}", name);
    Ok(name) // Return name as confirmation (actual UUID generated in render thread)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,wgpu=warn")),
        )
        .init();

    let wgpu_state = Arc::new(WgpuViewportState::new());
    let wgpu_state_for_setup = wgpu_state.clone();
    let wgpu_state_for_events = wgpu_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(cadhy_bridge::init())
        .manage(wgpu_state.clone())
        .invoke_handler(tauri::generate_handler![
            viewport_orbit,
            viewport_pan,
            viewport_zoom,
            viewport_reset_camera,
            viewport_set_view_mode,
            viewport_get_fps,
            scene_add_cube,
        ])
        .setup(move |app| {
            println!("[CADHY] Setting up inverted architecture: Raw Window + wgpu + WebView overlay");

            let app_handle = app.app_handle().clone();
            let state = wgpu_state_for_setup.clone();

            // Create channel for viewport messages
            let (tx, rx) = unbounded::<ViewportMessage>();
            if let Ok(mut guard) = state.sender.lock() {
                *guard = Some(tx);
            }

            // === STEP 1: Create RAW Window (no webview) ===
            // This window will host the wgpu surface
            let window = WindowBuilder::new(&app_handle, "main")
                .title("CADHY")
                .inner_size(1400.0, 900.0)
                .min_inner_size(800.0, 600.0)
                .resizable(true)
                .decorations(true)
                .transparent(true) // IMPORTANT: Allow transparency for webview overlay
                .visible(true)
                .center()
                .build()
                .expect("Failed to create main window");

            println!("[CADHY] Raw window created successfully");

            // Get window size for wgpu (physical) and webview (logical)
            let scale_factor = window.scale_factor().unwrap_or(1.0);
            let window_size = window.inner_size().unwrap_or(tauri::PhysicalSize {
                width: 1400,
                height: 900,
            });
            // wgpu uses physical pixels
            let width = window_size.width;
            let height = window_size.height;
            // webview uses logical pixels
            let logical_width = width as f64 / scale_factor;
            let logical_height = height as f64 / scale_factor;

            println!("[CADHY] Scale factor: {}", scale_factor);
            println!("[CADHY] Physical size: {}x{}", width, height);
            println!("[CADHY] Logical size: {}x{}", logical_width, logical_height);

            // === STEP 2: Initialize wgpu on this window (MUST be on main thread for macOS Metal) ===
            println!("[CADHY] Creating wgpu surface on main thread...");

            let window_arc = Arc::new(window.clone());
            let viewport = pollster::block_on(async {
                NativeViewport::new(window_arc, width, height).await
            });

            match viewport {
                Ok(viewport) => {
                    println!("[CADHY] wgpu surface created successfully!");
                    println!("[CADHY]   Format: {:?}", viewport.surface_format());
                    println!("[CADHY]   Size: {}x{}", viewport.size().0, viewport.size().1);

                    // Wrap for thread transfer
                    let viewport = Arc::new(Mutex::new(Some(viewport)));
                    let viewport_for_thread = viewport.clone();
                    let scene = Arc::clone(&state.scene);
                    let running = Arc::clone(&state.running);
                    let fps_counter = Arc::clone(&state.fps);
                    let frame_time_counter = Arc::clone(&state.frame_time_us);

                    running.store(true, Ordering::SeqCst);

                    // Start render thread
                    thread::spawn(move || {
                        let mut viewport = {
                            let mut guard = viewport_for_thread.lock().unwrap();
                            guard.take().expect("Viewport already taken")
                        };

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

                        let mut frame_count = 0u64;
                        let mut last_fps_update = Instant::now();
                        let mut frame_start: Instant;

                        println!("[CADHY] Render thread started");

                        while running.load(Ordering::SeqCst) {
                            frame_start = Instant::now();
                            // Process messages
                            while let Ok(msg) = rx.try_recv() {
                                match msg {
                                    ViewportMessage::Resize { width, height } => {
                                        if width > 0 && height > 0 {
                                            viewport.resize(width, height);
                                            camera.aspect_ratio = width as f32 / height as f32;
                                        }
                                    }
                                    ViewportMessage::Orbit { delta_x, delta_y } => {
                                        // Orbit around target using spherical coordinates
                                        let sensitivity = 0.005;

                                        // Get current camera offset from target
                                        let dir = camera.position - camera.target;
                                        let radius = dir.length();

                                        // Convert to spherical: yaw (horizontal), pitch (vertical)
                                        let mut yaw = dir.z.atan2(dir.x);
                                        let mut pitch = (dir.y / radius).asin();

                                        // Apply rotation - horizontal mouse = yaw, vertical = pitch
                                        yaw -= delta_x * sensitivity;
                                        pitch += delta_y * sensitivity;

                                        // Clamp pitch to avoid gimbal lock
                                        pitch = pitch.clamp(-1.5, 1.5);

                                        // Convert back to Cartesian
                                        camera.position = camera.target + glam::Vec3::new(
                                            radius * pitch.cos() * yaw.cos(),
                                            radius * pitch.sin(),
                                            radius * pitch.cos() * yaw.sin(),
                                        );
                                    }
                                    ViewportMessage::Pan { delta_x, delta_y } => {
                                        let sensitivity = 0.01;
                                        let forward = (camera.target - camera.position).normalize();
                                        let right = forward.cross(camera.up).normalize();
                                        let up = right.cross(forward);
                                        let offset = right * (-delta_x * sensitivity) + up * (delta_y * sensitivity);
                                        camera.position += offset;
                                        camera.target += offset;
                                    }
                                    ViewportMessage::Zoom { delta } => {
                                        let direction = (camera.target - camera.position).normalize();
                                        let distance = (camera.position - camera.target).length();
                                        // Zoom speed proportional to distance for consistent feel
                                        // Scroll deltaY is ~100 per tick, so use small multiplier
                                        let zoom_speed = distance * 0.002;
                                        let new_distance = (distance - delta * zoom_speed).clamp(0.5, 100.0);
                                        camera.position = camera.target - direction * new_distance;
                                    }
                                    ViewportMessage::ResetCamera => {
                                        camera.position = glam::Vec3::new(5.0, 5.0, 5.0);
                                        camera.target = glam::Vec3::ZERO;
                                        camera.up = glam::Vec3::Y;
                                    }
                                    ViewportMessage::SetViewMode(mode) => {
                                        viewport.render_context_mut().set_view_mode(mode);
                                    }
                                    ViewportMessage::AddCube { name, size } => {
                                        use cadhy_viewport::{SceneObject, GpuMesh};

                                        // Create mesh using GPU device from viewport
                                        let mesh = GpuMesh::cube_multicolor(viewport.device(), size);
                                        let obj = SceneObject::new(name).with_mesh(mesh);

                                        // Add to scene
                                        if let Ok(mut scene_guard) = scene.write() {
                                            let id = scene_guard.add(obj);
                                            println!("[CADHY] Added cube with id: {:?}", id);
                                        }
                                    }
                                    ViewportMessage::Stop => {
                                        running.store(false, Ordering::SeqCst);
                                    }
                                }
                            }

                            // Render
                            if let Ok(scene_guard) = scene.read() {
                                if let Err(e) = viewport.render(&scene_guard, &camera) {
                                    eprintln!("[CADHY] Render error: {:?}", e);
                                }
                            }

                            // Update frame time (in microseconds)
                            let frame_time = frame_start.elapsed();
                            frame_time_counter.store(frame_time.as_micros() as u32, Ordering::Relaxed);

                            // FPS calculation - update every second for smooth display
                            frame_count += 1;
                            let elapsed = last_fps_update.elapsed();
                            if elapsed >= Duration::from_secs(1) {
                                let fps = (frame_count as f64 / elapsed.as_secs_f64()) as u32;
                                fps_counter.store(fps, Ordering::Relaxed);
                                frame_count = 0;
                                last_fps_update = Instant::now();
                            }

                            // Small sleep to prevent 100% CPU usage while still allowing high FPS
                            thread::sleep(Duration::from_micros(100));
                        }

                        println!("[CADHY] Render thread stopped");
                    });
                }
                Err(e) => {
                    eprintln!("[CADHY] Failed to create wgpu surface: {:?}", e);
                    eprintln!("[CADHY] Continuing without wgpu rendering");
                }
            }

            // === STEP 3: Add WebView child for React UI (transparent, on top) ===
            println!("[CADHY] Adding WebView overlay for React UI...");

            // Get the dev URL or production path
            let webview_url = WebviewUrl::App("index.html".into());

            let webview = WebviewBuilder::new("ui", webview_url)
                .transparent(true)
                .auto_resize();

            // Add webview as child covering the entire window (use logical coordinates!)
            let _ui_webview = window
                .add_child(
                    webview,
                    LogicalPosition::new(0.0, 0.0),
                    LogicalSize::new(logical_width, logical_height),
                )
                .expect("Failed to create UI webview");

            println!("[CADHY] UI WebView created - React runs on top of wgpu");
            println!("[CADHY] Setup complete! wgpu renders behind, React UI on top");

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app_handle, event| {
            match event {
                RunEvent::WindowEvent {
                    label,
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    if label == "main" {
                        // Notify wgpu about resize
                        let _ = wgpu_state_for_events.send(ViewportMessage::Resize {
                            width: size.width,
                            height: size.height,
                        });
                    }
                }
                RunEvent::ExitRequested { .. } => {
                    // Stop render thread
                    wgpu_state_for_events.running.store(false, Ordering::SeqCst);
                    let _ = wgpu_state_for_events.send(ViewportMessage::Stop);
                }
                _ => {}
            }
        });
}
