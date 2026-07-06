use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{Builder, Manager, State};
use x32_osc::{X32Console, ConnectionManager};

struct AppData {
    osc_cons: ConnectionManager
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn scan(state: State<'_, Mutex<AppData>>) -> Vec<X32Console> {
    println!("Scanning for boards!");
    // TODO: Change this to no longer use unwrap
    let results = state.inner().lock().unwrap().osc_cons.scan();
    results
}

#[tauri::command]
fn connect_console(id: usize) -> Result<(), String> {
    println!("Requested connection to {id}");
    thread::sleep(Duration::from_secs(5));
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
        .invoke_handler(tauri::generate_handler![greet, scan, connect_console])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
