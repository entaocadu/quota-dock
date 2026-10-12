# QuotaDock

Monitor de quotas para Windows 10/11, feito com Tauri, Rust e React. A janela compacta abre pela bandeja, junto ao ícone, e mostra as contas Google do OpenCode e os limites Codex das contas OpenAI.

## Uso

- Clique no ícone da bandeja para abrir ou ocultar. Clique fora ou pressione Esc para ocultar.
- Arraste o título para mover. A próxima abertura volta para perto da bandeja.
- O aplicativo inicia oculto, não ocupa a barra de tarefas e registra a inicialização com o Windows. Isso pode ser desativado em Ajustes.
- Atualiza ao abrir e automaticamente a cada minuto, inclusive enquanto a janela está oculta. O botão Atualizar força uma nova consulta.
- Clique na fonte abaixo de cada conta para ver horários de renovação e leituras por modelo.
- A altura acompanha o conteúdo até 640 pixels lógicos, respeitando a área útil do monitor; listas maiores usam rolagem. Não há quantidade fixa de contas. As consultas usam até seis conexões simultâneas por provedor, com renovação independente por conta.

## Fontes reais

**Google:** lê as contas do plugin Antigravity do OpenCode e consulta `fetchAvailableModels` diretamente na Google. Renova tokens de acesso do Google em memória. Exibe os modelos retornados pelo provedor; o resumo de cada família é o menor saldo informado entre seus modelos, nunca a soma. Se uma consulta falhar, a leitura anterior permanece identificada como antiga.

**OpenAI:** consulta `https://chatgpt.com/backend-api/wham/usage`, a rota usada pelo cliente oficial Codex. As durações, percentuais e horários vêm da resposta; nenhuma janela ou quantidade é fixada como quota.

O novo login OpenCode com `chatgpt.tokens.use.direct` tem outro público/escopo e não permite esta consulta (401 verificado). Quando um login antigo compatível estiver disponível no OpenCode, ele é usado. Caso contrário, o QuotaDock pode ler somente o token de acesso salvo em `CODEX_HOME/auth.json` ou `~/.codex/auth.json`. Isso não executa o Codex nem exige que ele esteja aberto. Use a mesma conta.

Para sessões que o próprio QuotaDock possa renovar, abra **Ajustes → Adicionar conta OpenAI** e conecte cada conta usada no OpenCode uma vez no navegador. Cada conexão é independente, protegida pelo Windows DPAPI e nunca modifica as credenciais do Codex ou OpenCode. Reconectar uma conexão exige a mesma identidade; adicionar outra conta cria outro arquivo protegido. Fontes da mesma conta são reunidas em um cartão, priorizando a conexão própria saudável e mantendo alternativas de leitura quando necessário.

**Os limites exibidos para OpenAI são os limites Codex da conta. Esta fonte não fornece os limites específicos do novo compartilhamento de assinatura com o OpenCode.** A interface identifica a fonte e não apresenta esse limite separado como se tivesse sido medido.

## Compilação e verificação

Node.js 20+, Rust e WebView2. No Windows, use MSVC ou MinGW GNU compatível.

```powershell
npm ci
npm test
npm run build
npm run test:rust
npm run tauri build -- --bundles nsis
```

O script de testes Rust adiciona o manifesto Common Controls v6 aos executáveis de teste GNU, porque o harness padrão não inclui a dependência exigida por Tauri. O executável de produção recebe seu manifesto no build Tauri.

Para verificar consultas reais de suas contas, compile o exemplo `verify_sources` e execute-o com o mesmo manifesto de testes. Ele retorna somente DTOs públicos sanitizados, sem tokens. `--show` abre o popup para verificação; a inicialização normal permanece oculta.

Veja [BUILD_WINDOWS](docs/BUILD_WINDOWS.md), [PROVIDER_STATUS](docs/PROVIDER_STATUS.md) e [SECURITY](docs/SECURITY.md).
