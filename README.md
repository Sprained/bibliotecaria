# bibliotecária

Cuida do seu vault do Obsidian com a ajuda do Claude — sem deixar o agente escrever direto nas suas notas. Toda proposta de mudança fica pendente pra você revisar (com diff) antes de aceitar.

## Instalação

### 1. Baixe o app

Pega o `.dmg` mais recente na página de [Releases](https://github.com/Sprained/bibliotecaria/releases) (tem que ser a versão `aarch64` — Mac com chip Apple, M1/M2/M3 ou mais novo. Mac Intel e Windows/Linux ainda não têm build).

### 2. Instale

Abre o `.dmg` baixado e arrasta o ícone da bibliotecária pra pasta **Applications**.

### 3. Libere a primeira abertura

Essa versão ainda não tem assinatura de desenvolvedor da Apple, então na primeira vez que você abrir o macOS vai bloquear com uma mensagem tipo "não é possível verificar o desenvolvedor". É esperado — não é vírus, é só a Apple cobrando uma assinatura paga que esse projeto ainda não tem. Pra liberar:

- Vá na pasta **Applications**, **clique com o botão direito** no ícone da bibliotecária → **Abrir**
- Vai aparecer o aviso de novo, mas dessa vez com um botão **Abrir** — clica nele
- Só precisa fazer isso uma vez; depois o app abre normal, com duplo clique

Se isso não aparecer: **Ajustes do Sistema → Privacidade e Segurança**, rola até o fim → deve aparecer "bibliotecária foi bloqueado" → clica em **Abrir Assim Mesmo**.

### 4. Primeiro uso

Ao abrir pela primeira vez, o app pede:

1. **Conectar com Claude** — abre o navegador pra você logar com sua conta Claude (é a sua assinatura normal, não precisa de API key nem paga nada a mais)
2. **Escolher a pasta do seu vault** do Obsidian

Depois disso, é só usar os dois botões:

- **Escrivão** — escolhe uma nota bagunçada e pede pra reescrever
- **Bibliotecário** — audita o vault inteiro (notas órfãs, duplicatas, links quebrados) e gera um relatório

Toda sugestão do agente fica pendente em **"Revisar propostas"** — você vê lado a lado o que mudou antes de aceitar ou descartar. Nada é escrito na sua nota de verdade sem você mandar.

## Limitações desta versão (v0.1.0, pre-release)

- Só Mac Apple Silicon
- Sem assinatura/notarização da Apple (daí o aviso na primeira abertura)
- Sem atualização automática — pra pegar uma versão nova, baixa e reinstala na mão

---

Detalhes técnicos, decisões de arquitetura e o histórico do projeto estão documentados em [`CLAUDE.md`](CLAUDE.md).
