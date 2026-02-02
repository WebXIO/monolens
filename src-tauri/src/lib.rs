use mongodb::results::DatabaseSpecification;

use crate::connector::connector::Connection;

pub mod connector;



#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_databases() -> Vec<DatabaseSpecification> {
    let connector = Connection::new("mongodb://root:rootpass@localhost:27017");

    return connector.get_databases().await;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_databases])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
