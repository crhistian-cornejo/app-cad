use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

            // Get main window and configure for macOS
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                use tauri::TitleBarStyle;
                let _ = window.set_title_bar_style(TitleBarStyle::Overlay);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
