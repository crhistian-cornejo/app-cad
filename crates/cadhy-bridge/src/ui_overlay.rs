//! UI Overlay - Child WebViews for UI regions on top of wgpu main window
//!
//! This module implements the "inverted" architecture where:
//! - Main window = wgpu surface (receives native mouse events for viewport)
//! - Child webviews = UI regions only (titlebar, panels, status bar)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Main Window (raw, no webview)                   │
//! │  ┌─────────────────────────────────────────────────────────┐ │
//! │  │           wgpu Surface - Native Mouse Events            │ │
//! │  │                  (100+ FPS direct GPU)                  │ │
//! │  └─────────────────────────────────────────────────────────┘ │
//! │                                                              │
//! │  Child WebViews (overlay regions):                          │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │          Titlebar WebView (h=40px)                    │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! │  ┌──────┐                                    ┌───────────┐  │
//! │  │ Left │     ← viewport area →              │  Right    │  │
//! │  │ Panel│     (no webview here)              │  Panel    │  │
//! │  │ 280px│     mouse goes to wgpu             │  280px    │  │
//! │  └──────┘                                    └───────────┘  │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │          StatusBar WebView (h=24px)                   │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use std::sync::{Arc, RwLock};

use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Runtime,
    WebviewBuilder, WebviewUrl,
    window::WindowBuilder,
};

/// Configuration for the UI overlay layout
#[derive(Debug, Clone)]
pub struct OverlayLayout {
    /// Titlebar height (0 to disable)
    pub titlebar_height: u32,
    /// Left panel width (0 to disable)
    pub left_panel_width: u32,
    /// Right panel width (0 to disable)
    pub right_panel_width: u32,
    /// Status bar height (0 to disable)
    pub statusbar_height: u32,
    /// Whether left panel is collapsed
    pub left_collapsed: bool,
    /// Whether right panel is collapsed
    pub right_collapsed: bool,
}

impl Default for OverlayLayout {
    fn default() -> Self {
        Self {
            titlebar_height: 40,
            left_panel_width: 280,
            right_panel_width: 280,
            statusbar_height: 24,
            left_collapsed: false,
            right_collapsed: false,
        }
    }
}

/// Manages the UI overlay webviews
pub struct UiOverlay<R: Runtime> {
    /// The main window (raw Window with wgpu surface)
    main_window: tauri::Window<R>,
    /// Titlebar webview
    titlebar: Option<tauri::Webview<R>>,
    /// Left panel webview
    left_panel: Option<tauri::Webview<R>>,
    /// Right panel webview
    right_panel: Option<tauri::Webview<R>>,
    /// Status bar webview
    statusbar: Option<tauri::Webview<R>>,
    /// Current layout configuration
    layout: Arc<RwLock<OverlayLayout>>,
    /// Main window size
    window_size: Arc<RwLock<(u32, u32)>>,
}

impl<R: Runtime> UiOverlay<R> {
    /// Create a new UI overlay system
    ///
    /// This creates the main window (for wgpu) and child webviews (for UI regions).
    ///
    /// # Arguments
    /// * `app` - The Tauri app handle
    /// * `width` - Initial window width
    /// * `height` - Initial window height
    /// * `layout` - UI layout configuration
    pub fn new(
        app: &AppHandle<R>,
        width: u32,
        height: u32,
        layout: OverlayLayout,
    ) -> Result<Self, String> {
        tracing::info!(
            "Creating UI overlay system {}x{} with layout: {:?}",
            width, height, layout
        );

        // Create the main window (raw Window, no webview)
        // This will host the wgpu surface
        let main_window = WindowBuilder::new(app, "main")
            .title("CADHY")
            .inner_size(width as f64, height as f64)
            .min_inner_size(800.0, 600.0)
            .decorations(true)
            .transparent(false)  // No transparency needed - wgpu handles background
            .visible(true)
            .resizable(true)
            .build()
            .map_err(|e| format!("Failed to create main window: {}", e))?;

        tracing::info!("Main window created successfully");

        let layout = Arc::new(RwLock::new(layout));
        let window_size = Arc::new(RwLock::new((width, height)));

        let mut overlay = Self {
            main_window,
            titlebar: None,
            left_panel: None,
            right_panel: None,
            statusbar: None,
            layout,
            window_size,
        };

        // Create the UI webviews
        overlay.create_ui_webviews(app)?;

        Ok(overlay)
    }

