mod commands;
mod mirror;
mod sessions;

use commands::agent::{run_bibliotecario, run_escrivao};
use commands::auth::{connect_claude, is_connected};
use commands::proposals::{discard_proposal, list_proposals, promote_proposal, read_proposal};
use commands::vault::{get_vault_path, set_vault_path, sync_now};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            connect_claude,
            is_connected,
            get_vault_path,
            set_vault_path,
            sync_now,
            run_bibliotecario,
            run_escrivao,
            list_proposals,
            read_proposal,
            promote_proposal,
            discard_proposal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
