# Status e Detalhes de Medição por Provedor

Relatório técnico das fontes de dados integradas ao QuotaDock.

---

## 1. OpenAI / Codex

### Como a Quota é Medida:
O QuotaDock estabelece uma comunicação segura via subprocesso stdio com o executável oficial `codex app-server` (instalado no PATH com a CLI oficial do Codex).

1. **Handshake JSON-RPC:**
   - Envia `initialize` com identificação do cliente QuotaDock.
   - Envia `initialized` confirmando a sessão.
   - Envia `account/rateLimits/read` (id=2).
2. **Campos Lidos e Validados:**
   - `ordinaryUsageAllowed`: Indica se o usuário possui permissão de uso normal no momento. Se `false`, o app sinaliza bloqueio ativo.
   - `primary`: Janela de curto prazo (normalmente 5 horas / 300 minutos). Contém `usedPercent`, `windowDurationMins` e `resetsAt` (timestamp Unix epoch em segundos).
   - `secondary`: Janela semanal (normalmente 7 dias / 10080 minutos). Contém `usedPercent`, `windowDurationMins` e `resetsAt`.
   - `rateLimitResetCredits`: Quantidade de créditos de reset de rate limit disponíveis na conta ChatGPT Plus/Pro.
   - `planType`: Exibe a assinatura identificada (ex: `PLUS`, `PRO`, `TEAM`).
3. **Conversão de Quota:**
   - O Codex reporta `usedPercent`. O QuotaDock calcula `remainingPercent = clamp(100 - usedPercent, 0, 100)`.
   - A renovação (`resetsAt`) é convertida diretamente para o fuso horário local do Windows com contagem regressiva inteligente.
4. **Comportamento em Falha / 401:**
   - Caso o CLI não esteja autenticado ou retorne 401, o QuotaDock reporta `Sem permissão para consultar quota` e disponibiliza botão de atalho para o painel oficial da OpenAI (`https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan`).

---

## 2. Google / Antigravity

### Como a Quota é Medida:
O QuotaDock monitora o arquivo de cache mantido pelo plugin `@pieliesdie/opencode-antigravity-auth` no perfil do usuário:
`%USERPROFILE%\.config\opencode\antigravity-accounts.json`

1. **Leitura Segura:**
   - O arquivo é aberto estritamente em modo de leitura.
   - O QuotaDock extrai apenas o `cachedQuota` e `cachedQuotaUpdatedAt`.
2. **Modelos Monitorados:**
   - **Gemini 3.8 Flash:** Fração restante e horário de renovação (`resetTime`).
   - **Gemini Pro:** Fração restante e horário de renovação.
   - **Claude (via Antigravity):** Fração restante e horário de renovação.
3. **Conversão de Quota:**
   - O plugin reporta `remainingFraction` (0.0 a 1.0).
   - O QuotaDock converte para percentual: `remainingPercent = clamp(fraction * 100, 0, 100)`.
4. **Gestão de Frescor:**
   - Se a leitura tiver menos de 15 minutos: status **CONFIRMADO** (`ok`).
   - Se tiver entre 15 minutos e 24 horas: status **DESATUALIZADO** (`stale`) com aviso dos minutos passados.
   - Se o arquivo ou o cache não existirem: status **SEM SNAPSHOT** (`no_data`), sem inventar falsos zeros ou 100%.

---

## 3. OpenCode / Tokens Locais

### Como os Tokens são Medidos:
- Consulta via comando local `opencode stats --days <N> --cost --models --json`.
- Apresenta métricas agregadas de sessões locais: tokens de entrada, tokens de saída, reasoning e cache read.
- O painel exibe explicitamente o aviso: *"Tokens locais ≠ Saldo de assinatura"*, evitando qualquer confusão entre contabilidade de mensagens e cota restante de planos.
