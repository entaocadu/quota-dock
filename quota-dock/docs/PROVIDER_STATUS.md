# Fontes de quota — versão 0.2

| Fonte | Integração | O que mostra |
|---|---|---|
| Google/OpenCode Antigravity | OAuth do plugin + Google fetchAvailableModels | Percentual restante por modelo; resumo mínimo por família |
| OpenAI/QuotaDock | OAuth próprio + wham/usage | Janelas e percentuais reais Codex; renovação automática própria |
| OpenAI/OpenCode antigo | Token compatível já salvo, somente leitura | Janelas Codex quando o escopo permite |
| OpenAI/login local Codex | Token de acesso salvo, somente leitura | Janelas Codex da mesma conta, sem executar Codex |
| OpenCode novo token sharing | Token com público api.openai.com/v1 | Não fornece quota numérica por wham/usage; 401 constatado |
| Google/cache local | Snapshot já gerado pelo plugin | Leitura anterior, com fonte e data, sem fingir medição ao vivo |

Validação local em 12/10/2026: as duas contas Google retornaram HTTP 200, cada uma com 27 modelos com quota no payload completo. A OpenAI retornou HTTP 200 e janelas de 18000 e 604800 segundos. O app filtra as famílias Gemini Flash, Gemini Pro e Claude, preservando os nomes/modelos retornados. Percentuais ausentes permanecem ausentes.

A API Codex não expõe nesta resposta o teto específico do compartilhamento da assinatura com o OpenCode. A mesma conta não torna essas duas medidas equivalentes. O app rotula a origem e documenta essa distinção.

Falhas de rede conservam apenas a última leitura da mesma conta, marcada como antiga. Erros de autenticação pedem reconexão; não viram saldo zero. O número de contas é descoberto nos arquivos, não fixado no produto.

Referências: [cliente Codex](https://github.com/openai/codex/blob/main/codex-rs/backend-client/src/client/rate_limit_resets.rs), [login oficial](https://github.com/openai/codex/blob/main/codex-rs/login/src/server.rs), [plugin Google](https://github.com/pieliesdie/opencode-antigravity-auth/blob/main/src/plugin/quota.ts), [login atual OpenCode](https://github.com/anomalyco/opencode/blob/dev/packages/core/src/plugin/provider/chatgpt.ts).
