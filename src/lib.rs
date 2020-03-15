#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use rusqlite::{params, Connection, NO_PARAMS};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
enum TauriSqliteCmd {
  SetItemSqlite { key: String, value: String, callback: String, error: String },
  GetItemSqlite { key: String, callback: String, error: String },
}

pub struct TauriSqlite {
  storage_enabled: bool,
  connection: Connection,
}

impl TauriSqlite {
  pub fn new (db_name: String, storage_enabled: bool) -> Self {
    let connection = Connection::open(db_name).expect("failed to open sqlite connection");

    if storage_enabled {
      connection.execute(
        "CREATE TABLE IF NOT EXISTS tauri_storage (
          key TEXT NOT NULL UNIQUE,
          value TEXT
        )",
        NO_PARAMS,
      ).expect("failed to create storage table");
    }

    Self {
      storage_enabled,
      connection
    }
  }
}

impl tauri::plugin::Plugin for TauriSqlite {
  fn extend_api(&self, webview: &mut tauri::WebView<'_, ()>, payload: &str) -> Result<bool, String> {
    use TauriSqliteCmd::*;
    if self.storage_enabled {
      match serde_json::from_str(payload) {
        Err(e) => {
          Err(e.to_string())
        }
        Ok(command) => {
          match command {
            SetItemSqlite { key, value, callback, error } => {
              let result = self.connection.execute("INSERT OR REPLACE INTO tauri_storage VALUES (?1, ?2)", params![key, value])
                .and_then(|_| { Ok("{}".to_string()) })
                .map_err(|_| { tauri::Error::from("failed to insert data") });
              tauri::execute_promise(
                webview,
                move || {
                  result
                },
                callback,
                error,
              );
            }
            GetItemSqlite { key, callback, error } => {
              let mut query = self.connection.prepare("SELECT value FROM tauri_storage WHERE key = ?1").expect("failed to prepare statement");
              let results = query
                .query_and_then(params![key], |row| row.get(0))
                .expect("")
                .collect::<Vec<rusqlite::Result<String>>>();

              let result = match results.first() {
                Some(result) => Ok(format!("{{ value: '{}' }}", result.as_ref().expect("failed to read item"))),
                None => Err(tauri::Error::from("key not found"))
              };
              tauri::execute_promise(
                webview,
                move || {
                  result
                },
                callback,
                error,
              )
            }
          }
          Ok(true)
        }
      }
    } else {
      Ok(false)
    }
  }
}
