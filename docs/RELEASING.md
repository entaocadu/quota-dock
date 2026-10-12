# Preparar uma release

Este guia é para o mantenedor. O instalador e o pacote portátil são publicados em [GitHub Releases](https://github.com/entaocadu/quota-dock/releases), separados do código-fonte. Uma alteração apenas de documentação não exige substituir o executável instalado nem mover tags existentes.

## Antes de compilar

1. Confirme o commit, a branch, a versão e o estado limpo do checkout.
2. Para uma nova versão do app, atualize `quota-dock/package.json`, seu lockfile, `src-tauri/Cargo.toml`, `Cargo.lock` e `tauri.conf.json`.
3. Atualize [CHANGELOG](../CHANGELOG.md) e descreva comportamentos verificáveis.
4. Execute os comandos de [desenvolvimento](DEVELOPMENT.md): documentação, testes e build.
5. Revise dados sensíveis no diff e no histórico alcançável. Um scanner de segredos ajuda, mas a revisão de fixtures, imagens, logs e arquivos locais também é necessária.

## Preparar os downloads

Compile com `npm run tauri build -- --bundles nsis`. Nomeie os arquivos como `QuotaDock-<versão>-Setup.exe` e `QuotaDock-<versão>-Portable.zip`.

No portátil GNU, inclua somente `QuotaDock.exe`, `WebView2Loader.dll` e instruções de uso. Nunca compacte a pasta do perfil Windows, `%APPDATA%\QuotaDock`, `.tools/`, arquivos de login, bancos ou resultados de consultas.

Gere `SHA256SUMS.txt` para o instalador e o ZIP. Exemplo na pasta que contém os downloads:

```powershell
$downloads = @('QuotaDock-0.3.0-Setup.exe', 'QuotaDock-0.3.0-Portable.zip')
$lines = foreach ($file in $downloads) {
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash.ToLowerInvariant()
    "$hash  $file"
}
$lines | Set-Content -Encoding ascii -LiteralPath .\SHA256SUMS.txt
```

## Validar no Windows

Instale ou execute o pacote na mesma condição que será distribuída. Verifique a bandeja, recolhimento, fixação, barra de tarefas, arrasto, reabertura e altura da lista. Confira também o requisito WebView2 e a inclusão do loader GNU.

A consulta de contas reais fica no ambiente local; registre apenas a aprovação da integração, sem publicar identidades ou saldos. Confira os [limites das evidências](VALIDATION.md).

## Publicar

- Crie uma tag `v<versão>` para o commit correspondente; não reutilize ou mova uma tag já distribuída.
- Prepare a release como rascunho, com resumo, mudanças, requisitos, instruções e significado das quotas OpenAI.
- Anexe instalador, ZIP e `SHA256SUMS.txt`.
- Confira tamanhos e hashes dos arquivos enviados, publique e confirme o acesso aos downloads.
- Marque a versão atual como mais recente; versões anteriores continuam identificadas pelo número.

Relatos de vulnerabilidade devem seguir [SECURITY](../SECURITY.md). Logs de publicação não podem imprimir tokens GitHub ou credenciais dos provedores.
