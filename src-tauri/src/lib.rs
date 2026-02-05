use tauri::Manager;

use crate::connector::repositories::{connector_repository::ConnectorRepository, file_repository::FileRepository};

pub mod connector;
pub mod drivers;

pub struct AppState {
    pub connection_repository: Box<dyn ConnectorRepository>,
}

pub fn create_app_state(repository: impl ConnectorRepository + 'static) -> AppState {
    AppState {
        connection_repository: Box::new(repository),
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = app.path().app_config_dir().unwrap();
            
            let repository = FileRepository::new(path);
            
            let state = create_app_state(repository);
            app.manage(state);
            
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
