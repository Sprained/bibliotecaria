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
- **Prompt de cada modo** (decidido 2026-09-15, quando a UI virou "2 botões"
  de verdade): `prompts/shared.md` (Limite absoluto, Convenções do vault,
  Formato da resposta) + `prompts/escrivao.md` + `prompts/bibliotecario.md`,
  cada um só com a seção específica do modo. Rust concatena `shared + modo`
  na hora de montar o `--system-prompt`. Zero duplicação de regra de
  governança entre os dois modos.
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
  (testado) é chamado por comandos Tauri reais (`get_vault_path`/
  `set_vault_path`/`sync_now`), path do vault persistido via
  `tauri-plugin-store`, escolha de pasta via `tauri-plugin-dialog` no
  onboarding, botão "Trocar vault" na home.
- **Os 2 botões (Bibliotecário e Escrivão) funcionando ponta a ponta com
  vault real** (validado 2026-09-15): `commands/agent.rs` sobe `claude -p`
  com `--system-prompt` (`prompts/shared.md` + o modo específico, embutidos
  via `include_str!`) + `--mcp-config` inline apontando pro `vault_mcp` +
  `--strict-mcp-config` + `--tools ""` + `--permission-mode
  bypassPermissions`, token OAuth passado via `.env()` (nunca na linha de
  comando). Os dois comandos resincronizam o mirror antes de rodar
  (`sync_now`), pra não trabalhar em cima de conteúdo desatualizado.
  - **Bibliotecário**: auditou o vault de verdade via `list_mirror`/
    `read_mirror`, achou notas órfãs, MOC fora de convenção, referência
    morta, e escreveu o relatório em `out/`.
  - **Escrivão**: picker nativo filtrado por `.md`, começando na pasta do
    vault (`@tauri-apps/plugin-dialog`). Rust calcula o caminho relativo ao
    vault (`canonicalize` dos dois lados + `strip_prefix`, com erro se a
    nota escolhida cair fora do vault) e manda pro agente reescrever via
    `read_mirror`/`write_out`.
  - Rodar num vault real leva minutos (lê nota por nota) — UI avisa isso.
- Fora isso, o que existe é o protótipo de terminal (Opção A), testado no
  Windows, guardado em `legado-windows/` como referência de comportamento —
  não é pra evoluir, é pra não perder o que já foi validado (as regras do
  escrivão/bibliotecário, a estrutura mirror/out/staging).
- **Painel de revisão de propostas funcionando ponta a ponta, com diff de
  verdade** (validado 2026-09-16): botão "Revisar propostas" na home (com
  contador) → lista (`list_proposals`) → detalhe → Aceitar
  (`promote_proposal`: escreve no vault, resincroniza mirror, apaga de
  `out/`) / Rejeitar/Descartar (`discard_proposal`). Testado com nota real
  — Aceitar escreveu de fato na nota do vault, conferido abrindo no
  Obsidian. Detalhe da Substituição usa `@codemirror/merge` (`MergeView`,
  read-only, `lineWrapping`, `collapseUnchanged` pra pular trechos sem
  mudança em notas longas) com tema próprio (`EditorView.theme` lendo as
  mesmas variáveis CSS do app — claro/escuro automáticos); Relatório
  continua com `<pre>` simples (não tem original pra comparar). **Fecha o
  MVP** (as duas fatias do plano de propostas).
