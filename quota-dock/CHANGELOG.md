# Changelog

## 0.2.0 — 2026-10-12

- Popup de 360 pixels de largura em preto carbono, barras monocromáticas e altura automática; listas grandes usam rolagem.
- Posição junto à bandeja respeita área útil, DPI e múltiplos monitores. Arraste pelo título; clique fora ou Esc oculta.
- Inicialização Windows real, oculta; instância única e ausência da barra de tarefas.
- Consultas Google ao vivo, por conta e por modelo, com tokens em memória e cache anterior explicitamente marcado.
- OpenAI consulta diretamente wham/usage, lê credenciais atuais OpenCode/SQLite e login Codex compatível sem executar Codex.
- Conexão OpenAI própria com OAuth PKCE, DPAPI e renovação independente; nunca renova credenciais compartilhadas de outro app.
- Quantidade variável de contas Google e OpenAI; conexões protegidas individuais, consultas concorrentes limitadas e seleção de fonte saudável da mesma conta.
- Atualização em segundo plano a cada minuto e atualização manual forçada.
- Dados ausentes não viram zero ou cem por cento; não há versões de modelo ou durações de quota inventadas.
- Histórico limitado a 2000 leituras sanitizadas, testes de parsers, identidade, callbacks fragmentados e proteção Windows.
- Vitest atualizado; auditoria npm sem vulnerabilidades após a atualização.
