import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SanitizedDiagnostics } from '../contracts';

export const DiagnosticsView: React.FC = () => {
  const [diag, setDiag] = useState<SanitizedDiagnostics | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    loadDiag();
  }, []);

  const loadDiag = async () => {
    try {
      const data = await invoke<SanitizedDiagnostics>('get_diagnostics');
      setDiag(data);
    } catch (e) {
      console.error('Error getting diagnostics:', e);
    }
  };

  const handleCopy = () => {
    if (!diag) return;
    navigator.clipboard.writeText(JSON.stringify(diag, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (!diag) {
    return (
      <div style={{ textAlign: 'center', padding: '40px 0', color: 'var(--text-muted)', fontSize: '12px' }}>
        Carregando diagnósticos seguros...
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h2 style={{ fontSize: '13px', fontWeight: 600 }}>Diagnóstico do Sistema</h2>
          <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
            Estado das integrações (sem expor credenciais)
          </p>
        </div>
        <button className="cta-btn" onClick={handleCopy}>
          {copied ? 'Copiado!' : 'Copiar Diagnóstico'}
        </button>
      </div>

      {/* Integrations Checklist */}
      <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px' }}>
        <div style={{ fontSize: '12px', fontWeight: 600, marginBottom: '8px' }}>Integrações Locais</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11.5px' }}>
            <span>Codex CLI (app-server)</span>
            <span style={{ color: diag.codexCliDetected ? 'var(--accent-openai)' : 'var(--error)' }}>
              {diag.codexCliDetected ? '✓ Detectado' : '✗ Não detectado'}
            </span>
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11.5px' }}>
            <span>Cache Antigravity (accounts.json)</span>
            <span style={{ color: diag.antigravityAccountsDetected ? 'var(--accent-openai)' : 'var(--error)' }}>
              {diag.antigravityAccountsDetected ? '✓ Presente' : '✗ Ausente'}
            </span>
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11.5px' }}>
            <span>OpenCode SQLite DB</span>
            <span style={{ color: diag.opencodeDbDetected ? 'var(--accent-openai)' : 'var(--warn)' }}>
              {diag.opencodeDbDetected ? '✓ Detectado' : '– Não encontrado no padrão'}
            </span>
          </div>
        </div>
      </div>

      {/* Provider Details */}
      <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px' }}>
        <div style={{ fontSize: '12px', fontWeight: 600, marginBottom: '8px' }}>Status dos Provedores</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          {diag.providers.map((p, idx) => (
            <div
              key={idx}
              style={{
                background: 'rgba(255, 255, 255, 0.02)',
                border: '1px solid var(--line)',
                borderRadius: '8px',
                padding: '8px 10px',
                fontSize: '11px',
                display: 'flex',
                flexDirection: 'column',
                gap: '4px',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', fontWeight: 600 }}>
                <span>{p.providerId.toUpperCase()}</span>
                <span style={{ textTransform: 'uppercase' }}>{p.status}</span>
              </div>
              <div style={{ color: 'var(--text-muted)' }}>Origem: {p.origin}</div>
              {p.lastSuccess && <div style={{ color: 'var(--text-muted)' }}>Último sucesso: {new Date(p.lastSuccess).toLocaleTimeString()}</div>}
              {p.errorCode && <div style={{ color: 'var(--error)' }}>Erro: {p.errorCode}</div>}
            </div>
          ))}
        </div>
      </div>

      <div style={{ fontSize: '10.5px', color: 'var(--text-muted)', lineHeight: '1.4' }}>
        QuotaDock v{diag.appVersion} • {diag.os} • Consulta em {new Date(diag.timestamp).toLocaleTimeString()}
      </div>
    </div>
  );
};
