use crate::mirror::{self, StaticSync};
use serde_json::json;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

const CONFIG_STORE: &str = "config.json";
const VAULT_PATH_KEY: &str = "vault_path";
const MIRROR_DIR_NAME: &str = "mirror";
const OUT_DIR_NAME: &str = "out";

#[tauri::command]
pub fn get_vault_path(app: AppHandle) -> Option<String> {
    let store = app.store(CONFIG_STORE).ok()?;
    store.get(VAULT_PATH_KEY)?.as_str().map(str::to_string)
}

#[tauri::command]
pub fn set_vault_path(app: AppHandle, vault_path: String) -> Result<StaticSync, String> {
    let store = app.store(CONFIG_STORE).map_err(|e| e.to_string())?;
    store.set(VAULT_PATH_KEY, json!(vault_path));
    store.save().map_err(|e| e.to_string())?;

    run_sync(&app, &vault_path)
}

#[tauri::command]
pub fn sync_now(app: AppHandle) -> Result<StaticSync, String> {
    let vault_path =
        get_vault_path(app.clone()).ok_or_else(|| "nenhum vault configurado".to_string())?;
    run_sync(&app, &vault_path)
}

pub fn mirror_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(MIRROR_DIR_NAME))
}

pub fn out_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(OUT_DIR_NAME);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn run_sync(app: &AppHandle, vault_path: &str) -> Result<StaticSync, String> {
    mirror::synchronize(Path::new(vault_path), &mirror_dir(app)?)
}
