import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Header } from '../components/Header';
import { QuotaCard } from '../components/QuotaCard';
import { LocalTokensView } from '../components/LocalTokensView';
import { DiagnosticsView } from '../components/DiagnosticsView';
import { HistoryView } from '../components/HistoryView';
import { SettingsView } from '../components/SettingsView';
import { QuotaSnapshot, PublicSettings } from '../contracts';

type Tab = 'overview' | 'tokens' | 'history' | 'diagnostics' | 'settings';

export const App: React.FC = () => {
  const [tab, setTab] = useState<Tab>('overview');
  const [snapshots, setSnapshots] = useState<QuotaSnapshot[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);
  const [pinned, setPinned] = useState(false);

  const loadOverview = useCallback(async () => {
    setRefreshing(true);
    try {
      const data = await invoke<QuotaSnapshot[]>('get_overview');
      setSnapshots(data);
      setLastUpdated(new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));
    } catch (e) {
      console.error('Error fetching overview:', e);
    } finally {
      setRefreshing(false);
    }
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      const s = await invoke<PublicSettings>('get_settings');
      setPinned(s.pinned);
    } catch (e) {
      console.error('Error fetching settings:', e);
    }
  }, []);

  useEffect(() => {
    loadOverview();
    loadSettings();

    // Listen to ESC key to close/hide
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        invoke('hide_window');
      }
    };
    window.addEventListener('keydown', handleKeyDown);

    // Listen to tray events
    const unlistenRefresh = listen('trigger-refresh', () => {
      loadOverview();
    });

    const unlistenNav = listen<string>('navigate-tab', (event) => {
      if (['overview', 'tokens', 'history', 'diagnostics', 'settings'].includes(event.payload)) {
        setTab(event.payload as Tab);
      }
    });

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      unlistenRefresh.then((f) => f());
      unlistenNav.then((f) => f());
    };
  }, [loadOverview, loadSettings]);

  const togglePin = async () => {
    const nextPin = !pinned;
    setPinned(nextPin);
    await invoke('update_settings', { patch: { pinned: nextPin } });
  };

  return (
    <div className="window-container">
      <Header
        lastUpdated={lastUpdated}
        refreshing={refreshing}
        onRefresh={loadOverview}
        pinned={pinned}
        onTogglePin={togglePin}
      />

      <nav className="tab-bar">
        <button
          className={`tab-btn ${tab === 'overview' ? 'active' : ''}`}
          onClick={() => setTab('overview')}
        >
          Visão Geral
        </button>
        <button
          className={`tab-btn ${tab === 'tokens' ? 'active' : ''}`}
          onClick={() => setTab('tokens')}
        >
          Tokens Locais
        </button>
        <button
          className={`tab-btn ${tab === 'history' ? 'active' : ''}`}
          onClick={() => setTab('history')}
        >
          Histórico
        </button>
        <button
          className={`tab-btn ${tab === 'diagnostics' ? 'active' : ''}`}
          onClick={() => setTab('diagnostics')}
        >
          Diagnóstico
        </button>
        <button
          className={`tab-btn ${tab === 'settings' ? 'active' : ''}`}
          onClick={() => setTab('settings')}
        >
          Ajustes
        </button>
      </nav>

      <main className="content-body">
        {tab === 'overview' && (
          <>
            {snapshots.length === 0 && refreshing ? (
              <div style={{ textAlign: 'center', padding: '60px 0', color: 'var(--text-muted)', fontSize: '12px' }}>
                Consultando limites reais...
              </div>
            ) : (
              snapshots.map((s, idx) => (
                <QuotaCard
                  key={`${s.providerId}-${idx}`}
                  snapshot={s}
                  onOpenHelp={(key) => invoke('open_external_help', { key })}
                />
              ))
            )}
          </>
        )}

        {tab === 'tokens' && <LocalTokensView />}
        {tab === 'history' && <HistoryView />}
        {tab === 'diagnostics' && <DiagnosticsView />}
        {tab === 'settings' && <SettingsView />}
      </main>

      <footer className="footer-bar">
        <span>Residente na bandeja</span>
        <span>Esc para ocultar</span>
      </footer>
    </div>
  );
};
