use tauri::Manager;

use crate::{
    connection_handler::connection_handler::ConnectionHandler,
    connector::repositories::{
        connector_repository::ConnectorRepository, file_repository::FileRepository,
    },
    connector::repositories::credentials::credential_service::CredentialService
};
use std::sync::Arc;

use crate::task_manager::task_manager::TaskManager;

pub mod connection_handler;
pub mod connector;
pub mod database;
pub mod drivers;
pub mod menu;
pub mod task_manager;

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
    let mut builder = tauri::Builder::default();

    if cfg!(target_os = "macos") {
        builder = builder.menu(menu::macos_menu::<tauri::Wry>);
    }

    builder
        .plugin(
            tauri_plugin_log::Builder::new()
                // debug output stays in development builds
                .level(if cfg!(debug_assertions) {
                    tauri_plugin_log::log::LevelFilter::Debug
                } else {
                    tauri_plugin_log::log::LevelFilter::Info
                })
                .build(),
        )
        .setup(|app| {
            let path = app.path().app_config_dir().unwrap();

            log::debug!("Load config from path {}", path.display());

            let repository = FileRepository::new(path, Arc::new(CredentialService::new("monolens-connections")));

            let state = create_app_state(repository);
            app.manage(state);
            app.manage(ConnectionHandler::new());
            app.manage(TaskManager::new());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            database::commands::start_test_connection::start_test_connection,
            database::commands::start_connect::start_connect,
            database::commands::disconnect::disconnect,
            database::commands::list_collections::list_collections,
            database::commands::start_find_documents::start_find_documents,
            connector::commands::get_connections::get_connections,
            connector::commands::get_connection_password::get_connection_password,
            connector::commands::create_connection::create_connection,
            connector::commands::update_connection::update_connection,
            connector::commands::delete_connection::delete_connection,
            task_manager::commands::cancel_task,
            task_manager::commands::await_task_result,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
