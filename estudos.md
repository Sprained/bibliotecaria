# estudos.md — bibliotecária

Links pra estudar o caminho escolhido (Opção B: UI própria + Claude Agent SDK,
app de 2 botões, distribuição via Git). Coletados em set/2026.

> ⚠️ As docs da Anthropic trocam de path direto (`platform.claude.com` ↔
> `code.claude.com` ↔ `docs.claude.com`). Se der 404, procura pelo título.

---

## 1. Execução do agente — `claude` CLI + MCP em Rust (não Agent SDK)

- `claude --help` (flags confirmadas na sessão 2026-09-13, rodando local):
  `-p`/`--print` (non-interactive), `--mcp-config`, `--strict-mcp-config`,
  `--tools` (`""` desliga tudo), `--permission-mode` (inclui `dontAsk`),
  `--permission-prompts` (`host`/`none`), `--output-format stream-json`.
- **rmcp** — SDK oficial Rust do Model Context Protocol, macro `#[tool]`,
  transporte stdio pronto: https://github.com/modelcontextprotocol/rust-sdk
  / https://crates.io/crates/rmcp
- Guia prático rmcp + Claude Code: https://systemprompt.io/guides/build-mcp-server-rust
- Guia stdio MCP em Rust (Shuttle.dev): https://www.shuttle.dev/blog/2025/07/18/how-to-build-a-stdio-mcp-server-in-rust
- Doc oficial sobre MCP no Claude Code: https://code.claude.com/docs/en/agent-sdk/mcp

→ **SEGUIR:** `claude -p "<prompt>" --mcp-config <config.json>
--strict-mcp-config --tools "" --permission-mode dontAsk --output-format
stream-json`. `--tools ""` desliga toda tool nativa (Bash, Read, Write,
Edit...); o servidor MCP nosso (binário Rust separado, via `rmcp`) expõe só
`ler_mirror` / `listar_mirror` / `escrever_out`. Airgap duro sem precisar de
Node/Agent SDK — mata os críticos #2 e #3 do protótipo antigo (`legado-windows/`).
→ ⚠️ **Verificar na prática antes de confiar em produção**: como exatamente
`dontAsk` resolve uma chamada de tool MCP sem humano no loop (aprova
automático ou nega?) — testar isoladamente antes de destravar o resto.

## 2. Auth com a assinatura (sem API key)

