# CLAUDE.md — bibliotecária

Ferramenta desktop pra cuidar de um vault Obsidian com ajuda do Claude, sem deixar
o agente escrever direto nas notas. Todo trabalho do agente sai num staging; o
usuário revisa (com diff) e só ele promove pra nota de verdade.

## Como chegamos aqui

Nasceu como script de terminal (`lib.py` + Claude Code CLI) dentro do repo do
homelab, testado no Windows. Validou o conceito central — mirror somente-leitura,
staging de propostas, promoção manual — e também expôs limites (airgap "soft",
proposta que nunca morre, terminal inviável pra quem não é técnico). Virou repo
próprio quando o objetivo mudou de "ferramenta pessoal" pra "app pra compartilhar
com amigos não-técnicos".

## O que é (produto)

- Cada pessoa instala e usa **com a própria conta Claude** — mono-usuário por
  instalação, sem multi-tenant.
- O vault sincroniza sozinho (ex: Obsidian LiveSync); a bibliotecária não
  sincroniza nada — só produz propostas, que viajam junto com o vault.
- **Painel de revisão roda offline.** Só a sessão do agente (Escrivão /
  Bibliotecário) precisa de rede.

## Decisões travadas

| Data | Decisão | Porquê |
|---|---|---|
| 2026-09-09 | **Opção B**: UI própria hospeda a conversa via Claude Agent SDK, não terminal | UX pra leigo (botão, sem prompt de permissão); tools escopadas em código dão **airgap duro** — agente não tem path nem bash livre, só `ler_mirror` / `listar_mirror` / `escrever_out`. Custo: mais frontend, aceito (sem prazo) |
| 2026-09-09 | MVP = **app de 2 botões**, sem chat livre | Escrivão (por nota) e Bibliotecário (auditoria) cobrem os use cases centrais sem exigir UI de chat. Chat livre = fase 2 |
| 2026-09-09 | Distribuição via **GitHub Releases** | — |
| 2026-09-09 | Onboarding: **usuário não pré-instala nada** | runtime (Node/Claude Code) embutido no app via sidecar, não instalado no sistema — menos elevação/antivírus/PATH pra debugar remoto |
| 2026-09-09 | **UC5 (busca semântica) → fase 2** | embedding + índice + modelo local offline dobra o escopo do MVP |
| 2026-09-09 | **UC3 (multi-máquina) é opcional** | quem usa numa máquina só não é forçado ao modelo de sync |
| 2026-09-11 | Auth via **assinatura do usuário** (Agent SDK), não API key | Único caminho sem API key/billing à parte. **Correção 2026-09-12**: o crédito mensal separado ($20/$100/$200) foi anunciado pra 15/jun/2026 mas **pausado no mesmo dia**, antes de entrar em vigor — hoje o uso do Agent SDK/apps de terceiro conta no **mesmo limite** de uso do plano, mesma pool do Claude Code interativo (o que já uso pra codar esse projeto). Ver `estudos.md` §2 |
| 2026-09-11 | Shell do app: **Tauri** (não Wails/Electron) | Sidecar de binário externo é feature oficial e documentada no Tauri (`externalBin`), enquanto no Wails é discussion aberta sem solução — risco direto pro passo 1 do spike (embutir Node). Wails v3 também ainda em beta com gates bloqueantes pra GA, e mantido por sponsors independentes (bus factor baixo) vs. Tauri com org/foundation por trás. Custo aceito: Rust fora da zona de conforto do Gabs |
| 2026-09-12 | Identidade visual: **"biblioteca moderna"** — vinho/borgonha + creme, tema claro e escuro com peso igual | Foge do SaaS genérico (produto pra amigos, não empresa). Tipografia serifada (títulos) + sans (corpo/UI); vinho como cor de marca constante nos dois temas, ajustando luminosidade pra manter contraste em vez de dark mode forçado. Ver seção "Identidade visual" abaixo |
| 2026-09-13 | Backend do agente: **`claude` CLI puro (`-p`) + servidor MCP nosso em Rust (`rmcp`)**, não Agent SDK (TS/Python) | `--mcp-config` + `--strict-mcp-config` + `--tools ""` + `--permission-mode bypassPermissions` dá o mesmo airgap duro (zero tools nativas, só as 3 registradas) sem precisar de um processo Node hospedando o SDK. Mantém o backend 100% Rust — bate com o objetivo do Gabs de praticar Rust, e elimina de vez a necessidade de Node SEA em qualquer parte do app |
| 2026-09-14 | Flag de permissão do `claude -p`: **`bypassPermissions`**, não `dontAsk` | Testado na prática: `dontAsk` **nega** silenciosamente qualquer tool que precisaria de aprovação (não tem terminal pra perguntar em modo headless, então a resposta segura é negar). `bypassPermissions` de fato auto-aprova. Não reabre o airgap: `--tools ""` + `--strict-mcp-config` já limitam o universo de tools às 3 nossas — não existe Bash/Read nativo pra "bypassar". Validado chamando `list_mirror` de verdade via `vault_mcp` sob `claude -p`. Ver `estudos.md` §1 |

