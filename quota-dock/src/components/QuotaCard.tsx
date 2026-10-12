import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { QuotaSnapshot, Availability } from '../contracts';

interface QuotaCardProps {
  snapshot: QuotaSnapshot;
  onOpenHelp?: (key: string) => void;
}

export const QuotaCard: React.FC<QuotaCardProps> = ({ snapshot, onOpenHelp }) => {
  const isGoogle = snapshot.providerId === 'google-antigravity';
  const providerTitle = isGoogle ? 'GOOGLE / ANTIGRAVITY' : 'OPENAI / CODEX';
  const accentClass = isGoogle ? 'google' : 'openai';

  const formatReset = (isoStr: string | null) => {
    if (!isoStr) return 'Sem previsão';
    try {
      const dt = new Date(isoStr);
      if (isNaN(dt.getTime())) return 'Sem previsão';
      const now = new Date();
      const diffMs = dt.getTime() - now.getTime();
      const timeStr = dt.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      
      if (diffMs > 0) {
        const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
        const diffMins = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));
        if (diffHours > 24) {
          const days = Math.floor(diffHours / 24);
          return `Renova em ${days}d (${timeStr})`;
        }
        if (diffHours > 0) {
          return `Renova às ${timeStr} (em ${diffHours}h ${diffMins}m)`;
        }
        return `Renova às ${timeStr} (em ${diffMins}m)`;
      } else {
        return `Horário de renovação: ${timeStr} (expirado)`;
      }
    } catch {
      return 'Sem previsão';
    }
  };

  const getStatusBadge = (status: Availability) => {
    switch (status) {
      case 'ok':
        return <span className="badge badge-ok">CONFIRMADO</span>;
      case 'stale':
        return <span className="badge badge-stale">DESATUALIZADO</span>;
      case 'rate_limited':
        return <span className="badge badge-err">LIMITE ATINGIDO</span>;
      case 'unauthorized_scope':
        return <span className="badge badge-err">401 SEM PERMISSÃO</span>;
      case 'forbidden':
        return <span className="badge badge-err">403 BLOQUEADO</span>;
      case 'not_installed':
        return <span className="badge badge-neutral">NÃO INSTALADO</span>;
      case 'no_data':
        return <span className="badge badge-neutral">SEM SNAPSHOT</span>;
      default:
        return <span className="badge badge-neutral">{status}</span>;
    }
  };

  const handleOpenHelp = () => {
    if (onOpenHelp) {
      onOpenHelp(isGoogle ? 'google-help' : 'codex-usage');
    } else {
      invoke('open_external_help', { key: isGoogle ? 'google-help' : 'codex-usage' });
    }
  };

  return (
    <div className="quota-card">
      <div className="card-header">
        <div className="provider-info">
          <span className="provider-name">{providerTitle}</span>
          <span className="account-label">{snapshot.accountLabel}</span>
        </div>
        <div>{getStatusBadge(snapshot.status)}</div>
      </div>

      {/* Warnings & Notes */}
      {snapshot.warnings && snapshot.warnings.length > 0 && (
        <div className={snapshot.status === 'stale' ? 'warn-box' : 'err-box'}>
          {snapshot.warnings.map((w, idx) => (
            <div key={idx} style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
              <span>⚠️</span>
              <span>{w}</span>
            </div>
          ))}
        </div>
      )}

      {/* Codex Windows */}
      {!isGoogle && snapshot.windows.length > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {snapshot.windows.map((win) => {
            const hasValue = win.remainingPercent !== null && win.remainingPercent !== undefined;
            const remaining = hasValue ? Math.round(win.remainingPercent!) : null;
            const isLow = remaining !== null && remaining <= 25 && remaining > 10;
            const isCritical = remaining !== null && remaining <= 10;

            return (
              <div key={win.id} className="quota-item">
                <div className="quota-item-header">
                  <span className="quota-label">{win.label}</span>
                  <span
                    className="quota-value"
                    style={{
                      color: isCritical
                        ? 'var(--error)'
                        : isLow
                        ? 'var(--warn)'
                        : 'var(--text-primary)',
                    }}
                  >
                    {hasValue ? `${remaining}% restante` : 'Indisponível'}
                  </span>
                </div>

                <div className="quota-progress-track">
                  {hasValue && (
                    <div
                      className={`quota-progress-fill ${accentClass} ${
                        isCritical ? 'critical' : isLow ? 'low' : ''
                      }`}
                      style={{ width: `${Math.max(0, Math.min(100, remaining!))}%` }}
                    />
                  )}
                </div>

                <div className="quota-item-footer">
                  <span>{formatReset(win.resetAt)}</span>
                  <span>{win.usedPercent !== null ? `Usado: ${Math.round(win.usedPercent)}%` : ''}</span>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Google Models */}
      {isGoogle && snapshot.models.length > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {snapshot.models.map((mod) => {
            const hasValue = mod.remainingPercent !== null && mod.remainingPercent !== undefined;
            const remaining = hasValue ? Math.round(mod.remainingPercent!) : null;
            const isLow = remaining !== null && remaining <= 25 && remaining > 10;
            const isCritical = remaining !== null && remaining <= 10;

            return (
              <div key={mod.modelId} className="quota-item">
                <div className="quota-item-header">
                  <span className="quota-label">{mod.displayName}</span>
                  <span
                    className="quota-value"
                    style={{
                      color: isCritical
                        ? 'var(--error)'
                        : isLow
                        ? 'var(--warn)'
                        : 'var(--text-primary)',
                    }}
                  >
                    {hasValue ? `${remaining}% restante` : 'Indisponível'}
                  </span>
                </div>

                <div className="quota-progress-track">
                  {hasValue && (
                    <div
                      className={`quota-progress-fill ${accentClass} ${
                        isCritical ? 'critical' : isLow ? 'low' : ''
                      }`}
                      style={{ width: `${Math.max(0, Math.min(100, remaining!))}%` }}
                    />
                  )}
                </div>

                <div className="quota-item-footer">
                  <span>{formatReset(mod.resetAt)}</span>
                  <span>{mod.groupLabel}</span>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Empty State / CTA if unavailable */}
      {snapshot.windows.length === 0 && snapshot.models.length === 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', padding: '6px 0' }}>
          <p style={{ fontSize: '11.5px', color: 'var(--text-secondary)' }}>
            {snapshot.status === 'unauthorized_scope'
              ? 'A autenticação atual do editor não possui permissão para ler o endpoint de cotas.'
              : 'Nenhuma quota pôde ser consultada por esta fonte no momento.'}
          </p>
          <button className="cta-btn" onClick={handleOpenHelp}>
            <span>Abrir Painel Oficial</span>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
          </button>
        </div>
      )}
    </div>
  );
};
