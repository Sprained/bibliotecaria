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

fn has_saved_token(service: &str, account: &str) -> bool {
    match Entry::new(service, account) {
        Ok(entry) => entry.get_password().is_ok(),
        Err(_) => false,
    }
}

#[tauri::command]
pub fn is_connected() -> bool {
    has_saved_token("bibliotecaria", "claude_code_oauth_token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_no_saved_token() {
        assert!(!has_saved_token("bibliotecaria-test", "chave-inexistente"));
    }

    #[test]
    fn detects_saved_token() {
        let entry = Entry::new("bibliotecaria-test", "chave-existente").unwrap();
        entry.set_password("valor-fake").unwrap();

        assert!(has_saved_token("bibliotecaria-test", "chave-existente"));

        entry.delete_credential().unwrap();
    }
}