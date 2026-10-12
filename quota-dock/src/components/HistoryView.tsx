import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { QuotaSnapshot } from '../contracts';

export const HistoryView: React.FC = () => {
  const [history, setHistory] = useState<QuotaSnapshot[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadHistory();
  }, []);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const data = await invoke<QuotaSnapshot[]>('get_history', { limit: 50 });
      setHistory(data);
    } catch (e) {
      console.error('Error fetching history:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleClear = async () => {
    if (window.confirm('Deseja realmente limpar todo o histórico local do QuotaDock?')) {
      await invoke('clear_history');
      setHistory([]);
    }
  };

  const handleExportJson = () => {
    const blob = new Blob([JSON.stringify(history, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `quotadock_history_${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h2 style={{ fontSize: '13px', fontWeight: 600 }}>Histórico Local</h2>
          <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
            Snapshots salvos no SQLite do QuotaDock
          </p>
        </div>
        <div style={{ display: 'flex', gap: '6px' }}>
          <button className="cta-btn" onClick={handleExportJson} disabled={history.length === 0}>
            Exportar JSON
          </button>
          <button
            className="cta-btn"
            onClick={handleClear}
            disabled={history.length === 0}
            style={{ color: 'var(--error)' }}
          >
            Limpar
          </button>
        </div>
      </div>

      {loading ? (
        <div style={{ textAlign: 'center', padding: '40px 0', color: 'var(--text-muted)', fontSize: '12px' }}>
          Carregando histórico...
        </div>
      ) : history.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '40px 0', color: 'var(--text-muted)', fontSize: '12px' }}>
          Nenhum snapshot arquivado ainda.
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {history.map((s, idx) => (
            <div
              key={idx}
              style={{
                background: 'var(--surface-card)',
                border: '1px solid var(--line)',
                borderRadius: '8px',
                padding: '8px 12px',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                fontSize: '11.5px',
              }}
            >
              <div>
                <div style={{ fontWeight: 600 }}>
                  {s.providerId === 'google-antigravity' ? 'Google Antigravity' : 'OpenAI Codex'}
                </div>
                <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>
                  {s.observedAt ? new Date(s.observedAt).toLocaleString() : 'Sem timestamp'}
                </div>
              </div>
              <div style={{ textAlign: 'right' }}>
                <span style={{ fontSize: '10px', textTransform: 'uppercase', padding: '2px 5px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)' }}>
                  {s.status}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