    /// Create all UI webviews as children of the main window
    fn create_ui_webviews(&mut self, _app: &AppHandle<R>) -> Result<(), String> {
        let layout = self.layout.read().map_err(|e| e.to_string())?;
        let (width, height) = *self.window_size.read().map_err(|e| e.to_string())?;

        // Calculate positions based on layout
        let titlebar_height = layout.titlebar_height;
        let left_width = if layout.left_collapsed { 0 } else { layout.left_panel_width };
        let right_width = if layout.right_collapsed { 0 } else { layout.right_panel_width };
        let statusbar_height = layout.statusbar_height;

        // Create titlebar webview (full width at top)
        if titlebar_height > 0 {
            let titlebar = WebviewBuilder::new(
                "ui-titlebar",
                WebviewUrl::App("index.html?region=titlebar".into())
            )
                .transparent(true)
                .auto_resize();

            let titlebar_webview = self.main_window
                .add_child(
                    titlebar,
                    LogicalPosition::new(0.0, 0.0),
                    LogicalSize::new(width as f64, titlebar_height as f64),
                )
                .map_err(|e| format!("Failed to create titlebar webview: {}", e))?;

            tracing::info!("Titlebar webview created: {}x{}", width, titlebar_height);
            self.titlebar = Some(titlebar_webview);
        }

        // Create left panel webview
        if left_width > 0 {
            let panel_y = titlebar_height as f64;
            let panel_height = height as f64 - titlebar_height as f64 - statusbar_height as f64;

            let left = WebviewBuilder::new(
                "ui-left-panel",
                WebviewUrl::App("index.html?region=left-panel".into())
            )
                .transparent(true)
                .auto_resize();

            let left_webview = self.main_window
                .add_child(
                    left,
                    LogicalPosition::new(0.0, panel_y),
                    LogicalSize::new(left_width as f64, panel_height),
                )
                .map_err(|e| format!("Failed to create left panel webview: {}", e))?;

            tracing::info!("Left panel webview created: {}x{} at y={}", left_width, panel_height, panel_y);
            self.left_panel = Some(left_webview);
        }

        // Create right panel webview
        if right_width > 0 {
            let panel_x = width as f64 - right_width as f64;
            let panel_y = titlebar_height as f64;
            let panel_height = height as f64 - titlebar_height as f64 - statusbar_height as f64;

            let right = WebviewBuilder::new(
                "ui-right-panel",
                WebviewUrl::App("index.html?region=right-panel".into())
            )
                .transparent(true)
                .auto_resize();

            let right_webview = self.main_window
                .add_child(
                    right,
                    LogicalPosition::new(panel_x, panel_y),
                    LogicalSize::new(right_width as f64, panel_height),
                )
                .map_err(|e| format!("Failed to create right panel webview: {}", e))?;

            tracing::info!("Right panel webview created: {}x{} at x={}", right_width, panel_height, panel_x);
            self.right_panel = Some(right_webview);
        }

        // Create status bar webview (full width at bottom)
        if statusbar_height > 0 {
            let statusbar_y = height as f64 - statusbar_height as f64;

            let statusbar = WebviewBuilder::new(
                "ui-statusbar",
                WebviewUrl::App("index.html?region=statusbar".into())
            )
                .transparent(true)
                .auto_resize();

            let statusbar_webview = self.main_window
                .add_child(
                    statusbar,
                    LogicalPosition::new(0.0, statusbar_y),
                    LogicalSize::new(width as f64, statusbar_height as f64),
                )
                .map_err(|e| format!("Failed to create statusbar webview: {}", e))?;

            tracing::info!("Statusbar webview created: {}x{} at y={}", width, statusbar_height, statusbar_y);
            self.statusbar = Some(statusbar_webview);
        }

        Ok(())
    }

