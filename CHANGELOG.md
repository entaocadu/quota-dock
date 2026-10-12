# Changelog

## Não lançado — documentação e estrutura

- README na raiz apresenta o monitor como complemento do OpenCode, com prints editados para privacidade.
- Guias de instalação, contas, fontes, solução de problemas, arquitetura, desenvolvimento, validação e releases.
- Licença MIT, política de segurança, contribuição, formulários de issues e modelo de PR.
- CI Windows com Node.js 24 e verificação dos links da documentação.
- Remoção de métricas antigas sem metodologia reproduzível e correção do requisito Node.js para Vitest 5.

## 0.3.0 — 2026-10-12

- Entrada de 230 ms e saída de 170 ms pela borda direita, com movimento suave e suporte à preferência de reduzir animações do Windows.
- Janela alinhada ao centro da borda direita do monitor; mantém altura automática, rolagem e arrasto pelo título.
- Ajuste para fixar acima dos outros apps e manter aberto ao perder o foco. Esc e o botão de recolher continuam disponíveis.
- Ajuste para barra de tarefas ou apenas bandeja. No modo barra, recolher minimiza; restaurar reapresenta a janela.
- Preferências antigas preservadas na migração; transições repetidas descartam eventos antigos sem ocultar uma janela recém-aberta.

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
