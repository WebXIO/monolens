use tauri::Manager;

use crate::{
    connection_handler::connection_handler::ConnectionHandler,
    connector::repositories::{
        connector_repository::ConnectorRepository, file_repository::FileRepository,
    },
};

pub mod connection_handler;
pub mod connector;
pub mod database;
pub mod drivers;

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
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path")
                .join("salt.txt");

            let path = app.path().app_config_dir().unwrap();
            let repository = FileRepository::new(path);

            app.handle().plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;

            let state = create_app_state(repository);
            app.manage(state);
            app.manage(ConnectionHandler::new());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            database::commands::test_connection::test_connection,
            database::commands::get_databases::get_databases,
            database::commands::list_collections::list_collections,
            connector::commands::get_connections::get_connections,
            connector::commands::get_connection::get_connection,
            connector::commands::create_connection::create_connection,
            connector::commands::update_connection::update_connection,
            connector::commands::delete_connection::delete_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
