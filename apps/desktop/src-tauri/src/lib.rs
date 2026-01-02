//! CADHY Desktop - Tauri application with wgpu rendering
//!
//! Uses cadhy-bridge for rendering while we figure out native layer integration.

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,wgpu=warn")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(cadhy_bridge::init())
        .setup(|app| {
            println!("[CADHY] Application started");

            // Get main window - titlebar configuration is in tauri.conf.json
            let _window = app
                .get_webview_window("main")
                .expect("Main window not found");

            println!("[CADHY] Using cadhy-bridge offscreen renderer");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
