# Política de segurança

## Reportar uma vulnerabilidade

Use o [canal privado de relatos de segurança do GitHub](https://github.com/entaocadu/quota-dock/security/advisories/new). Evite abrir uma issue pública que inclua detalhes exploráveis, tokens ou identidades reais de contas.

Descreva a versão afetada, o impacto e passos de reprodução com dados fictícios. Se houver credenciais expostas, revogue-as no provedor responsável; não as envie no relato. Não anexe arquivos `auth.json`, `antigravity-accounts.json`, bancos OpenCode/QuotaDock, sessões DPAPI ou URLs completas de callback.

## Versões e correções

O mantenedor prioriza a versão estável mais recente publicada em [Releases](https://github.com/entaocadu/quota-dock/releases/latest). Use essa versão ao verificar um possível problema e informe se ele também ocorre em uma versão anterior. Não há prazo de resposta ou suporte comercial prometido.

## Modelo de proteção

Credenciais ficam no backend Rust; DTOs enviados à UI e ao histórico não incluem tokens. Sessões OpenAI próprias usam Windows DPAPI e identidades independentes. Fontes compartilhadas do OpenCode e Codex são abertas somente para leitura.

O monitor faz requisições diretas HTTPS aos provedores, sem servidor de telemetria próprio. As fronteiras, caminhos locais e limites da proteção estão em [PRIVACY](docs/PRIVACY.md); o fluxo está em [ARCHITECTURE](docs/ARCHITECTURE.md).

O `.gitignore`, os checksums e a verificação do histórico ajudam na publicação, mas não substituem a revisão de conteúdo. Um backup privado que contenha dados antigos deve continuar privado.
