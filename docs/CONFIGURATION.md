# Contas e configurações

## Google / Antigravity no OpenCode

Configure e autentique as contas no [plugin Antigravity compatível com OpenCode](https://github.com/pieliesdie/opencode-antigravity-auth). O QuotaDock lê `%USERPROFILE%\.config\opencode\antigravity-accounts.json` e detecta todas as contas registradas, independentemente de qual está ativa no OpenCode.

Não há um cadastro Google separado dentro do QuotaDock. Adicione ou reautentique contas no plugin e use o botão **Atualizar quotas**. O arquivo do plugin não é modificado pelo monitor. Tokens Google de acesso são renovados em memória para consultar o provedor.

## OpenAI usada no OpenCode

Para uma sessão de quota independente:

1. Abra **Ajustes → Adicionar conta OpenAI**.
2. Faça login no navegador com a mesma conta usada no OpenCode.
3. Aguarde a confirmação no QuotaDock.
4. Repita para cada outra conta desejada.

Cada sessão própria é armazenada separadamente e pode ser renovada pelo QuotaDock. **Reconectar** exige a mesma identidade da conexão selecionada. Para adicionar outra identidade, use **Adicionar conta OpenAI**.

O aplicativo também pode ler logins compatíveis já salvos no OpenCode ou em `CODEX_HOME/auth.json` / `~/.codex/auth.json`. Essas fontes são somente leitura: o QuotaDock não altera nem renova o login compartilhado de outro aplicativo e não executa o Codex. Fontes reconhecidas da mesma conta são reunidas em um cartão, priorizando uma sessão própria saudável.

Conectar a mesma conta não muda o significado da quota OpenAI. Consulte [fontes e limites](PROVIDERS.md).

## Opções de janela

| Opção | Efeito |
|---|---|
| Iniciar com o Windows | Registra o executável para iniciar recolhido na sessão do usuário |
| Fixar acima dos outros apps | Mantém a janela acima e impede o recolhimento ao clicar fora |
| Mostrar na barra de tarefas | Ao recolher, minimiza na barra; desativado, oculta na bandeja |
| Intervalo de atualização | Permite 1, 2, 5 ou 10 minutos; padrão de 1 minuto |

Esc e X continuam recolhendo a janela fixada. Para parar o monitoramento, use **Sair** no menu da bandeja. O aplicativo mantém uma única instância.

## Janela e leitura

O painel tem 360 pixels lógicos de largura. A altura acompanha o conteúdo até 640 pixels lógicos e respeita a área útil do monitor. Muitas contas e detalhes abertos usam rolagem.

Ao abrir, o painel se posiciona na borda direita do monitor e desliza para dentro. A preferência de reduzir movimento do Windows elimina o deslocamento. Você pode arrastar o título; a próxima abertura volta à posição de entrada.

Clique na fonte abaixo de cada conta para ver os modelos, horários de renovação e avisos disponíveis. Os e-mails são mascarados na interface. As configurações descritas aqui são as disponíveis na UI atual; campos legados nos contratos não significam funcionalidades expostas ao usuário.

[Voltar à documentação](README.md)
