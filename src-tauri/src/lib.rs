mod layout;
mod prefs;
mod registry;
mod shell;
mod status;

use shell::create_main_shell;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            create_main_shell(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running multi-web-chat");
}
