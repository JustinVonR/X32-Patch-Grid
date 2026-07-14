mod x32_osc;

use tauri::{AppHandle, Manager, State};
use x32_osc::{ConnectionList, ConnectionManager};

struct AppData {
    osc_cons: ConnectionManager
}

#[tauri::command]
async fn scan(state: State<'_, AppData>) -> Result<ConnectionList, String> {
    state.osc_cons.scan().await;
    Ok(state.osc_cons.get_connection_list().await)
}

#[tauri::command]
async fn get_connections(state: State<'_, AppData>) -> Result<ConnectionList, String> {
    Ok(state.osc_cons.get_connection_list().await)
}

#[tauri::command]
async fn connect(_app: AppHandle, state: State<'_, AppData>, id: u32) -> Result<(), String> {
    state.osc_cons.connect(id)?;
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, AppData>) -> Result<(), String> {
    state.osc_cons.disconnect();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppData {
                osc_cons: ConnectionManager::new(),
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![scan, connect, disconnect, get_connections])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
