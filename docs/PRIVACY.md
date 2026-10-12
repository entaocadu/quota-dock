# Privacidade e armazenamento

## O que fica no computador

| Dado | Local / tratamento |
|---|---|
| Preferências e histórico de quotas | `%APPDATA%\QuotaDock\quotadock.db`, SQLite local |
| Sessões OpenAI criadas no QuotaDock | `%APPDATA%\QuotaDock\openai-sessions\<hash>.bin`, cifradas com Windows DPAPI |
| Contas Google do plugin | Arquivo do OpenCode, lido sem modificações |
| Credenciais OpenCode | Banco ou arquivo original, aberto somente para leitura |
| Login local Codex | `CODEX_HOME/auth.json` ou `~/.codex/auth.json`, somente leitura |
| Tokens Google de acesso renovados | Memória do processo |

O histórico é limitado aos últimos 2000 snapshots sanitizados. Ele contém leituras, datas, origem, identificadores derivados e rótulos mascarados; não contém tokens. Embora não contenha credenciais, o histórico pode revelar padrões de uso e não deve ser anexado a issues públicas.

DPAPI protege a sessão própria para o usuário Windows que a criou. O arquivo não é uma sessão portátil entre usuários ou máquinas. O SQLite local não é descrito como um banco cifrado.

## Para onde vão as consultas

O backend Rust se comunica diretamente com os endpoints HTTPS dos provedores para autenticar e consultar quotas. Não há servidor de telemetria do QuotaDock. Tokens não são entregues à interface React nem incluídos em mensagens de erro.

As requisições têm timeout e recusam redirecionamentos HTTP. O callback de login próprio usa somente loopback IPv4. O QuotaDock não modifica as credenciais compartilhadas do OpenCode ou do Codex.

## Publicação e suporte

Tokens, senhas, arquivos de login, bancos, backups e resultados locais não pertencem ao repositório ou aos pacotes de release. O `.gitignore` exclui os caminhos e formatos conhecidos; ele é uma proteção preventiva, não substitui a revisão do conteúdo antes de publicar.

As imagens de documentação derivam de prints autorizados pelo autor e receberam edição para remover identificações de contas e projetos. Elas não devem ser usadas como dataset de quotas ou benchmark. Veja [imagens](assets/README.md).

O histórico público e os pacotes distribuídos foram auditados antes da publicação inicial. Novas contribuições e releases precisam repetir a revisão de dados sensíveis. Uma ausência de achados em um scanner não é uma garantia universal contra vazamentos.

## Limites da proteção

Um processo malicioso rodando no mesmo usuário Windows pode ter acesso aos dados desse usuário. O QuotaDock não substitui a segurança do sistema, do navegador ou das contas dos provedores. Configure suas contas apenas em canais confiáveis e evite expor arquivos locais.

Para reportar uma vulnerabilidade, consulte [SECURITY](../SECURITY.md).

[Voltar à documentação](README.md)
