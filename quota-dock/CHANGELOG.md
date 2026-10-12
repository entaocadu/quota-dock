# Changelog

Todas as alterações relevantes no projeto QuotaDock serão documentadas neste arquivo.

## [0.1.0] - 2026-10-11

### Adicionado
- **Core Architecture:** Estrutura completa em Tauri 2, Rust e React 19 + TypeScript.
- **System Tray:** Residente na bandeja de notificações do Windows, com flyout flutuante, posicionamento inteligente e menu nativo de contexto (Abrir, Atualizar, Diagnóstico, Sair).
- **Adaptador OpenAI Codex:** Leitura via stdio do `codex app-server` oficial com parsing de janelas (5h e 7d), `usedPercent` normalizado para `remainingPercent`, timestamps de renovação em fuso horário local e créditos de rate limit.
- **Adaptador Google Antigravity:** Leitura segura e somente leitura do arquivo de cache `antigravity-accounts.json` do plugin `@pieliesdie`, com acompanhamento de Gemini 3.8 Flash, Gemini Pro e Claude.
- **Painel de Tokens Locais:** Integração com `opencode stats --json` para contabilidade clara e separada do uso de tokens na máquina.
- **Diagnóstico Seguro:** Verificação de integridade das integrações locais sem exibição ou exportação de credenciais.
- **Histórico & Ajustes:** Armazenamento SQLite próprio para histórico e configurações (tema, início automático com o Windows, alertas e polling interval).
- **Testes Unitários:** Testes de normalização, contratos, banco SQLite e sanitização validados.
