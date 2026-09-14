use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_handler, tool_router, transport::stdio, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};

#[derive(Clone)]
struct VaultTools {
    mirror: PathBuf,
    out: PathBuf,
}

#[derive(Deserialize, JsonSchema)]
struct ReadMirrorArgs {
    path: String,
}

#[derive(Deserialize, JsonSchema)]
struct WriteOutArgs {
    path: String,
    content: String,
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

    #[tool(description = "Lê o conteúdo de uma nota do mirror, dado o caminho relativo (ex: subpasta/nota.md)")]
    fn read_mirror(&self, Parameters(args): Parameters<ReadMirrorArgs>) -> Result<String, String> {
        let full_path = resolve_within(&self.mirror, &args.path)?;
        std::fs::read_to_string(&full_path).map_err(|e| format!("erro ao ler {}: {e}", args.path))
    }

    #[tool(description = "Escreve o conteúdo de uma proposta na área de staging, dado o caminho relativo e o conteúdo")]
    fn write_out(&self, Parameters(args): Parameters<WriteOutArgs>) -> Result<String, String> {
        let full_path = resolve_within(&self.out, &args.path)?;
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("erro ao criar pasta pra {}: {e}", args.path))?;
        }
        std::fs::write(&full_path, &args.content)
            .map_err(|e| format!("erro ao escrever {}: {e}", args.path))?;
        Ok(format!("proposta salva em {}", args.path))
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

#[tool_handler(
    name = "vault-tools",
    version = "0.1.0",
    instructions = "Ferramentas escopadas pra ler o mirror do vault Obsidian e escrever propostas no staging"
)]
impl ServerHandler for VaultTools {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mirror = args
        .next()
        .expect("uso: vault_mcp <caminho-do-mirror> <caminho-do-out>");
    let out = args
        .next()
        .expect("uso: vault_mcp <caminho-do-mirror> <caminho-do-out>");

    let tools = VaultTools {
        mirror: mirror.into(),
        out: out.into(),
    };
    let service = tools.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
