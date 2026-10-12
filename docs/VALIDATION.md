# Testes e validação

## Evidências da versão 0.3

No desenvolvimento da série 0.3 foram executados **9 testes Vitest e 31 testes Rust**, além da compilação TypeScript/Vite e do instalador NSIS. Esses números descrevem a suíte daquele estado do código; o resultado atual deve ser conferido no [CI](https://github.com/entaocadu/quota-dock/actions).

As verificações nativas Windows incluíram:

- Entrada pela direita e saída, com durações configuradas de 230 ms e 170 ms.
- Arrasto do título, inclusive com a fixação desativada.
- Recolhimento ao clicar fora, Esc e botão X.
- Fixação acima dos apps e permanência ao perder foco.
- Modo bandeja, modo barra de tarefas e restauração após minimizar.
- Reabertura durante uma saída pendente, sem ocultar a nova janela.
- Atualização da instalação com preferências preservadas.
- Consultas locais reais às fontes configuradas, sem publicar credenciais ou resultados de contas.

## O que os testes automatizados verificam

| Área | Exemplos de regressão coberta |
|---|---|
| Quotas | Campos ausentes não viram saldo cheio; janelas preservam durações da resposta |
| Contas | Uma conta não substitui a sessão de outra; fontes são reconciliadas pela identidade |
| OAuth | Estado, código, callbacks fragmentados e identidade de reconexão |
| Windows | DPAPI aceita a sessão válida e rejeita dados adulterados |
| Concorrência | Cem tarefas de conta com concorrência limitada e ordem estável |
| UI | Leitura antiga rotulada, animações canceladas e eventos antigos descartados |
| Janela | Geometria respeita área útil, DPI e coordenadas negativas |
| Migração | Novas opções não apagam preferências da versão compacta anterior |

## Limites das evidências

O teste de cem contas usa tarefas controladas, não cem contas reais autenticadas. Ele comprova a política de concorrência e ordenação, não uma SLA dos provedores.

As durações de animação são parâmetros de implementação. Não equivalem a benchmark p95 de abertura, tempo total de consulta ou garantia de desempenho em qualquer computador. Não há uma medição reproduzível de CPU, memória ou inicialização publicada para a série 0.3; valores antigos sem metodologia foram retirados da documentação atual.

O CI usa dados fictícios e não consulta contas reais. Mudanças externas em autenticação ou endpoints exigem verificação específica de integração.

## Como reproduzir

Siga [DEVELOPMENT](DEVELOPMENT.md). Para uma versão distribuída, repita o roteiro de janela acima no Windows e confira a origem, data e estado de cada conta. Qualquer resultado local de consumo deve ficar fora dos commits e issues.

[Voltar à documentação](README.md)