- **Help center oficial**: https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan
- Claude Code — Authentication: https://code.claude.com/docs/en/authentication
- Histórico do pedido (issue #559): https://github.com/anthropics/claude-agent-sdk-python/issues/559
- Walkthrough Pro/Max + SDK: https://dev.to/aviv_shaked/how-to-use-your-claude-promax-subscription-with-the-agent-sdk-python-typescript-4emi

→ **Atualizado 2026-09-12 (reconferido nesta sessão):** o crédito mensal
separado ($20 Pro / $100 Max 5x / $200 Max 20x) foi anunciado pra entrar em
vigor em 15/jun/2026, mas foi **pausado no mesmo dia**, antes de valer. Texto
oficial do help center: *"We're pausing the changes to Claude Agent SDK
usage... For now, nothing has changed: Claude Agent SDK, `claude -p`, and
third-party app usage still draw from your subscription's usage limits."*
→ **O que isso significa pro projeto:** **não existe crédito separado hoje.**
Rodar a bibliotecária (Escrivão/Bibliotecário via Agent SDK) consome o **mesmo
limite de uso** da assinatura que qualquer sessão interativa do Claude Code —
inclusive a sessão usada pra codar o próprio projeto. Não muda a decisão de
auth via assinatura (continua sendo o único caminho sem API key avulsa e
billing à parte), só a expectativa de custo: não é uma pool extra "de graça".
→ ⚠️ **Reconferir esse help center antes de cada milestone** — já reverteu de
posição duas vezes (ban fev/2026 → anúncio de liberação pra jun/2026 → pausado
no próprio dia 15/jun/2026, antes de entrar em vigor).

## 3. Login do Claude Code de dentro do app (onboarding, passo 3)

- `claude setup-token` → `CLAUDE_CODE_OAUTH_TOKEN` (token OAuth de 1 ano):
  https://code.claude.com/docs/en/authentication
- Mesmo problema num VPS sem browser: https://codeongrass.com/blog/how-to-run-claude-code-on-a-remote-server/
- Issue de docs de auth headless: https://github.com/anthropics/claude-code/issues/7100

→ **SEGUIR:** o app dispara `claude setup-token`, abre a URL no browser,
captura o token e guarda no keychain do SO. Resolve o passo 3 do onboarding
sem terminal pro usuário.

## 4. Shell do desktop — Tauri vs Electron vs Wails (decidir no spike)

- Tauri v2 vs Electron, comparação honesta: https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026
- PkgPulse (bundle / RAM / segurança / fit de time): https://www.pkgpulse.com/guides/electron-vs-tauri-2026
- **Wails** vs Tauri/Electron (IPC, Go): https://medium.com/@tacherasasi/why-wails-wins-at-ipc-for-go-desktop-apps-and-how-it-stacks-up-against-tauri-electron-5a00b202cf09
- Panorama 2026 (Tauri/Electron/Wails/Deno): https://www.digitalapplied.com/blog/desktop-apps-web-stack-tauri-electron-deno-wails-2026

→ **Frame do spike:** **Wails** (Go — você manda bem, IPC sem boilerplate) vs
**Tauri** (Rust, ecossistema maior pra API nativa) vs **Electron** (Node
nativo, mais pesado ~180MB, mas o Agent SDK TS roda sem sidecar). Frontend web
em qualquer um — é inevitável na Opção B.

## 5. Bundlar o runtime (meta: usuário não instala nada)

- Tauri — Node.js como sidecar: https://v2.tauri.app/learn/sidecar-nodejs/
- Tauri — embedding external binaries: https://v2.tauri.app/develop/sidecar/
- **Node SEA** (single executable nativo, substitui o `pkg`): https://joyeecheung.github.io/blog/2026/01/26/improving-single-executable-application-building-for-node-js/
- Node SEA — guia de produção 2026: https://www.hirenodejs.com/blog/nodejs-single-executable-applications-2026

→ **SEGUIR:** se o shell for Tauri/Wails, o Claude Code + Agent SDK entram como
**sidecar**, com o Node compilado via **SEA** — não `pkg` (arquivado em 2024,
read-only, CVE aberta; mesma armadilha do FileBrowser). Se for Electron, o Node
já vem junto e esse problema some.

## 6. UI de diff / promover proposta (roda offline)

- CodeMirror 6 — merge view (`@codemirror/merge`): https://codemirror.net/docs/ref/#merge
- Monaco diff editor: https://microsoft.github.io/monaco-editor/
- Comparação (core ~300KB vs 5–10MB): https://www.pkgpulse.com/guides/monaco-editor-vs-codemirror-6-vs-sandpack-in-browser-2026

→ **SEGUIR: CodeMirror 6 + `@codemirror/merge`.** Leve, modular, feito pra ser
componente dentro de app maior (Prisma Studio, clients de SQL usam). Monaco é o
motor do VS Code — pesado demais pro que precisamos (ver original ao lado da
proposta, aceitar / descartar).

## 7. Least privilege / prompt injection (por que o airgap existe)

- OWASP — AI Agent Security Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
- OWASP — Prompt Injection: https://owasp.org/www-community/attacks/PromptInjection
- "Your AI, My Shell" (prompt injection em editores agênticos): https://arxiv.org/pdf/2509.22040

→ Fundamenta a decisão: tools escopadas em código + `dontAsk` + humano aprova a
escrita (promover) = o "least privilege + human approval for high-risk actions"
do OWASP, ao pé da letra.

## 8. Obsidian — MOC e convenções (contexto do modo bibliotecário)

- Nick Milo — Linking Your Thinking (origem do MOC): https://www.linkingyourthinking.com/
- Guia MOC completo (dsebastien): https://www.dsebastien.net/2022-05-15-maps-of-content/
- Obsidian Rocks — Maps of Content: https://obsidian.rocks/maps-of-content-effortless-organization-for-notes/

→ Base pro UC2/UC4 (nota órfã, MOC como camada de navegação, propriedade >
pasta). Já resumido no `CLAUDE.md` do agente; esses links são o porquê.

## 9. Auto-update do app (pós-MVP)

- Tauri updater plugin: https://v2.tauri.app/plugin/updater/
- Tauri v2 + GitHub Releases (tutorial): https://thatgurjot.com/til/tauri-auto-updater/

→ Distribuição é via GitHub Releases (decisão travada). O updater do Tauri lê
um `latest.json` publicado no release. Wails tem mecanismo próprio; Electron
usa `electron-updater`.

---

## Descartado (registrando o porquê)

- **Claude Agent SDK (TS/Python)** —
  https://platform.claude.com/docs/en/agent-sdk/overview e
  https://platform.claude.com/docs/en/agent-sdk/custom-tools — decisão
  2026-09-13: exigiria um processo Node/Python extra só pra hospedar as tools
  in-process (`@tool`/`create_sdk_mcp_server`). `claude` CLI puro (`-p`) +
  servidor MCP em Rust (`rmcp`) dá o mesmo airgap sem esse runtime extra, e
  mantém o backend 100% Rust (ver §1).
- **API Messages crua** — https://platform.claude.com/docs/en/api/messages —
  exige API key com billing à parte. Fora: queremos rodar na assinatura.
- **`pkg` da Vercel** — https://github.com/vercel/pkg — arquivado jan/2024,
  read-only, CVE aberta. Usar Node SEA.
- **Monaco pro diff** — pesado demais (motor do VS Code) pro escopo.
- **Opção A (terminal embutido)** — descartada por UX de leigo (prompt de
  permissão, digitar comando). Contexto no `task.md`.
- **Chat livre no MVP** — decisão de produto, volta na fase 2.

## Interessante pro projeto (fase 2 / inspiração)

- **foam-notes-mcp** — https://github.com/fbdo/foam-notes-mcp — MCP local pra
  vault markdown com 3 camadas de busca: ripgrep (keyword) + grafo de wikilink
  com PageRank + **busca semântica offline (Transformers.js + sqlite-vec)**. É o
  UC5 praticamente pronto como referência.
- **sqlite-vec** — https://github.com/asg017/sqlite-vec — vector search dentro
  do SQLite, sem servidor. Casa com self-hosted + review offline.
- **sqlite-memory** — https://github.com/sqliteai/sqlite-memory — memória de
  agente em markdown, chunking markdown-aware + embedding local via llama.cpp.
- **Agent SDK TS V2 preview** — https://platform.claude.com/docs/en/agent-sdk/typescript-v2-preview
  — tira o inferno de async generator; multi-turn vira `send()`/`stream()` por
  turno. Mais simples pro app de 2 botões, se for de TS.
- **Promptfoo — provider do Agent SDK** — https://www.promptfoo.dev/docs/providers/claude-agent-sdk/
  — dava pra testar o escrivão como eval ("não inventou conteúdo?", "manteve a
  voz?") em vez de conferir na mão.
