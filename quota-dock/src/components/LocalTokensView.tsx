import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LocalTokens } from '../contracts';

export const LocalTokensView: React.FC = () => {
  const [range, setRange] = useState<'today' | '7d' | '30d'>('7d');
  const [tokens, setTokens] = useState<LocalTokens | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadTokens(range);
  }, [range]);

  const loadTokens = async (r: 'today' | '7d' | '30d') => {
    setLoading(true);
    try {
      const data = await invoke<LocalTokens | null>('get_local_tokens', { range: r });
      setTokens(data);
    } catch (e) {
      console.error('Error fetching local tokens:', e);
    } finally {
      setLoading(false);
    }
  };

  const formatNumber = (num: number | null | undefined) => {
    if (num === null || num === undefined) return '0';
    if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(2)}M`;
    if (num >= 1_000) return `${(num / 1_000).toFixed(1)}k`;
    return num.toLocaleString();
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h2 style={{ fontSize: '13px', fontWeight: 600 }}>Tokens Locais do OpenCode</h2>
          <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
            Estatísticas agregadas de atividade na máquina
          </p>
        </div>
        <div style={{ display: 'flex', background: 'var(--surface-elevated)', borderRadius: '6px', padding: '2px' }}>
          {(['today', '7d', '30d'] as const).map((r) => (
            <button
              key={r}
              onClick={() => setRange(r)}
              style={{
                background: range === r ? 'var(--line-strong)' : 'transparent',
                border: 'none',
                color: range === r ? 'var(--text-primary)' : 'var(--text-muted)',
                fontSize: '11px',
                fontWeight: range === r ? 600 : 400,
                padding: '4px 8px',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              {r === 'today' ? 'Hoje' : r === '7d' ? '7 dias' : '30 dias'}
            </button>
          ))}
        </div>
      </div>

      {loading ? (
        <div style={{ textAlign: 'center', padding: '40px 0', color: 'var(--text-muted)', fontSize: '12px' }}>
          Calculando tokens locais...
        </div>
      ) : tokens ? (
        <>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '8px',
            }}
          >
            <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px' }}>
              <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Total Consumido</div>
              <div style={{ fontSize: '18px', fontWeight: 700, color: 'var(--text-primary)', marginTop: '4px' }}>
                {formatNumber(tokens.total)}
              </div>
            </div>

            <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px' }}>
              <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Cache Leitura</div>
              <div style={{ fontSize: '18px', fontWeight: 700, color: 'var(--accent-openai)', marginTop: '4px' }}>
                {formatNumber(tokens.cacheRead)}
              </div>
            </div>

            <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px' }}>
              <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Entrada (Prompts)</div>
              <div style={{ fontSize: '15px', fontWeight: 600, color: 'var(--text-primary)', marginTop: '4px' }}>
                {formatNumber(tokens.input)}
              </div>
            </div>

            <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px' }}>
              <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Saída (Respostas)</div>
              <div style={{ fontSize: '15px', fontWeight: 600, color: 'var(--text-primary)', marginTop: '4px' }}>
                {formatNumber(tokens.output)}
              </div>
            </div>
          </div>

          {/* Model Breakdown */}
          {tokens.models.length > 0 && (
            <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px' }}>
              <div style={{ fontSize: '12px', fontWeight: 600, marginBottom: '8px' }}>Uso por Modelo</div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                {tokens.models.map((m, idx) => (
                  <div
                    key={idx}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      fontSize: '11.5px',
                      padding: '4px 0',
                      borderBottom: idx < tokens.models.length - 1 ? '1px solid var(--line)' : 'none',
                    }}
                  >
                    <span style={{ color: 'var(--text-secondary)', fontFamily: 'monospace' }}>{m.modelId}</span>
                    <span style={{ fontWeight: 600 }}>{formatNumber(m.tokens)}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div
            style={{
              padding: '10px',
              borderRadius: '8px',
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid var(--line)',
              fontSize: '11px',
              color: 'var(--text-muted)',
              lineHeight: '1.4',
            }}
          >
            ℹ️ <strong>Importante:</strong> Esta contagem representa a atividade registrada localmente pelo OpenCode CLI e não deve ser confundida com a cota restante da sua assinatura ou saldo de API.
          </div>
        </>
      ) : (
        <div style={{ textAlign: 'center', padding: '40px 0', color: 'var(--text-muted)', fontSize: '12px' }}>
          Não foi possível carregar estatísticas do OpenCode stats.
        </div>
      )}
    </div>
  );
};
