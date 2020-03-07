#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri_sqlite::TauriSqlite;

fn main() {
  tauri::AppBuilder::new()
    .extension(TauriSqlite::new("storage.db".to_string(), true))
    .build()
    .run();
}
