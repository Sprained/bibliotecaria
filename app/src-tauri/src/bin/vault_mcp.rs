use rmcp::{tool, tool_handler, tool_router, transport::stdio, ServerHandler, ServiceExt};
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct VaultTools {
    mirror: PathBuf,
}

#[tool_router]
impl VaultTools {
    #[tool(description = "Lista os arquivos .md disponíveis no mirror do vault")]
    fn list_mirror(&self) -> String {
        let files = collect_md(&self.mirror, &self.mirror);
        if files.is_empty() {
            "(nenhuma nota encontrada)".into()
        } else {
            files.join("\n")
        }
    }
}

fn collect_md(base: &Path, current: &Path) -> Vec<String> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(current) else {
        return result;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_md(base, &path));
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(relative) = path.strip_prefix(base) {
                result.push(relative.display().to_string());
            }
        }
    }

    result
}

#[tool_handler(
    name = "vault-tools",
    version = "0.1.0",
    instructions = "Ferramentas escopadas pra ler o mirror do vault Obsidian"
)]
impl ServerHandler for VaultTools {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mirror = std::env::args()
        .nth(1)
        .expect("uso: vault_mcp <caminho-do-mirror>");

    let tools = VaultTools {
        mirror: mirror.into(),
    };
    let service = tools.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
