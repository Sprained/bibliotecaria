use keyring::Entry;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[tauri::command]
pub async fn connect_claude(app: tauri::AppHandle) -> Result<(), String> {
    let shell = app.shell();
    let (mut rx, _child) = shell
        .sidecar("claude")
        .map_err(|e| e.to_string())?
        .args(["setup-token"])
        .spawn()
        .map_err(|e| e.to_string())?;

    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(bytes) = event {
            let row = String::from_utf8_lossy(&bytes);
            if let Some(start) = row.find("sk-ant-oat01-") {
                let token = row[start..].trim();
                let entry = Entry::new("bibliotecaria", "claude_code_oauth_token")
                    .map_err(|e| e.to_string())?;
                entry.set_password(token).map_err(|e| e.to_string())?;
                return Ok(());
            }
        }
    }

    Err("processo encerrado sem token".into())
}