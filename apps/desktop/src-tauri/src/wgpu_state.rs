//! wgpu State - Main thread GPU initialization
//!
//! This module handles wgpu initialization on the main thread,
//! which is required for Metal backend on macOS.

use std::sync::{Arc, Mutex, RwLock};
use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration};
use cadhy_viewport::{Camera, RenderContext, Scene};
use glam::Vec3;

/// GPU state initialized on the main thread
pub struct WgpuState {
    pub instance: Instance,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Surface<'static>,
    pub config: Mutex<SurfaceConfiguration>,
    pub render_context: RwLock<RenderContext>,
    pub scene: RwLock<Scene>,
    pub camera: RwLock<Camera>,
}

impl WgpuState {
    /// Create new wgpu state - MUST be called on main thread
    pub fn new<W>(window: Arc<W>, width: u32, height: u32) -> Self
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static,
    {
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface - this MUST happen on main thread for Metal
        let surface = instance.create_surface(window).expect("Failed to create surface");

        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find GPU adapter");

        println!("[wgpu] Using adapter: {:?}", adapter.get_info().name);

        // Request device
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("CADHY Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed to create device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync, // 60 FPS with VSync
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create render context
        let render_context = RenderContext::new(&device, format);

        // Create default camera
        let mut camera = Camera::new(
            Vec3::new(5.0, 5.0, 5.0),
            Vec3::ZERO,
        );
        camera.set_aspect_ratio(width, height);

        println!("[wgpu] Initialized {}x{} @ {:?}", width, height, format);

        Self {
            instance,
            device,
            queue,
            surface,
            config: Mutex::new(config),
            render_context: RwLock::new(render_context),
            scene: RwLock::new(Scene::new()),
            camera: RwLock::new(camera),
        }
    }

    /// Resize the surface
    pub fn resize(&self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        let mut config = self.config.lock().unwrap();
        config.width = width;
        config.height = height;
        self.surface.configure(&self.device, &config);

        // Update camera aspect ratio
        if let Ok(mut camera) = self.camera.write() {
            camera.set_aspect_ratio(width, height);
        }
    }

    /// Orbit the camera
    pub fn orbit(&self, delta_x: f32, delta_y: f32) {
        if let Ok(mut camera) = self.camera.write() {
            use glam::Mat4;

            let to_camera = camera.position - camera.target;
            let distance = to_camera.length();

            // Horizontal rotation (around Y axis)
            let rotation_y = Mat4::from_rotation_y(-delta_x * 0.01);

            // Get right vector for vertical rotation
            let forward = to_camera.normalize();
            let right = camera.up.cross(forward).normalize();

            // Vertical rotation (around right axis)
            let rotation_x = Mat4::from_axis_angle(right, -delta_y * 0.01);

            // Apply rotations
            let new_direction = rotation_x.transform_vector3(rotation_y.transform_vector3(to_camera));
            camera.position = camera.target + new_direction.normalize() * distance;
        }
    }

    /// Pan the camera
    pub fn pan(&self, delta_x: f32, delta_y: f32) {
        if let Ok(mut camera) = self.camera.write() {
            let forward = (camera.target - camera.position).normalize();
            let right = forward.cross(camera.up).normalize();
            let camera_up = right.cross(forward).normalize();

            let distance = (camera.target - camera.position).length();
            let pan_speed = distance * 0.002;

            let offset = right * (-delta_x * pan_speed) + camera_up * (delta_y * pan_speed);

            camera.position += offset;
            camera.target += offset;
        }
    }

    /// Zoom the camera
    pub fn zoom(&self, delta: f32) {
        if let Ok(mut camera) = self.camera.write() {
            let to_target = camera.target - camera.position;
            let distance = to_target.length();

            let zoom_factor = 1.0 - delta * 0.001;
            let new_distance = (distance * zoom_factor).max(0.1).min(10000.0);

            let direction = to_target.normalize();
            camera.position = camera.target - direction * new_distance;
        }
    }

    /// Render a frame
    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let config = self.config.lock().unwrap();
        let width = config.width;
        let height = config.height;
        drop(config);

        // Create depth texture
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Update camera uniforms
        {
            let camera = self.camera.read().unwrap();
            let render_context = self.render_context.read().unwrap();
            render_context.update_camera(&self.queue, &camera);
        }

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let scene = self.scene.read().unwrap();
            let render_context = self.render_context.read().unwrap();

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(render_context.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_context.render(&mut render_pass, &scene, &self.queue);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get current size
    pub fn size(&self) -> (u32, u32) {
        let config = self.config.lock().unwrap();
        (config.width, config.height)
    }
}

// Safety: wgpu resources are thread-safe
unsafe impl Send for WgpuState {}
unsafe impl Sync for WgpuState {}
