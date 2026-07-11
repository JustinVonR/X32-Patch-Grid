use std::sync::Mutex;
use tauri::{Manager, State};
use x32_osc::{ConnectionList, ConnectionManager};

struct AppData {
    osc_cons: ConnectionManager
}

#[tauri::command]
async fn scan(state: State<'_, Mutex<AppData>>) -> Result<ConnectionList, String> {
    match state.inner().lock() {
        Ok(mut app_data) => {
            app_data.osc_cons.scan();
            Ok(app_data.osc_cons.get_connection_list())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

#[tauri::command]
async fn get_connections(state: State<'_, Mutex<AppData>>) -> Result<ConnectionList, String> {
    match state.inner().lock() {
        Ok(app_data) => {
            Ok(app_data.osc_cons.get_connection_list())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

#[tauri::command]
fn connect(state: State<'_, Mutex<AppData>>, id: u32) -> Result<(), String> {
    match state.inner().lock() {
        Ok(mut app_data) => {
            app_data.osc_cons.connect(id)?;
            Ok(())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

#[tauri::command]
fn disconnect(state: State<'_, Mutex<AppData>>) -> Result<(), String> {
    match state.inner().lock() {
        Ok(mut app_data) => {
            app_data.osc_cons.disconnect();
            Ok(())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
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
