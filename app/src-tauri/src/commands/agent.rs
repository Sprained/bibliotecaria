use crate::commands::auth::get_saved_token;
use crate::commands::vault::{get_vault_path, mirror_dir, out_dir, sync_now};
use crate::sessions::{self, SessionEvent};
use serde_json::json;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

const SHARED_PROMPT: &str = include_str!("../../../../prompts/shared.md");
const BIBLIOTECARIO_PROMPT: &str = include_str!("../../../../prompts/bibliotecario.md");
const ESCRIVAO_PROMPT: &str = include_str!("../../../../prompts/escrivao.md");

#[tauri::command]
pub async fn run_bibliotecario(app: AppHandle) -> Result<String, String> {
    sync_now(app.clone())?;

    let system_prompt = format!("{SHARED_PROMPT}\n\n{BIBLIOTECARIO_PROMPT}");
    let result = run_agent(
        &app,
        &system_prompt,
        "Audite o vault e gere o relatório do bibliotecário.",
    )
    .await;
    log_agent_run(&app, "bibliotecario", None, &result);
    result
}

#[tauri::command]
pub async fn run_escrivao(app: AppHandle, note_path: String) -> Result<String, String> {
    sync_now(app.clone())?;

    let vault_path =
        get_vault_path(app.clone()).ok_or_else(|| "nenhum vault configurado".to_string())?;
    let relative = relative_to_vault(&vault_path, &note_path)?;

    let system_prompt = format!("{SHARED_PROMPT}\n\n{ESCRIVAO_PROMPT}");
    let prompt = format!("Reescreva a nota `{relative}` seguindo o modo escrivão.");
    let result = run_agent(&app, &system_prompt, &prompt).await;
    log_agent_run(&app, "escrivao", Some(relative), &result);
    result
}

fn log_agent_run(
    app: &AppHandle,
    mode: &'static str,
    note: Option<String>,
    result: &Result<String, String>,
) {
    let (success, summary) = match result {
        Ok(text) => (true, text.clone()),
        Err(error) => (false, error.clone()),
    };
    let event = SessionEvent::AgentRun {
        mode,
        note,
        success,
        summary,
    };
    if let Err(e) = sessions::log_event(app, event) {
        eprintln!("[sessions.log] erro ao registrar evento: {e}");
    }
}

fn relative_to_vault(vault_path: &str, note_path: &str) -> Result<String, String> {
    let vault_abs = std::fs::canonicalize(vault_path).map_err(|e| e.to_string())?;
    let note_abs = std::fs::canonicalize(note_path).map_err(|e| e.to_string())?;

    let relative = note_abs.strip_prefix(&vault_abs).map_err(|_| {
        "A nota escolhida precisa estar dentro do vault selecionado.".to_string()
    })?;

    Ok(relative.to_string_lossy().to_string())
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