- **Primeiro release publicado** (2026-09-17):
  [v0.1.0](https://github.com/Sprained/bibliotecaria/releases/tag/v0.1.0),
  marcado como pre-release. `vault_mcp` entrou no `externalBin` do Tauri
  (mesmo tratamento do binário `claude`) via `scripts/build-sidecars.sh`;
  validado com `tauri build` real — o `.app` final tem os três binários
  (`app`/`claude`/`vault_mcp`) em `Contents/MacOS/` e o sidecar sobe
  certinho no app empacotado, não só em dev. Ganhou nome e ícone de
  verdade (`productName`/`identifier` trocados de "app" genérico,
  `app-icon.svg` gerado a partir dos mesmos paths do livro que já
  existiam na UI). Limitações conhecidas, documentadas no release: só
  Apple Silicon, sem assinatura/notarização (Gatekeeper bloqueia até o
  usuário liberar manualmente).
- Próximo passo real de código: MVP fechado, primeiro release no ar.
  Resta o `sessions.log` (crítico herdado, ainda em aberto) e, quando for
  hora de distribuir pra valer pros amigos, assinatura/notarização +
  multi-plataforma (Windows/Linux) — ver pendências abaixo.

## Pendências / ordem de trabalho

1. ~~**Spike de onboarding**~~ — resolvido. Achado importante: `claude` já é
   um binário nativo autocontido (não precisa de Node/SEA pra essa parte) — o
   Agent SDK usa ele como optional dependency e spawna como subprocesso.
   Sidecar do Tauri testado e funcionando nesse Mac.
2. **MVP**: ~~app de 2 botões, tools escopadas via servidor MCP em Rust~~
   ~~painel de diff/promover offline (fatia 1: mecânica + visualização
   simples; fatia 2: diff de verdade com CodeMirror 6 +
   `@codemirror/merge`)~~ — **fechado** (2026-09-16). `commands/proposals.rs`
   (`list_proposals`/`read_proposal`/`promote_proposal`/`discard_proposal`)
   + telas de lista/detalhe no frontend, com `MergeView` pra Substituição.
   Validado com vault real.
3. **Críticos herdados do protótipo** — todos resolvidos (ver seção abaixo).
4. **Distribuição de verdade** (pra sair de "só eu" pra "amigos não-técnicos"):
   - ~~Instrução de instalação/Gatekeeper~~: resolvido (2026-09-18) —
     `README.md` na raiz com passo a passo de instalação e como liberar o
     app não assinado.
   - Assinatura de código + notarização: **decidido (2026-09-18) não pagar
     Apple Developer Program (US$99/ano) por ora** — sem isso não tem
     como eliminar o aviso do Gatekeeper de verdade (confirmado: nem
     certificado self-signed resolve, ACL do Keychain é presa ao cdhash do
     binário, não ao certificado — ver `estudos.md` se quiser o porquê
     técnico). Fica em aberto, revisitar se o README não for suficiente.
   - Multi-plataforma (Windows/Linux — hoje só builda
     `aarch64-apple-darwin`): em aberto.
   - Auto-update (plugin do Tauri já existe, não tá ligado): em aberto.
5. **Tela de histórico** lendo o `sessions.log`: combinado (2026-09-18) de
   fazer depois de resolver a distribuição — os dados já estão sendo
   escritos, só falta a UI de leitura.
6. **Fase 2**: chat livre, UC5 (busca semântica — ver `estudos.md`, seção
   "interessante pro projeto"), UC4/6/7.
7. **Normalizar nomenclatura do frontend (TS) pra inglês** — convenção do
   projeto é nomes de função/variável em inglês, comentários/erros podem
   ficar em português (já aplicado no backend Rust). O `app/src/main.ts`
   ainda tem função/variável em português de sessões anteriores
   (`mostrarTela`, `abrirPropostas`, `rodarBibliotecario`, etc.).
   Combinado (2026-09-18) de deixar assim por ora — não bloqueia nada — e
   normalizar depois, junto de outra rodada de mudança no frontend em vez
   de um rename isolado.

## Críticos herdados do protótipo

1. ~~Proposta-zumbi~~: resolvido e implementado — Aceitar/Rejeitar/Descartar
   sempre apagam o arquivo de `out/` (`promote_proposal`/`discard_proposal`).
2. ~~Airgap "soft"~~: resolvido pela Opção B (tools escopadas, sem bash livre).
3. ~~Push pré-autorizado~~: resolvido pela Opção B (promover é ação da UI, não
   comando de agente).
4. ~~Path do vault fixo por SO no protótipo~~: resolvido — folder picker no
   onboarding (`tauri-plugin-dialog`) + path salvo via `tauri-plugin-store`,
   sincroniza o mirror na escolha, e botão "Trocar vault" na home pra mudar
   depois.
5. Sem detecção de conflito máquina-a-máquina (relevante só se UC3 entrar).
6. ~~Nenhum registro de sessão~~: resolvido (2026-09-18). `sessions.rs`
   escreve `sessions.log` (JSONL, um evento por linha) na pasta de dados do
   app, do lado de `mirror/`/`out/`. Dois tipos de evento: `AgentRun`
   (fim de uma sessão do Bibliotecário/Escrivão — modo, nota quando
   Escrivão, sucesso, resumo do que o agente devolveu) e `Decisao`
   (aceita/descartada, com o path). Só escrita por enquanto, sem tela de
   leitura — decisão deliberada pra não inventar UI que ninguém pediu
   ainda; os dados já ficam prontos pro dia que quiser essa tela (ver
   pendências).
7. ~~`vault_mcp` não tem empacotamento de verdade~~: resolvido (2026-09-17).
   Entrou no `externalBin` do `tauri.conf.json` igual o `claude`;
   `scripts/build-sidecars.sh` builda e renomeia com o target triple certo
   (com bootstrap de placeholder, já que o `build.rs` do Tauri valida a
   existência do `externalBin` antes mesmo de compilar). O código que
   resolve o caminho em `agent.rs` (`current_exe().parent()`) não precisou
   mudar — confirmado que o Tauri copia sidecars pra mesma pasta do
   executável principal no bundle final, igual já acontecia em dev por
   coincidência.

## Documentação relacionada

- **`estudos.md`** — links de estudo por tema, o que seguir, o que foi
  descartado e porquê.
- **`prompts/shared.md` + `prompts/escrivao.md` + `prompts/bibliotecario.md`**
  — regras completas do Escrivão e do Bibliotecário (conservação total, voz
  do Gabs, convenções do vault — MOC, frontmatter). É o texto que vira o
  `--system-prompt` de cada modo (shared + modo específico, concatenados).
- **`legado-windows/`** — protótipo de terminal, mantido como referência, não
  como base de código.
- **`app/src-tauri/scripts/test-mcp.sh`** — testa um binário de servidor MCP
  (tipo o `vault_mcp`) na mão, mandando mensagens JSON-RPC direto pelo stdin,
  sem precisar do `claude` de verdade. Uso:
  `test-mcp.sh <binário> <arquivo.jsonl> [argumentos-do-binário...]` — ver
  `vault-mcp-smoke.jsonl` de exemplo (cobre `list_mirror`, `read_mirror` —
  incluindo tentativa de path traversal — e `write_out`).
