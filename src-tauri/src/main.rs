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
//  - Check validity of OSC paths, build out more specific rules for doing this including needing exact paths for queries
//    to get a response (one specific discovered rule is to not allow / at the end of a path)
//  - Review error handling of all connection errors, maybe close connection instead of continuing?
//  - Add unit testing of parsing / validation functions, maybe doesn't make sense for network logic or tauri commands / async
//  - Refactor and clean up before moving on to UI and Input/Output mapping logic

