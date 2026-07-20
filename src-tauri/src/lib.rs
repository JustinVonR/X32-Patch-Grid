mod x32_osc;

use rosc::{OscMessage};
use tauri::{AppHandle, Manager, State};
use x32_osc::ConnectionList;
use x32_osc::ConnectionManager;
use crate::x32_osc::X32OscMessage;

struct AppData {
    osc_cons: ConnectionManager
}

#[tauri::command]
async fn fetch(state: State<'_, AppData>, address: String) -> Result<(), String> {
    let osc_fetch = OscMessage {
        addr: address,
        args: vec![],
    };

    let send_msg = X32OscMessage::new(osc_fetch)?;

    let result = state.osc_cons.send_query(send_msg).await?;

    println!("{:?}", result.args);
    Ok(())
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
async fn connect(app: AppHandle, state: State<'_, AppData>, id: u32) -> Result<(), String> {
    state.osc_cons.connect(id, app).await?;
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, AppData>) -> Result<(), String> {
    state.osc_cons.disconnect().await;
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
        .invoke_handler(tauri::generate_handler![scan, connect, disconnect, get_connections, fetch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
