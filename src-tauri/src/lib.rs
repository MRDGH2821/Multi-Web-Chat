mod layout;
mod prefs;
mod registry;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let _window = tauri::window::WindowBuilder::new(app, "main")
                .title("multi-web-chat")
                .inner_size(1280.0, 800.0)
                .build()?;

            // Chrome webview added in Task 5; placeholder title-only window for now.
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running multi-web-chat");
}
