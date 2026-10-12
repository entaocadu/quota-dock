# Segurança

Os tokens são usados apenas pelo backend Rust. A interface e o histórico recebem QuotaSnapshot, sem credenciais. Mensagens de erro não incluem corpos de respostas, cabeçalhos Authorization ou URLs de callback.

O banco e o arquivo de credenciais do OpenCode são abertos somente para leitura. O login do Codex também é somente leitura: QuotaDock não executa Codex, não copia seu refresh token para outro armazenamento e não renova nem sobrescreve esse login.

A conexão OpenAI própria usa OAuth Authorization Code com PKCE S256 e estado aleatório, callback apenas no loopback IPv4 (1455 ou 1457) e timeout de 180 segundos. Requisições incompletas ou estados inválidos não encerram nem autorizam a sessão. A troca de código vai ao endpoint HTTPS fixo da OpenAI; a identidade da resposta confiável é reconciliada com a consulta de quota e, em uma reconexão, com a identidade da conexão escolhida.

Cada sessão própria é cifrada com Windows DPAPI para o usuário atual, em APPDATA/QuotaDock/openai-sessions/<identidade-hash>.bin. A identidade é conferida também ao abrir o arquivo. Renovações e reconexões da mesma conta são serializadas; contas diferentes usam bloqueios independentes. A sessão legada única é migrada após salvamento protegido. Os tokens Google de acesso ficam em memória; o arquivo de contas do plugin não é alterado.

As consultas HTTPS usam origens fixas, TLS e timeout; redirecionamentos HTTP são recusados. Arquivos de login possuem limite de tamanho. O histórico local é limitado aos últimos 2000 snapshots sanitizados.

Não há servidor próprio de telemetria. O backend se conecta diretamente aos provedores. Os limites são reportados pelo provedor, e qualquer dado anterior é identificado como antigo.

As constantes OAuth do plugin Google e o client ID público do Codex são configurações públicas dos clientes; não são tokens ou segredos pessoais do usuário.

## Publicação

Ferramentas locais, backups, arquivos de login, bancos de dados e sessões são excluídos pelo `.gitignore`. Os pacotes publicados contêm somente o executável, a dependência WebView2 necessária e instruções de uso. Cada release inclui os hashes SHA-256 dos arquivos distribuídos. Screenshots de contas e resultados de consultas locais não são anexados às releases.

Antes de tornar o repositório público, o histórico Git foi verificado com Gitleaks e comparado, sem registrar valores, com as credenciais e identidades locais. Os pacotes de distribuição também foram verificados. Nenhum token, segredo pessoal ou identidade de conta foi incluído nos arquivos publicados.
