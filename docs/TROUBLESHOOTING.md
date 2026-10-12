# Solução de problemas e FAQ

## Não aparece uma janela ao iniciar

A inicialização recolhida é o comportamento esperado. Abra pelo ícone da bandeja, incluindo a área de apps ocultos. Com **Mostrar na barra de tarefas** habilitado, ele pode iniciar minimizado na barra.

## Não apareceu minha conta Google

Confira se ela foi autenticada no plugin Antigravity do OpenCode no mesmo usuário Windows. O QuotaDock lê o arquivo padrão do plugin. Use **Atualizar quotas** depois de adicionar a conta. Não envie o arquivo `antigravity-accounts.json` em uma issue.

## OpenAI mostra Autenticação ou a sessão expirou

Use **Ajustes → Adicionar conta OpenAI** para uma conexão própria, ou **Reconectar** para a conexão existente. Faça login com a mesma conta usada no OpenCode.

O QuotaDock não renova os logins compartilhados de outros apps. Se você escolheu usar somente um login local, renove esse login no aplicativo responsável. Uma assinatura compartilhada no OpenCode pode usar um token com escopo que não permite consultar os limites Codex.

## O navegador abre, mas a conexão não termina

O callback usa `127.0.0.1:1455` ou `127.0.0.1:1457` e espera até 180 segundos. Confira se essas portas não estão ocupadas e se o navegador concluiu o fluxo. Volte ao aplicativo para ver a mensagem. Não copie a URL completa do callback para chats públicos ou issues.

## Os valores não mudam imediatamente

As consultas são periódicas e a resposta depende do provedor. O padrão é um minuto; abrir pode reaproveitar uma leitura de até 15 segundos. Use **Atualizar quotas** para forçar uma consulta e confira a data na fonte.

Uma **Leitura anterior** não deve ser interpretada como saldo atual. Confira a rede e a autenticação. Não há estimativa de consumo de tokens para inventar quotas ausentes.

## O resumo Google difere de um modelo específico

O resumo usa o menor saldo informado da família. Abra os detalhes da fonte e compare os modelos individualmente. [Entenda a regra](PROVIDERS.md).

## O painel fica aberto ao clicar fora

Desative **Fixar acima dos outros apps**. Esc e X recolhem mesmo com a fixação habilitada. Quando a barra de tarefas está habilitada, recolher minimiza; no modo bandeja, oculta.

## Falta WebView2Loader.dll na versão portátil

Extraia todo o ZIP oficial, mantendo a DLL na pasta do executável. Instale também o runtime Microsoft WebView2. Evite copiar apenas o EXE da versão portátil.

## Preciso manter OpenCode ou Codex aberto?

O QuotaDock consulta os provedores diretamente com as credenciais compatíveis já salvas. Não precisa manter as janelas desses apps abertas. Uma credencial compartilhada expirada precisa ser renovada no app responsável; uma sessão OpenAI própria do QuotaDock pode ser renovada pelo monitor.

## Posso usar outras IAs ou outros sistemas operacionais?

A distribuição atual é Windows x64 e acompanha Google/Antigravity e OpenAI Codex. A presença de um provedor no OpenCode não significa que ele é suportado pelo QuotaDock.

## Como reportar um problema

Use o [formulário de bug](https://github.com/entaocadu/quota-dock/issues/new/choose). Inclua versão, Windows, passos, fonte e estado mostrado. Remova e-mails, IDs de contas, nomes de projetos e qualquer URL de login das imagens. Nunca anexe tokens, arquivos de autenticação, bancos locais ou respostas brutas dos provedores.

Para vulnerabilidades, siga a [política de segurança](../SECURITY.md).

[Voltar à documentação](README.md)
