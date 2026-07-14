// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod x32_osc;

fn main() {
    x32_patch_grid_lib::run()
}
