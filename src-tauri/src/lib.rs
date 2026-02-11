use tauri::Manager;

use crate::{connection_handler::connection_handler::ConnectionHandler, connector::repositories::{connector_repository::ConnectorRepository, file_repository::FileRepository}};

pub mod connector;
pub mod drivers;
pub mod connection_handler;
pub mod database;

pub struct AppState {
    pub connection_repository: Box<dyn ConnectorRepository>,
}

pub fn create_app_state(repository: impl ConnectorRepository + 'static) -> AppState {
    AppState {
        connection_repository: Box::new(repository),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = app.path().app_config_dir().unwrap();
            
            let repository = FileRepository::new(path);
            
            let state = create_app_state(repository);
            app.manage(state);
            app.manage(ConnectionHandler::new());
            
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            database::commands::test_connection::test_connection,
            database::commands::get_databases::get_databases,
            connector::commands::get_connections::get_connections,
            connector::commands::get_connection::get_connection,
            connector::commands::create_connection::create_connection,
            connector::commands::update_connection::update_connection,
            connector::commands::delete_connection::delete_connection,
            connector::commands::save_connections::save_connections,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
