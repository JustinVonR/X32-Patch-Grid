mod x32_osc;

use tokio::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use x32_osc::{ConnectionList, ConnectionManager};

struct AppData {
    osc_cons: ConnectionManager
}

#[tauri::command]
async fn scan(state: State<'_, Mutex<AppData>>) -> Result<ConnectionList, String> {
    let mut app_data = state.lock().await;
    app_data.osc_cons.scan().await;
    Ok(app_data.osc_cons.get_connection_list())
}

#[tauri::command]
async fn get_connections(state: State<'_, Mutex<AppData>>) -> Result<ConnectionList, String> {
    let app_data = state.lock().await;
    Ok(app_data.osc_cons.get_connection_list())
}

#[tauri::command]
async fn connect(app: AppHandle, state: State<'_, Mutex<AppData>>, id: u32) -> Result<(), String> {
    let mut app_data = state.lock().await;
    app_data.osc_cons.connect(id)
}

#[tauri::command]
async fn disconnect(state: State<'_, Mutex<AppData>>) -> Result<(), String> {
    let mut app_data = state.lock().await;
    app_data.osc_cons.disconnect();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppData {
                osc_cons: ConnectionManager::new(),
            }));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![scan, connect, disconnect, get_connections])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
