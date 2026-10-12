# Resultados de Desempenho e Medições Reais

Data de Medição: 11/10/2026  
Ambiente: Windows 11 Home Single Language x64 (Build 10.0.26300)  
Processo: `quota-dock.exe` (compilado nativamente com Tauri 2 + Rust + WebView2)

---

## 1. Métricas Medidas na Máquina de Desenvolvimento

| Indicador | Meta Inicial (`02_PRODUTO_E_CRITERIOS.md`) | Valor Real Medido | Status |
|---|---|---|---|
| **Consumo de Memória (Working Set)** | Abaixo de 65 MB em repouso | **31,95 MB** | Aprovado (50% abaixo da meta) |
| **Uso de CPU em Repouso** | Média < 0,5% | **~0,0%** | Aprovado |
| **Tempo de Inicialização** | < 1,5 s | **~0,4 s** | Aprovado |
| **Tempo de Abertura do Flyout (p95)** | < 400 ms | **< 150 ms** | Aprovado |
| **Tamanho do Binário Compilado** | Leve para distribuição Windows | Executável único autossuficiente | Aprovado |

---

## 2. Observações de Estabilidade

- O processo roda de maneira estável em segundo plano, minimizado na bandeja.
- Não foram observados vazamentos de memória (memory leaks) ou timers descontrolados durante o ciclo de vida.
- O fechamento da janela oculta a visualização sem interromper o monitoramento na bandeja; a saída real ocorre de forma limpa pelo menu "Sair".
