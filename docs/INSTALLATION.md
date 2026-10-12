# Instalação e atualização

## Requisitos para usar

- Windows 10/11 x64. A implementação de sessões próprias usa a proteção Windows DPAPI; não há distribuição Linux ou macOS nesta versão.
- Microsoft WebView2 Runtime. O instalador Tauri pode solicitar a instalação do runtime quando necessário. [Download oficial Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/).
- Acesso à internet para consultar os provedores.
- OpenCode e as contas desejadas configurados no mesmo usuário do Windows. Para Google, o plugin Antigravity deve ter salvado suas contas. Veja [contas e configurações](CONFIGURATION.md).

Node.js, Rust e o executável Codex não são requisitos para executar a versão distribuída.

## Instalar

1. Abra a [release mais recente](https://github.com/entaocadu/quota-dock/releases/latest).
2. Baixe `QuotaDock-0.3.0-Setup.exe` e, se desejar verificar a integridade, `SHA256SUMS.txt`.
3. Execute o instalador no seu usuário Windows.
4. Procure o ícone do QuotaDock na bandeja, incluindo o menu de apps ocultos. A inicialização normal fica recolhida.
5. Clique no ícone para abrir e confira as fontes das contas.

O destino padrão do instalador é `%LOCALAPPDATA%\QuotaDock`. A opção de iniciar com o Windows pode ser alterada em **Ajustes**.

## Pacote portátil

Extraia `QuotaDock-0.3.0-Portable.zip` para uma pasta permanente. Execute `QuotaDock.exe` e mantenha `WebView2Loader.dll` ao lado. O runtime WebView2 ainda precisa estar instalado no Windows.

Se mover a pasta depois de habilitar a inicialização automática, desative e reative **Iniciar com o Windows** para registrar o novo caminho. O pacote portátil usa o mesmo armazenamento local do usuário; não guarda tokens dentro da pasta do executável.

## Conferir o download

Na pasta do arquivo baixado, execute:

```powershell
Get-FileHash -Algorithm SHA256 -LiteralPath .\QuotaDock-0.3.0-Setup.exe
```

Compare o campo `Hash` com a linha correspondente em `SHA256SUMS.txt`. Faça o mesmo para o ZIP, quando usar a distribuição portátil. Diferenças exigem baixar novamente o arquivo correto.

## Atualizar

Encerre pelo menu da bandeja **Sair** e execute o instalador da versão nova. Se for solicitado, remova a instalação anterior antes de instalar a nova. O diretório de dados `%APPDATA%\QuotaDock` deve ser mantido para preservar preferências e sessões próprias.

A série 0.3 mantém as preferências da série 0.2. A migração inicial de versões anteriores ao painel compacto redefine alguns padrões, como o intervalo de consulta e a inicialização automática.

O aplicativo não possui atualização automática embutida nesta versão. Downloads são publicados em [Releases](https://github.com/entaocadu/quota-dock/releases).

## Encerrar e desinstalar

Esc e X apenas recolhem. Use **Sair** no menu da bandeja para encerrar o processo e, depois, o desinstalador do Windows. O armazenamento local do QuotaDock pode permanecer para permitir uma reinstalação. Não exclua bancos ou arquivos de login do OpenCode para desinstalar o QuotaDock.

[Voltar à documentação](README.md)
