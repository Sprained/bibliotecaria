use crate::commands::auth::get_saved_token;
use crate::commands::vault::{mirror_dir, out_dir};
use serde_json::json;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

const SHARED_PROMPT: &str = include_str!("../../../../prompts/shared.md");
const BIBLIOTECARIO_PROMPT: &str = include_str!("../../../../prompts/bibliotecario.md");

#[tauri::command]
pub async fn run_bibliotecario(app: AppHandle) -> Result<String, String> {
    let system_prompt = format!("{SHARED_PROMPT}\n\n{BIBLIOTECARIO_PROMPT}");
    run_agent(
        &app,
        &system_prompt,
        "Audite o vault e gere o relatório do bibliotecário.",
    )
    .await
}

fn vault_mcp_path() -> Result<PathBuf, String> {
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = current_exe
        .parent()
        .ok_or_else(|| "não achei o diretório do executável".to_string())?;
    let name = if cfg!(windows) {
        "vault_mcp.exe"
    } else {
        "vault_mcp"
    };
    Ok(dir.join(name))
}

fn build_mcp_config(vault_mcp_bin: &Path, mirror: &Path, out: &Path) -> String {
    json!({
        "mcpServers": {
            "vault-tools": {
                "command": vault_mcp_bin.to_string_lossy(),
                "args": [mirror.to_string_lossy(), out.to_string_lossy()]
            }
        }
    })
    .to_string()
}

async fn run_agent(app: &AppHandle, system_prompt: &str, prompt: &str) -> Result<String, String> {
    let token = get_saved_token()?;
    let mirror = mirror_dir(app)?;
    let out = out_dir(app)?;
    let vault_mcp_bin = vault_mcp_path()?;
    let mcp_config = build_mcp_config(&vault_mcp_bin, &mirror, &out);

    let shell = app.shell();
    let (mut rx, _child) = shell
        .sidecar("claude")
        .map_err(|e| e.to_string())?
        .env("CLAUDE_CODE_OAUTH_TOKEN", token)
        .args([
            "-p",
            prompt,
            "--system-prompt",
            system_prompt,
            "--mcp-config",
            &mcp_config,
            "--strict-mcp-config",
            "--tools",
            "",
            "--permission-mode",
            "bypassPermissions",
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    // Fecha o stdin do processo (EOF) sem matá-lo — sem isso o `claude -p`
    // espera ~3s por dados que nunca vêm, já que nunca escrevemos nada nele.
    drop(_child);

    let mut output = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                output.push_str(&String::from_utf8_lossy(&bytes));
            }
            CommandEvent::Stderr(bytes) => {
                eprintln!("[claude stderr] {}", String::from_utf8_lossy(&bytes));
            }
            _ => {}
        }
    }

    Ok(output.trim().to_string())
}
