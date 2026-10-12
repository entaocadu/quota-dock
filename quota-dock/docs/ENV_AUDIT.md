# Compatibilidade e fontes de integração

Este documento descreve o ambiente necessário ao aplicativo, sem publicar inventários de computadores ou dados de contas.

## Windows e compilação

- Windows 10/11 x64, com Microsoft WebView2 instalado.
- Node.js 20+ e Rust para compilar.
- Toolchain MSVC ou MinGW GNU compatível; a distribuição GNU inclui `WebView2Loader.dll`.

## Integrações locais

- OpenCode: banco em `%USERPROFILE%/.local/share/opencode/opencode.db`, aberto somente para leitura.
- Antigravity: contas em `%USERPROFILE%/.config/opencode/antigravity-accounts.json`; o QuotaDock consulta a Google diretamente e não modifica esse arquivo.
- OpenAI: login compatível do OpenCode ou token de acesso em `CODEX_HOME/auth.json` / `~/.codex/auth.json`, somente para leitura. A consulta de quota não executa o Codex.
- Sessões próprias OpenAI: conexão no navegador, armazenamento protegido por Windows DPAPI e renovação independente por conta.

Os percentuais e horários exibidos vêm dos provedores. O novo login compartilhado do OpenCode tem escopo diferente e pode não oferecer a consulta de limites Codex; as alternativas compatíveis estão descritas no [README](../README.md).

Veja [BUILD_WINDOWS](BUILD_WINDOWS.md), [PROVIDER_STATUS](PROVIDER_STATUS.md) e [SECURITY](SECURITY.md) para os detalhes atuais.
