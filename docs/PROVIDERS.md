# Fontes e interpretação das quotas

O objetivo do QuotaDock é acompanhar as contas que você usa no OpenCode. Uma leitura é sempre rotulada pela fonte que efetivamente respondeu.

| Fonte | Como é consultada | O que mede |
|---|---|---|
| Google / Antigravity | Contas do plugin OpenCode + Google `fetchAvailableModels` | Percentual restante e renovação por modelo informado |
| OpenAI / conexão QuotaDock | OAuth próprio + `wham/usage` | Janelas de uso Codex da conta, com renovação própria do login |
| OpenAI / login OpenCode compatível | Credencial salva, somente leitura + `wham/usage` | Limites Codex quando o token permite a consulta |
| OpenAI / login local Codex | Token de acesso salvo, somente leitura + `wham/usage` | Limites Codex da conta, sem executar Codex |
| Google / cache OpenCode | Último snapshot salvo pelo plugin | Leitura anterior identificada como tal |

## Google: modelos e famílias

O monitor filtra as famílias Claude, Gemini Flash e Gemini Pro. Os nomes e saldos dos modelos vêm do payload do provedor. O resumo de uma família é o **menor saldo válido informado** entre seus modelos, não a soma nem a média das quotas.

Os detalhes da conta mostram as leituras individuais. Uma ausência de percentual não é convertida em zero ou cem. Modelos fora das famílias suportadas não viram cartões adicionais nesta versão.

## OpenAI: limites Codex da conta

A consulta usa `https://chatgpt.com/backend-api/wham/usage`, rota utilizada pelo cliente Codex. Durações, percentuais usados, saldos e horários são derivados da resposta. O monitor não fixa as janelas como regra universal; 5 horas e 7 dias nos prints são exemplos de uma resposta.

O login atual do compartilhamento OpenCode pode ter público/escopo diferente, direcionado a `api.openai.com/v1`, e não autorizar a leitura dessa rota. Nesse caso, conecte uma sessão própria no QuotaDock ou use um login local compatível da mesma conta.

**A quota Codex da conta não é uma medição do teto específico do compartilhamento da assinatura com OpenCode. Esse teto separado não é informado por esta resposta.** O fato de a conta ser a mesma não torna as duas medidas equivalentes.

## Atualização e falhas

As consultas são feitas ao abrir, em intervalos configurados e pelo botão de atualização. A leitura ao abrir pode aproveitar um resultado de até 15 segundos; a atualização manual força a consulta. As requisições têm timeout e não são um fluxo de eventos em tempo real.

| Estado na interface | Interpretação |
|---|---|
| Ao vivo | A fonte respondeu com uma leitura válida |
| Parcial | Há dados válidos, mas a resposta não está completa |
| Leitura anterior | Um valor salvo é mostrado com sua data; pode ter mudado |
| Autenticação | A sessão ou o escopo precisa ser corrigido |
| Falha de conexão | A consulta não conseguiu obter uma resposta atual |
| Sem leitura / Fonte alterada | O dado esperado não foi retornado ou mudou de formato |

Falhas transitórias podem conservar a última leitura da mesma conta, marcada como antiga. Erros de autenticação não viram saldo zero. Veja [solução de problemas](TROUBLESHOOTING.md).

## Referências de implementação

O comportamento publicado deve ser conferido no código do QuotaDock. Estas referências externas ajudam a entender os clientes integrados e podem evoluir:

- [Cliente Codex](https://github.com/openai/codex/tree/main/codex-rs/backend-client).
- [Login Codex](https://github.com/openai/codex/tree/main/codex-rs/login).
- [Plugin Antigravity](https://github.com/pieliesdie/opencode-antigravity-auth).
- [OpenCode](https://github.com/anomalyco/opencode).

[Voltar à documentação](README.md)
