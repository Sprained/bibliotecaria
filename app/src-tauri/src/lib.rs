// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use keyring::Entry;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn conectar_claude(app: tauri::AppHandle) -> Result<(), String> {
    let shell = app.shell();
    let (mut rx, _child) = shell
        .sidecar("claude")
        .map_err(|e| e.to_string())?
        .args(["setup-token"])
        .spawn()
        .map_err(|e| e.to_string())?;

    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(bytes) = event {
            let linha = String::from_utf8_lossy(&bytes);
            if let Some(inicio) = linha.find("sk-ant-oat01-") {
                let token = linha[inicio..].trim();
                let entry = Entry::new("bibliotecaria", "claude_code_oauth_token")
                    .map_err(|e| e.to_string())?;
                entry.set_password(token).map_err(|e| e.to_string())?;
                return Ok(());
            }
        }
    }

    Err("processo encerrado sem token".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, conectar_claude])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
