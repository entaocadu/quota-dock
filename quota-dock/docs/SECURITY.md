# Política de Segurança e Privacidade — QuotaDock

## 1. Princípios Inegociáveis

1. **Leitor Estrito, Não Gerenciador de Credenciais:** O QuotaDock não armazena senhas, refresh tokens ou segredos de autenticação em disco.
2. **Zero Backend Próprio:** Toda e qualquer operação é executada exclusivamente na sua máquina local. Nenhuma telemetria, medição de uso ou tráfego de rede é enviado para servidores do QuotaDock.
3. **Não Destrutivo:** O QuotaDock nunca altera nem escreve nos arquivos de configuração ou bancos de dados do OpenCode (`opencode.db`, `opencode.json`, `antigravity-accounts.json`, etc.). Qualquer acesso a arquivos locais é estritamente em modo *read-only*.
4. **Isolamento de Segredos:** Qualquer dado sensível lido temporariamente para consultas de autorização permanece exclusivamente em memória pelo tempo estritamente necessário para o request e é descartado imediatamente. Nenhuma credencial ou token JWT é passado para o frontend WebView ou gravado em logs.

## 2. Redação e Mascaramento

- E-mails de contas são mascarados no backend antes de alcançar a interface gráfica: ex. `e***@gmail.com`.
- Chaves identificadoras de contas (`accountKey`) são geradas via hash SHA-256 local com salt exclusivo, evitando a propagação de tokens brutos como identificadores.
- Funções de redação sanitizam strings de erro e respostas para suprimir cabeçalhos `Authorization: Bearer [REDACTED]` e segredos OAuth.

## 3. Postura diante de Endpoints Privados

- **Google Antigravity:** O modo padrão do QuotaDock é **Somente Leitura de Cache Local** (`antigravity_plugin_cache`). A consulta direta aos endpoints comunitários privados (`v1internal:fetchAvailableModels`) é experimental, mantida desativada por padrão e exige consentimento explícito do usuário nas Configurações, devido aos termos de serviço do provedor.
- **OpenAI Codex:** O acesso a cotas é realizado primariamente através do protocolo oficial JSON-RPC stdio do `codex app-server`. Caso uma autenticação retorne erro 401 ou escopo não autorizado, o QuotaDock não tenta contornar o erro e disponibiliza link direto para o dashboard oficial da OpenAI.

## 4. Banco de Dados Local do QuotaDock

O aplicativo mantém um banco SQLite próprio e isolado localizado em `%APPDATA%\QuotaDock\quotadock.db`. Esse banco armazena exclusivamente:
- Preferências públicas do usuário (tema, intervalo de atualização, início com Windows).
- Histórico resumido e sanitizado de quotas observadas (`json_sanitized` sem segredos).
