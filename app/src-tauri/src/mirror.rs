use std::{fs, path::Path};

const IGNORED_NAME: &[&str] = &[".obsidian", ".git", ".trash"];

#[derive(serde::Serialize)]
pub struct StaticSync {
    pub notes: usize,
    pub bytes: u64,
}

pub fn synchronize(vault: &Path, mirror: &Path) -> Result<StaticSync, String> {
    if !vault.is_dir() {
        return Err(format!("Vault não encontrado: {}", vault.display()));
    }

    let vault_canon  = fs::canonicalize(vault).map_err(|e| e.to_string())?;
    let mirro_abs = if mirror.exists() {
        fs::canonicalize(mirror).map_err(|e| e.to_string())?
    } else {
        std::path::absolute(mirror).map_err(|e| e.to_string())?
    };

    if mirro_abs.starts_with(&vault_canon) {
        return Err("O mirror não pode estar dentro do vault".into());
    }

    if mirror.exists() {
        fs::remove_dir_all(mirror).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(mirror).map_err(|e| e.to_string())?;

    let mut stats = StaticSync { notes: 0, bytes: 0 };
    copy_folder(vault, mirror, &mut stats)?;

    Ok(stats)
}

fn should_ignore(name: &str) -> bool {
    IGNORED_NAME.contains(&name) || name.ends_with(".tmp")
}

fn copy_folder(origin: &Path, destination: &Path, stats: &mut StaticSync) -> Result<(), String> {
    for entry in fs::read_dir(origin).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if should_ignore(&name_str) {
            continue;
        }

        let source_path = entry.path();
        let destination_path = destination.join(&name);
        let file_type = entry.file_type().map_err(|e| e.to_string())?;

        if file_type.is_dir() {
            fs::create_dir_all(&destination_path).map_err(|e| e.to_string())?;
            copy_folder(&source_path, &destination_path, stats)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|e| e.to_string())?;
            let size = entry.metadata().map_err(|e| e.to_string())?.len();
            stats.bytes += size;
            if name_str.ends_with(".md") {
                stats.notes += 1;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

use crate::mirror::synchronize;

    #[test]
    fn synchronize_and_ignore_obsidian() {
        let test_dir = std::env::temp_dir().join("bibliotecaria_teste_mirror");
        let vault = test_dir.join("vault");
        let mirror = test_dir.join("mirror");
        let _ = fs::remove_dir_all(&test_dir);

        fs::create_dir_all(vault.join(".obsidian")).unwrap();
        fs::write(vault.join("nota.md"), "conteudo").unwrap();
        fs::write(vault.join(".obsidian").join("config"), "x").unwrap();

        let stats = synchronize(&vault, &mirror).unwrap();

        assert_eq!(stats.notes, 1);
        assert!(mirror.join("nota.md").exists());
        assert!(!mirror.join(".obsidian").exists());

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn reject_mirror_inside_vault() {
        let test_dir = std::env::temp_dir().join("bibliotecaria_teste_mirror_erro");
        let vault = test_dir.join("vault");
        let mirror = test_dir.join("mirror_invalido");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&mirror).unwrap();

        let results = synchronize(&vault, &mirror);
        assert!(results.is_err());

        fs::remove_dir_all(test_dir).unwrap();
    }
}