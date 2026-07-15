// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod x32_osc;

fn main() {
    x32_patch_grid_lib::run()
}

// TODO:
//  - Need to return to Error handling and figure out a consistent way of passing errors up the chain then converting to Strings for UI
//  - Continue working on creating socket and spawning async tasks in the new() method of Connection, should be hooked up to trigger with the
//    UI buttons already once completed. Remember to also create channels and queue notification system for outgoing OSC commands
//  - Move on to IO patch retrieval and parsing

