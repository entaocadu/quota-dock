# Desenvolvimento no Windows

Execute os comandos abaixo a partir da pasta `quota-dock/` do repositório, salvo quando outro diretório for indicado.

## Pré-requisitos

- **Node.js 24** com npm. Vitest 5 exige `^22.12.0 || ^24.0.0 || >=26.0.0`; Node.js 20 não atende à suíte atual. Node 24 é a versão usada no CI.
- Rust estável, Cargo e uma toolchain Windows x64.
- Microsoft WebView2 Runtime.
- MSVC com as ferramentas de compilação C++ e Windows SDK, ou MinGW GNU compatível com o Rust selecionado. Consulte os [pré-requisitos oficiais Tauri](https://tauri.app/start/prerequisites/).

A distribuição 0.3 foi validada com GNU/MinGW; o CI usa o ambiente Windows MSVC. Não misture uma toolchain Rust GNU com um linker MSVC ou vice-versa.

## Preparar

```powershell
git clone https://github.com/entaocadu/quota-dock.git
cd quota-dock\quota-dock
npm ci
```

Use os lockfiles versionados. Não copie arquivos de autenticação para dentro do checkout.

## Executar durante desenvolvimento

```powershell
npm run tauri -- dev -- -- --show
```

O `beforeDevCommand` compila a interface e o Tauri carrega os arquivos estáticos. Esse modo não configura HMR automático. Para trabalhar somente na interface no navegador, `npm run dev` inicia Vite, mas as chamadas nativas Tauri exigem o aplicativo ou mocks nos testes.

Sem `--show`, o aplicativo começa recolhido. Use a bandeja para abri-lo. As consultas podem acessar as contas do usuário; o exemplo de consultas reais não faz parte do CI.

## Verificar

```powershell
npm test
npm run build
npm run test:rust
```

`npm test` executa Vitest. `npm run build` faz a checagem TypeScript e gera a interface. O wrapper Rust compila os testes, embute um manifesto Common Controls v6 nos executáveis de teste Windows e os executa. Isso evita a falha de importação `TaskDialogIndirect` observada no harness GNU.

Na raiz do repositório, valide os links da documentação:

```powershell
node .github/scripts/check-docs.mjs
```

## Compilar a distribuição

```powershell
npm run tauri build -- --bundles nsis
```

| Saída | Caminho dentro do aplicativo |
|---|---|
| Executável | `src-tauri/target/release/quota-dock.exe` |
| Instalador NSIS | `src-tauri/target/release/bundle/nsis/` |
| Loader GNU, quando gerado | `src-tauri/target/release/WebView2Loader.dll` |

O pacote portátil GNU deve incluir o loader. O runtime WebView2 continua sendo um requisito do sistema. Veja [preparar uma release](RELEASING.md).

## Consultar contas reais para diagnóstico local

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --example verify_sources
./scripts/run-rust-tests.ps1 -ExistingExecutable src-tauri/target/debug/examples/verify_sources.exe
```

O exemplo exige fontes locais configuradas, consulta os provedores e retorna DTOs sem credenciais. Os resultados ainda contêm metadados de conta e consumo, portanto não publique a saída ou redirecione-a para um arquivo versionado. A ausência de uma fonte ou quota causa falha neste exemplo.

## Mapa do código

- `src/app/`: composição da UI, eventos e animação.
- `src/components/`: cartões, cabeçalho e ajustes; há componentes legados fora da tela principal.
- `src/contracts/`: tipos públicos da interface.
- `src-tauri/src/providers/`: consultas, descoberta de credenciais, autenticação própria e parsers legados.
- `src-tauri/src/tray/`: posicionamento, arrasto, transições e bandeja.
- `src-tauri/src/storage/`: preferências e histórico.
- `src-tauri/src/security/`: mascaramento e identificadores derivados.
- `src-tauri/gen/`: schemas gerados por Tauri; não representam código escrito à mão.

[Arquitetura](ARCHITECTURE.md) · [Contribuição](../CONTRIBUTING.md) · [Documentação](README.md)
