# Auditoria Sanitizada do Ambiente (Fase 0)

Data: 11/10/2026  
Plataforma: Windows 11 x64 (Build 10.0.26300, Home Single Language)

## 1. Ferramentas e Runtimes Detectados

| Ferramenta / Runtime | Versão Detectada | Caminho / Detalhes | Status |
|---|---|---|---|
| Windows OS | 10.0.26300 64-bit | `Win32_OperatingSystem` | Suportado |
| Microsoft WebView2 | 154.0.4258.62 | `C:\Program Files (x86)\Microsoft\EdgeWebView\Application` | Presente / Funcional |
| Rust (rustc) | 1.98.1 | Host: `x86_64-pc-windows-gnu` | Suportado |
| Cargo | 1.98.1 | Presente no PATH | Suportado |
| GCC / MinGW-w64 | 16.1.0 | UCRT posix-seh | Suportado |
| Node.js | v24.21.0 | Presente no PATH | Suportado |
| npm | 11.19.0 | Presente no PATH | Suportado |
| OpenCode CLI | v2.0.26 | Presente no PATH | Suportado |
| Codex CLI | 0.162.1 | `%USERPROFILE%\AppData\Roaming\npm\codex.ps1` | Presente / Funcional |

## 2. Caminhos e Dados Locais de Integração

| Recurso | Caminho Detectado | Detalhes Sanitizados | Status de Quota |
|---|---|---|---|
| OpenCode DB | `%USERPROFILE%\.local\share\opencode\opencode.db` | Detectado via `opencode debug paths db` | SQLite ativo (read-only mode) |
| OpenCode Config | `%USERPROFILE%\.config\opencode` | Detectado via `opencode debug paths config` | Acessível |
| Plugin Google Antigravity | `%USERPROFILE%\.config\opencode\antigravity-accounts.json` | Plugin `@pieliesdie/opencode-antigravity-auth`. Chaves presentes: `version`, `accounts`, `activeIndex`. Cada conta possui `cachedQuota` (`gemini-flash`, `gemini-pro`, `claude`) com `remainingFraction`, `resetTime`, `modelCount` e `cachedQuotaUpdatedAt` (epoch ms). | Suportado via Cache local (`antigravity_plugin_cache`) |
| Codex app-server | Executável `codex app-server` | Handshake JSON-RPC stdio funcional (`initialize`, `initialized`, `account/rateLimits/read`). Responde com bucket `codex`, janelas 5h e 7d, `ordinaryUsageAllowed=true`, `planType: "plus"`, `rateLimitResetCredits`. | Suportado via JSON-RPC stdio oficial (`codex_app_server`) |
| OpenCode Stats | `opencode stats --json` | Comando retorna JSON estruturado com métricas de tokens (`input`, `output`, `cache.read`, `cache.write`), modelos e sessões. | Suportado para Tokens Locais (`opencode_stats`) |

## 3. Matriz de Viabilidade Real

| Provedor / Fonte | Método | Disponibilidade na Máquina | Classificação | Observações |
|---|---|---|---|---|
| OpenAI Codex | `codex app-server` stdio | SIM (Ativo e Autenticado) | `supported` | Retorna rateLimits, janelas 5h e 7d e reset timestamps reais |
| OpenAI OpenCode | Token WHAM do OpenCode | NÃO GARANTIDO (401 conhecido) | `unavailable` | Fallback para app-server oficial e link oficial |
| Google Antigravity | Cache local `antigravity-accounts.json` | SIM (Arquivo presente e populado) | `supported` | Leitura read-only do snapshot existente com carimbo de tempo |
| Google Antigravity Live | Endpoint não oficial `v1internal` | DESLIGADO POR PADRÃO | `conditional` | Requer consentimento explícito, alto risco de ToS |
| Tokens Locais OpenCode | `opencode stats --json` | SIM (Funcional e rápido) | `supported` | Exibido em seção separada rotulada como Tokens Locais |

Nenhuma credencial, segredo ou token foi exportado ou modificado durante esta auditoria.