    /// Update webview positions when window is resized
    pub fn on_resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        // Update stored size
        if let Ok(mut size) = self.window_size.write() {
            *size = (width, height);
        }

        let layout = self.layout.read().map_err(|e| e.to_string())?;

        let titlebar_height = layout.titlebar_height;
        let left_width = if layout.left_collapsed { 0 } else { layout.left_panel_width };
        let right_width = if layout.right_collapsed { 0 } else { layout.right_panel_width };
        let statusbar_height = layout.statusbar_height;

        // Update titlebar
        if let Some(ref titlebar) = self.titlebar {
            let _ = titlebar.set_position(LogicalPosition::new(0.0, 0.0));
            let _ = titlebar.set_size(LogicalSize::new(width as f64, titlebar_height as f64));
        }

        // Update left panel
        if let Some(ref left) = self.left_panel {
            let panel_y = titlebar_height as f64;
            let panel_height = height as f64 - titlebar_height as f64 - statusbar_height as f64;
            let _ = left.set_position(LogicalPosition::new(0.0, panel_y));
            let _ = left.set_size(LogicalSize::new(left_width as f64, panel_height));
        }

        // Update right panel
        if let Some(ref right) = self.right_panel {
            let panel_x = width as f64 - right_width as f64;
            let panel_y = titlebar_height as f64;
            let panel_height = height as f64 - titlebar_height as f64 - statusbar_height as f64;
            let _ = right.set_position(LogicalPosition::new(panel_x, panel_y));
            let _ = right.set_size(LogicalSize::new(right_width as f64, panel_height));
        }

        // Update status bar
        if let Some(ref statusbar) = self.statusbar {
            let statusbar_y = height as f64 - statusbar_height as f64;
            let _ = statusbar.set_position(LogicalPosition::new(0.0, statusbar_y));
            let _ = statusbar.set_size(LogicalSize::new(width as f64, statusbar_height as f64));
        }

        Ok(())
    }

    /// Get the viewport bounds (area not covered by UI)
    pub fn get_viewport_bounds(&self) -> Result<ViewportBounds, String> {
        let layout = self.layout.read().map_err(|e| e.to_string())?;
        let (width, height) = *self.window_size.read().map_err(|e| e.to_string())?;

        let left_width = if layout.left_collapsed { 0 } else { layout.left_panel_width };
        let right_width = if layout.right_collapsed { 0 } else { layout.right_panel_width };

        Ok(ViewportBounds {
            x: left_width as i32,
            y: layout.titlebar_height as i32,
            width: width - left_width - right_width,
            height: height - layout.titlebar_height - layout.statusbar_height,
        })
    }

    /// Toggle left panel collapsed state
    pub fn toggle_left_panel(&mut self) -> Result<(), String> {
        {
            let mut layout = self.layout.write().map_err(|e| e.to_string())?;
            layout.left_collapsed = !layout.left_collapsed;
        }

        let (width, height) = *self.window_size.read().map_err(|e| e.to_string())?;
        self.on_resize(width, height)
    }

    /// Toggle right panel collapsed state
    pub fn toggle_right_panel(&mut self) -> Result<(), String> {
        {
            let mut layout = self.layout.write().map_err(|e| e.to_string())?;
            layout.right_collapsed = !layout.right_collapsed;
        }

        let (width, height) = *self.window_size.read().map_err(|e| e.to_string())?;
        self.on_resize(width, height)
    }

    /// Get reference to main window
    pub fn main_window(&self) -> &tauri::Window<R> {
        &self.main_window
    }

    /// Get current layout
    pub fn layout(&self) -> Arc<RwLock<OverlayLayout>> {
        self.layout.clone()
    }
}

/// Viewport bounds (area for wgpu rendering)
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ViewportBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
