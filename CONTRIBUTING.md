# Como contribuir

O QuotaDock é um complemento do OpenCode para monitorar quotas no Windows. Contribuições devem manter esse propósito e identificar corretamente a fonte dos dados.

## Antes de começar

- Para bugs, abra uma [issue](https://github.com/entaocadu/quota-dock/issues/new/choose) com versão e passos reproduzíveis.
- Para provedores ou mudanças de fluxo, explique a fonte de quota disponível e o que ela mede.
- Para vulnerabilidades, use [SECURITY](SECURITY.md), evitando uma issue pública com detalhes sensíveis.

## Preparar e verificar

Siga [DEVELOPMENT](docs/DEVELOPMENT.md). O aplicativo fica em `quota-dock/`; a documentação principal fica na raiz e em `docs/`.

Use uma branch para a contribuição e os lockfiles existentes. Antes do PR, execute as verificações proporcionais à mudança:

```powershell
# Na raiz
node .github/scripts/check-docs.mjs

# Na pasta quota-dock/
npm test
npm run build
npm run test:rust
```

Uma alteração de integração exige testes do parser, identidade e falhas relevantes. Uma mudança de janela exige verificação nativa Windows. Documentação deve corresponder ao comportamento exposto pelo aplicativo.

## Regras para dados e quotas

- Use fixtures fictícias e domínios como `example.com`. Não copie IDs de contas de respostas reais.
- Não envie tokens, senhas, `.env` real, arquivos de autenticação, SQLite, backups ou resultados locais.
- Não exponha valores privados em screenshots, comentários ou saída de testes.
- Não invente limites, janelas, percentuais ou nomes de modelos quando o provedor não os informou.
- Preserve a distinção entre quota Codex da conta e um limite específico de compartilhamento do OpenCode.
- Documente novos endpoints, escopos e mudanças de armazenamento em [PROVIDERS](docs/PROVIDERS.md) e [PRIVACY](docs/PRIVACY.md).

## Pull requests

Descreva o problema, o comportamento resultante e a validação. Inclua screenshots sanitizados quando a UI mudar. Evite alterações de lógica em um PR apenas de documentação ou visual. Novas versões devem atualizar [CHANGELOG](CHANGELOG.md).

Ao contribuir, você permite distribuir sua contribuição sob a [licença MIT](LICENSE) do projeto. Componentes e marcas de terceiros conservam seus próprios termos.
