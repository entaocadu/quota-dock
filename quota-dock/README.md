# Aplicativo QuotaDock

Esta pasta contém a interface React, o backend Rust/Tauri, os testes e os scripts do monitor de quotas para OpenCode.

Leia o [README principal](../README.md) para apresentação, prints e downloads. Consulte [desenvolvimento Windows](../docs/DEVELOPMENT.md), [arquitetura](../docs/ARCHITECTURE.md) e [contribuição](../CONTRIBUTING.md) para trabalhar no código.

```powershell
npm ci
npm test
npm run build
npm run test:rust
```

Requer Node.js 24 e uma toolchain Rust Windows compatível. Os detalhes de compilação e runtime estão no guia de desenvolvimento.