## Use cases

| # | Job | Modo | MVP? |
|---|---|---|---|
| UC1 | braindump zoado → versão legível | Escrivão (por nota) | sim |
| UC2 | auditar vault (órfã, duplicata, link quebrado, nota inchada) → relatório | Bibliotecário (semanal por ora) | sim |
| UC3 | aceitar/rejeitar propostas de outra máquina | — | opcional |
| UC4 | criar/atualizar MOC (`A00`) da pasta | ação dirigida | pós-MVP |
| UC5 | "onde falei sobre X" (busca semântica) | — | fase 2 |
| UC6 | quebrar nota grande em duas | ação dirigida | pós-MVP |
| UC7 | preencher frontmatter em lote | — | pós-MVP |

- Escrivão roda **por nota** (usuário aponta o arquivo).
- Bibliotecário pode fazer **handoff** pro Escrivão ("essa órfã tá confusa" → job
  de substituição), sempre com aprovação do usuário.

## Ciclo de vida de uma proposta

3 tipos de saída do agente:

| Tipo | Quem gera | Mapeia pra | Fim de vida |
|---|---|---|---|
| Substituição | Escrivão | nota existente | diff → sobrescreve a nota real, ou descarta |
| Nota nova | Bibliotecário (MOC, split) | arquivo inexistente | cria no vault, ou descarta |
| Relatório | Bibliotecário (auditoria) | nada (documento solto) | lê → pode virar job de Escrivão → arquiva/apaga |

- Estados: `rascunho` → `em revisão` → `aceita` / `rejeitada` / `adiada`.
- **Staging é outbox, não histórico.** `aceita` e `rejeitada` removem a proposta e
  essa remoção **propaga** — nunca fica resíduo que ressuscita. (No protótipo
  antigo isso não acontecia — era o bug da "proposta-zumbi", ver `legado-windows/`.)
- Promover = a UI escreve na nota real, **só sob comando explícito, depois do
  diff**. Único caminho de escrita no vault.

## Arquitetura

- **Execução do agente**: `claude` CLI em modo `-p` (print/non-interactive),
  não Agent SDK. Tools escopadas (`ler_mirror`, `listar_mirror`,
  `escrever_out`) viram um **servidor MCP em Rust** (crate `rmcp`), registrado
  via `--mcp-config` + `--strict-mcp-config`. `--tools ""` desliga toda tool
  nativa (Bash, Read, Write, Edit...); `--permission-mode bypassPermissions`
  aprova automático as tools que existem — só as 3 nossas, já que não há
  Bash/Read nativo pra bypassar. Zero Node em qualquer parte do app, backend
  100% Rust. Ver `estudos.md` §1.
- **Prompt de cada modo**: hoje escrivão e bibliotecário vivem juntos em
  `prompts/vault-agent.md` (herdado do protótipo). **A decidir**: separar em dois
  arquivos/system-prompts quando a UI virar "2 botões", ou manter um com seleção
  de modo por parâmetro.
- **Auth**: `claude setup-token` disparado pelo app → token OAuth de 1 ano
  guardado no keychain do SO. Consome o limite de uso normal da assinatura do
  usuário, mesma pool do Claude Code interativo. Ver `estudos.md` §2 e §3.
- **Shell do app**: **Tauri**. Ver `estudos.md` §4.
- **Runtime embutido**: só o binário `claude` (nativo, autocontido) como
  *sidecar* do Tauri (`externalBin`) — validado no spike. Sem Node/SEA em
  nenhuma parte do app.
- **UI de diff/promover**: CodeMirror 6 + `@codemirror/merge`, roda 100% offline.
  Ver `estudos.md` §6.
- **Distribuição/update**: GitHub Releases; updater nativo do shell escolhido.
  Ver `estudos.md` §9.

## Identidade visual

Direção "biblioteca moderna" — vinho/borgonha + creme, tema claro e escuro
com peso igual (não é dark mode forçado, os dois são pensados juntos).

- **Tipografia**: `Newsreader` (serifada, títulos) + `Archivo` (sans, corpo/UI),
  ambas Google Fonts.
- **Cor de marca**: vinho (matiz ~19 em oklch) constante nos dois temas — mais
  escuro/saturado no claro (`oklch(34% 0.135 19)`), mais claro no escuro
  (`oklch(62% 0.15 19)`) pra manter contraste em vez de usar o mesmo valor nos
  dois. Dourado (matiz ~75-78) como accent secundário (badge do Bibliotecário,
  indicador de "conectado").
- **Ícones**: SVG desenhados na mão (sem emoji/dingbat) — pena pro Escrivão,
  estante+lupa pro Bibliotecário, livro aberto como marca/wordmark.
