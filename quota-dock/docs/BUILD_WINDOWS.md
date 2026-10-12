# Build Windows

Requisitos: Node.js 20+, Rust, WebView2 e toolchain MSVC ou MinGW GNU no PATH. Validado nesta máquina com Rust 1.99.0 GNU, GCC 16.2 UCRT e Node.js instalado.

```powershell
npm ci
npm test
npm run build
npm run test:rust
npm run tauri build -- --bundles nsis
```

Instalador: src-tauri/target/release/bundle/nsis/. Executável: src-tauri/target/release/quota-dock.exe. Para distribuição GNU portátil, inclua WebView2Loader.dll que o build gera ao lado do executável.

O harness de testes GNU recebe um manifesto mínimo do Rust que omite Common Controls v6. Isso impede importar TaskDialogIndirect. scripts/run-rust-tests.ps1 compila os testes, embute o manifesto adequado e coloca o WebView2Loader ao lado dos testes. O script recusa executáveis fora do target do projeto.

Consulta real, sem imprimir credenciais:
```powershell
cargo build --manifest-path src-tauri/Cargo.toml --example verify_sources
./scripts/run-rust-tests.ps1 -ExistingExecutable src-tauri/target/debug/examples/verify_sources.exe
```

O exemplo valida as fontes configuradas e mostra apenas snapshots públicos. Não copie resultados pessoais para commits.

Inicialização: sem argumentos ou com --background, fica oculto. --show abre o popup para uma verificação manual. O menu Sair encerra o processo; X, Esc e perda de foco apenas ocultam a janela.
