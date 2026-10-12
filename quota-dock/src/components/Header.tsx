import React from 'react';
import { invoke } from '@tauri-apps/api/core';

interface HeaderProps {
  lastUpdated: string | null;
  refreshing: boolean;
  onRefresh: () => void;
  pinned: boolean;
  onTogglePin: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  lastUpdated,
  refreshing,
  onRefresh,
  pinned,
  onTogglePin,
}) => {
  const handleClose = async () => {
    try {
      await invoke('hide_window');
    } catch (e) {
      console.error('Error hiding window:', e);
    }
  };

  return (
    <header className="header">
      <div className="header-title-group">
        <div className="logo-badge">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#ffffff" strokeWidth="2.5">
            <circle cx="12" cy="12" r="10" />
            <path d="m10 15 5-3-5-3v6z" />
          </svg>
        </div>
        <div>
          <h1 className="header-title">QuotaDock</h1>
        </div>
      </div>

      <div className="header-actions">
        {lastUpdated && (
          <span style={{ fontSize: '11px', color: 'var(--text-muted)', marginRight: '6px' }}>
            {lastUpdated}
          </span>
        )}
        <button
          className={`icon-btn ${refreshing ? 'spinning' : ''}`}
          onClick={onRefresh}
          title="Atualizar agora"
          disabled={refreshing}
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            style={{ animation: refreshing ? 'spin 1s linear infinite' : 'none' }}
          >
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
            <path d="M21 3v5h-5" />
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
            <path d="M8 16H3v5" />
          </svg>
        </button>

        <button
          className={`icon-btn ${pinned ? 'active' : ''}`}
          onClick={onTogglePin}
          title={pinned ? 'Desafixar janela' : 'Fixar janela visível'}
        >
          <svg width="13" height="13" viewBox="0 0 24 24" fill={pinned ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="2">
            <line x1="12" y1="17" x2="12" y2="22" />
            <path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a1 1 0 0 0 0-2H8a1 1 0 0 0 0 2h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z" />
          </svg>
        </button>

        <button className="icon-btn" onClick={handleClose} title="Ocultar na bandeja (Esc)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <style>{`
        @keyframes spin {
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </header>
  );
};
