# QuotaDock

**QuotaDock** é um monitor independente de limites e quotas de IA para Windows (Windows 11 / Windows 10 x64), residente na bandeja do sistema (system tray), com visual minimalista estilo carbono/Apple-like, construído nativamente com **Tauri 2**, **Rust** e **React 19**.

Diferente de extensões ou plugins acoplados ao editor, o QuotaDock funciona de forma desacoplada de projetos: ele monitora as quotas reais das suas contas conectadas de **OpenAI Codex** e **Google Antigravity**, sem exigir chaves de API pagas adicionais e sem quebrar suas autenticações existentes no OpenCode.

---

## 🌟 Principais Recursos

- 🪟 **Residente na Bandeja (Tray Icon):** Fica discretamente na área de notificação do Windows. Clique com o botão esquerdo para abrir/fechar o flyout flutuante; clique com o botão direito para menu de contexto rápido (Atualizar, Diagnóstico, Sair).
- ⚡ **Consumo Mínimo de Recursos:** Menos de 35 MB de consumo de memória em repouso e consumo de CPU ~0%.
- 📊 **Cotas Reais OpenAI Codex:**
  - Leitura via stdio do runtime oficial `codex app-server`.
  - Exibe tanto a janela de curto prazo (5 horas) quanto a janela semanal (7 dias).
  - Mostra percentuais restantes autênticos, contadores de tempo para renovação (reset em horário local) e créditos de reset disponíveis.
- 🌐 **Cotas Google Antigravity:**
  - Leitura estritamente segura e *read-only* do snapshot local gerado pelo plugin `@pieliesdie/opencode-antigravity-auth` (`antigravity-accounts.json`).
  - Apresenta cotas separadas por modelo/família: **Gemini 3.8 Flash**, **Gemini Pro** e **Claude**.
  - Detecção inteligente de carimbo de tempo: indica se o dado é recente ou desatualizado (*stale*).
- 🔢 **Tokens Locais OpenCode:**
  - Painel dedicado mostrando tokens de entrada, saída, reasoning e leitura de cache (`opencode stats --json`).
  - Separação clara entre *tokens locais consumidos* e *quota restante da assinatura*.
- 🛡️ **Privacidade & Segurança por Padrão:**
  - 100% local, zero backend remoto próprio.
  - Mascaramento de dados sensíveis (`u***@domain.com`) e hashes determinísticos locais para identificação.
  - Nunca exporta nem altera senhas, refresh tokens ou bancos de terceiros.
- ⚙️ **Configurável:** Início automático com o Windows opcional, intervalo de atualização customizável, alertas visuais de quota baixa (25%, 10%) e histórico local exportável em JSON.

---

## 🚀 Como Executar

### Pré-requisitos
- Windows 10/11 x64 com WebView2 Runtime (já nativo no Windows 11).
- OpenCode v2.0+ e/ou Codex CLI instalados para leitura dos dados locais.

### Executando em Modo de Desenvolvimento
```bash
# 1. Instalar dependências da UI
npm install

# 2. Iniciar o app completo (Vite + Tauri)
npm run tauri dev
```

### Compilando o Executável Nativo do Windows
```bash
# Build do frontend
npm run build

# Build do executável de release com Rust
cd src-tauri
cargo build --release
```
O executável compilado estará localizado em `src-tauri/target/release/quota-dock.exe`.

---

## 📖 Arquitetura e Estrutura de Pastas

```text
quota-dock/
├── src/                     # Frontend React 19 + TypeScript
│   ├── app/                 # Shell da aplicação e lifecycle
│   ├── components/          # QuotaCard, Header, LocalTokensView, DiagnosticsView, etc.
│   ├── contracts/           # DTOs canônicos tipados
│   ├── styles/              # Design tokens e CSS carbono
│   └── tests/               # Testes unitários Vitest
├── src-tauri/               # Backend nativo Rust (Tauri 2)
│   ├── src/
│   │   ├── contracts/       # DTOs Serde serializados
│   │   ├── providers/       # Adaptadores OpenAI, Google Antigravity e OpenCode
│   │   ├── quota/           # Normalizadores matemáticos e cálculo de fusos
│   │   ├── security/        # Mascaramento de dados e sanitização
│   │   ├── storage/         # Banco SQLite isolado do QuotaDock
│   │   └── tray/            # Ícone da bandeja e menus nativos
│   └── icons/               # Ícones nativos (ico, pngs)
└── docs/                    # Especificações técnicas e relatórios
```

---

## 📄 Licença

Consulte `LICENSE_NOTICES.md` para atribuições e termos de bibliotecas terceiras utilizadas.
