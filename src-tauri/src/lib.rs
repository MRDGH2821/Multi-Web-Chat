mod adapter_runtime;
mod commands;
mod layout;
mod prefs;
mod registry;
mod shell;
mod status;

use commands::{get_prefs, get_providers, new_chat_all, send_prompt, set_provider_enabled};
use shell::create_main_shell;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_providers,
            get_prefs,
            set_provider_enabled,
            send_prompt,
            new_chat_all
        ])
        .setup(|app| {
            create_main_shell(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running multi-web-chat");
}
