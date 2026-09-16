use crate::commands::vault::{get_vault_path, out_dir, sync_now};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;
use tauri::AppHandle;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalSummary {
    pub path: String,
    pub is_substitution: bool,
    pub modified: u64,
}

#[derive(Serialize)]
pub struct ProposalDetail {
    pub proposal: String,
    pub original: Option<String>,
}

#[tauri::command]
pub fn list_proposals(app: AppHandle) -> Result<Vec<ProposalSummary>, String> {
    let out = out_dir(&app)?;
    let vault_path = get_vault_path(app.clone());

    let mut proposals = Vec::new();
    collect_proposals(&out, &out, vault_path.as_deref(), &mut proposals)?;
    proposals.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(proposals)
}

#[tauri::command]
pub fn read_proposal(app: AppHandle, path: String) -> Result<ProposalDetail, String> {
    let out = out_dir(&app)?;
    let proposal_path = resolve_within(&out, &path)?;
    let proposal = std::fs::read_to_string(&proposal_path).map_err(|e| e.to_string())?;

    let original = get_vault_path(app)
        .map(|vault_path| Path::new(&vault_path).join(&path))
        .filter(|p| p.is_file())
        .map(std::fs::read_to_string)
        .transpose()
        .map_err(|e| e.to_string())?;

    Ok(ProposalDetail { proposal, original })
}

#[tauri::command]
pub fn promote_proposal(app: AppHandle, path: String) -> Result<(), String> {
    let out = out_dir(&app)?;
    let proposal_path = resolve_within(&out, &path)?;
    let content = std::fs::read_to_string(&proposal_path).map_err(|e| e.to_string())?;

    let vault_path =
        get_vault_path(app.clone()).ok_or_else(|| "nenhum vault configurado".to_string())?;
    let destination = Path::new(&vault_path).join(&path);
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&destination, content).map_err(|e| e.to_string())?;

    sync_now(app.clone())?;

    std::fs::remove_file(&proposal_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn discard_proposal(app: AppHandle, path: String) -> Result<(), String> {
    let out = out_dir(&app)?;
    let proposal_path = resolve_within(&out, &path)?;
    std::fs::remove_file(&proposal_path).map_err(|e| e.to_string())
}

fn collect_proposals(
    base: &Path,
    current: &Path,
    vault_path: Option<&str>,
    result: &mut Vec<ProposalSummary>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(current).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| e.to_string())?;

        if file_type.is_dir() {
            collect_proposals(base, &path, vault_path, result)?;
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let relative = path
                .strip_prefix(base)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string();
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .map_err(|e| e.to_string())?
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs();
            let is_substitution = vault_path
                .map(|v| Path::new(v).join(&relative).is_file())
                .unwrap_or(false);

            result.push(ProposalSummary {
                path: relative,
                is_substitution,
                modified,
            });
        }
    }
    Ok(())
}

fn resolve_within(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative);
    let is_safe = relative_path
        .components()
        .all(|c| matches!(c, Component::Normal(_)));

    if !is_safe {
        return Err(format!("caminho fora da área permitida: {relative}"));
    }

    let root_abs =
        std::fs::canonicalize(root).map_err(|e| format!("erro no diretório base: {e}"))?;
    Ok(root_abs.join(relative_path))
}

#[cfg(test)]
mod tests {
    use super::{collect_proposals, resolve_within};
    use std::fs;

    #[test]
    fn detects_substitution_vs_relatorio() {
        let test_dir = std::env::temp_dir().join("bibliotecaria_teste_proposals");
        let vault = test_dir.join("vault");
        let out = test_dir.join("out");
        let _ = fs::remove_dir_all(&test_dir);

        fs::create_dir_all(&vault).unwrap();
        fs::create_dir_all(out.join("subpasta")).unwrap();
        fs::write(vault.join("nota.md"), "conteudo original").unwrap();
        fs::write(out.join("nota.md"), "conteudo reescrito").unwrap();
        fs::write(out.join("subpasta/relatorio.md"), "auditoria").unwrap();

        let vault_str = vault.to_string_lossy().to_string();
        let mut proposals = Vec::new();
        collect_proposals(&out, &out, Some(&vault_str), &mut proposals).unwrap();
        proposals.sort_by(|a, b| a.path.cmp(&b.path));

        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[0].path, "nota.md");
        assert!(proposals[0].is_substitution);
        assert_eq!(proposals[1].path, "subpasta/relatorio.md");
        assert!(!proposals[1].is_substitution);

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn rejects_path_traversal() {
        let test_dir = std::env::temp_dir().join("bibliotecaria_teste_proposals_traversal");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let result = resolve_within(&test_dir, "../fora.md");
        assert!(result.is_err());

        fs::remove_dir_all(test_dir).unwrap();
    }
}
