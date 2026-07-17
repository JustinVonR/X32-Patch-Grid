// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod x32_osc;

#[tokio::main]
async fn main() {
    // Set to tokio runtime for more async flexibility
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    x32_patch_grid_lib::run()
}

// TODO:
//  - Implement queue with notifications for sending outgoing messages
//  - Start on actual handling of connection event queues, channels, etc.
//  - Handle board event subscription and implement connect/disconnect messages to UI status indicator