- **Mockup de referência**: [artefato publicado](https://claude.ai/code/artifact/9b6afbef-9c9a-4142-9c3a-5302d280ed00) —
  4 artboards (onboarding e home, claro e escuro). Fonte dos tokens de cor
  exatos pra quando for implementar o CSS de verdade.

## Estado atual

- **Spike de onboarding resolvido tecnicamente** (nesse Mac, `aarch64-apple-darwin`):
  app Tauri em `app/` prova que dá pra rodar o `claude` como sidecar embutido
  (sem o usuário instalar nada), disparar `claude setup-token` de dentro do
  app, capturar o token sem ele passar pela tela, e gravar no Keychain do
  macOS. Falta multi-plataforma (Windows/Linux) e o binário de distribuição de
  verdade (via optional dependency do Agent SDK, não cópia manual) — fica pra
  fase de empacotamento/distribuição.
- **Sincronização do mirror e escolha de vault resolvidas**: `mirror::synchronize`
  (testado) agora é chamado por comandos Tauri reais (`get_vault_path`/
  `set_vault_path`/`sync_now`), path do vault persistido via
  `tauri-plugin-store`, escolha de pasta via `tauri-plugin-dialog` no
  onboarding. Falta UI pra trocar de vault depois de escolhido (ver críticos
  herdados, item 4).
- **Servidor MCP em Rust com as 3 tools prontas e testadas** (`list_mirror`,
  `read_mirror`, `write_out`), validado rodando de verdade sob `claude -p`
  com `--mcp-config` + `--strict-mcp-config` + `--tools ""` +
  `--permission-mode bypassPermissions`. Falta plugar isso no fluxo real do
  app (hoje só é chamado manualmente via `scripts/test-mcp.sh` ou `claude -p`
  direto no terminal).
- Fora isso, o que existe é o protótipo de terminal (Opção A), testado no
  Windows, guardado em `legado-windows/` como referência de comportamento —
  não é pra evoluir, é pra não perder o que já foi validado (as regras do
  escrivão/bibliotecário, a estrutura mirror/out/staging).
- Próximo passo real de código: **MVP** (app de 2 botões, chamando o
  servidor MCP via `claude -p`, painel de diff/promover offline) — ver
  pendências abaixo.

## Pendências / ordem de trabalho

1. ~~**Spike de onboarding**~~ — resolvido. Achado importante: `claude` já é
   um binário nativo autocontido (não precisa de Node/SEA pra essa parte) — o
   Agent SDK usa ele como optional dependency e spawna como subprocesso.
   Sidecar do Tauri testado e funcionando nesse Mac.
2. **MVP**: app de 2 botões, tools escopadas via servidor MCP em Rust
   (`list_mirror`/`read_mirror`/`write_out` — prontas e testadas), painel de
   diff/promover offline.
3. **Críticos herdados do protótipo** (abaixo) — a maioria já é resolvida pelo
   desenho da Opção B, falta implementar.
4. **Fase 2**: chat livre, UC5 (busca semântica — ver `estudos.md`, seção
   "interessante pro projeto"), UC4/6/7.

## Críticos herdados do protótipo

1. **Proposta-zumbi**: resolvido pelo ciclo de vida acima (staging = outbox).
   Falta implementar.
2. ~~Airgap "soft"~~: resolvido pela Opção B (tools escopadas, sem bash livre).
3. ~~Push pré-autorizado~~: resolvido pela Opção B (promover é ação da UI, não
   comando de agente).
4. ~~Path do vault fixo por SO no protótipo~~: resolvido — folder picker no
   onboarding (`tauri-plugin-dialog`) + path salvo via `tauri-plugin-store`,
   sincroniza o mirror na escolha. **Gap novo**: não tem UI pra trocar de
   vault depois de escolhido (só editando o `config.json` na mão ou
   reinstalando). Falta decidir onde esse botão mora (home? configurações?).
5. Sem detecção de conflito máquina-a-máquina (relevante só se UC3 entrar).
6. Nenhum registro de sessão (`sessions.log`) — decidir se entra no MVP ou fica
   pra depois.

## Documentação relacionada

- **`estudos.md`** — links de estudo por tema, o que seguir, o que foi
  descartado e porquê.
- **`prompts/vault-agent.md`** — regras completas do Escrivão e do Bibliotecário
  (conservação total, voz do Gabs, convenções do vault — MOC, frontmatter). É o
  texto que vira o system prompt das tools.
- **`legado-windows/`** — protótipo de terminal, mantido como referência, não
  como base de código.
- **`app/src-tauri/scripts/test-mcp.sh`** — testa um binário de servidor MCP
  (tipo o `vault_mcp`) na mão, mandando mensagens JSON-RPC direto pelo stdin,
  sem precisar do `claude` de verdade. Uso:
  `test-mcp.sh <binário> <arquivo.jsonl> [argumentos-do-binário...]` — ver
  `vault-mcp-smoke.jsonl` de exemplo (cobre `list_mirror`, `read_mirror` —
  incluindo tentativa de path traversal — e `write_out`).
