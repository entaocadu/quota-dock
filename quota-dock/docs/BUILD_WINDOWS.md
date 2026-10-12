# Instruções de Build no Windows

Guia reproduzível para compilar o **QuotaDock** no Windows 10/11 x64.

## 1. Pré-requisitos de Sistema

- **Sistema Operacional:** Windows 10 x64 (com WebView2) ou Windows 11 x64.
- **Node.js:** Versão 20+ ou 24+ LTS.
- **Rust Toolchain:**
  - `rustc` e `cargo` 1.80+ (testado e homologado com `rustc 1.98.1`).
  - Toolchain MSVC (`x86_64-pc-windows-msvc`) ou MinGW GNU (`x86_64-pc-windows-gnu` com GCC 16+ / UCRT).
- **Microsoft Edge WebView2 Runtime:** Instalado por padrão no Windows 11. Para Windows 10, disponível no [site oficial da Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

## 2. Passo a Passo de Compilação

### Passo 2.1 — Instalar Dependências NPM
No diretório raiz do projeto (`quota-dock`):
```powershell
npm install
```

### Passo 2.2 — Executar Testes Automatizados
```powershell
# Testes do Frontend (TypeScript / Vitest)
npm test

# Testes do Backend (Rust / Cargo)
cd src-tauri
cargo test
cd ..
```

### Passo 2.3 — Gerar o Build de Produção
```powershell
# 1. Compila os assets do frontend React para dist/
npm run build

# 2. Compilar o executável de release com otimizações
cd src-tauri
cargo build --release
```

O binário final estará em:
`quota-dock\src-tauri\target\release\quota-dock.exe`

## 3. Empacotamento do Instalador Windows (NSIS)

Para gerar o instalador `.exe` (NSIS):
```powershell
npm run tauri build
```
O instalador será gerado na pasta:
`quota-dock\src-tauri\target\release\bundle\nsis\`

## 4. Notas sobre Assinatura de Código (Code Signing)

Caso você não possua um certificado digital comercial (EV/OV) instalado, o Windows SmartScreen pode apresentar um aviso informativo na primeira execução do executável gerado. Isso é esperado em builds locais; para executar, basta clicar em "Mais informações" -> "Executar assim mesmo".
