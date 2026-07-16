// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod x32_osc;

fn main() {
    x32_patch_grid_lib::run()
}

// TODO:
//  - Make the disconnect method of Connection actually disconnect the socket and tear itself down (Possibly implement as drop method?)
//  - Implement queue with notifications for sending outgoing messages
//  - Start on actual handling of connection event queues, channels, etc.
//  - Handle board event subscription and implement connect/disconnect messages to UI status indicator

