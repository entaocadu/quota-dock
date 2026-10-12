import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PublicSettings, SafeSettingsPatch } from '../contracts';

export const SettingsView: React.FC = () => {
  const [settings, setSettings] = useState<PublicSettings | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const data = await invoke<PublicSettings>('get_settings');
      setSettings(data);
    } catch (e) {
      console.error('Error fetching settings:', e);
    }
  };

  const updateSetting = async (patch: SafeSettingsPatch) => {
    try {
      const updated = await invoke<PublicSettings>('update_settings', { patch });
      setSettings(updated);
    } catch (e) {
      console.error('Error updating settings:', e);
    }
  };

  if (!settings) return null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
      <div>
        <h2 style={{ fontSize: '13px', fontWeight: 600 }}>Ajustes do QuotaDock</h2>
        <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Preferências locais e privacidade
        </p>
      </div>

      <div style={{ background: 'var(--surface-card)', border: '1px solid var(--line)', borderRadius: '10px', padding: '12px', display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {/* Startup */}
        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '12px', cursor: 'pointer' }}>
          <div>
            <div style={{ fontWeight: 500 }}>Iniciar com o Windows</div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>Executar em segundo plano na bandeja ao ligar o PC</div>
          </div>
          <input
            type="checkbox"
            checked={settings.startWithWindows}
            onChange={(e) => updateSetting({ startWithWindows: e.target.checked })}
          />
        </label>

        <div style={{ height: '1px', background: 'var(--line)' }} />

        {/* Polling Interval */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '12px' }}>
          <div>
            <div style={{ fontWeight: 500 }}>Intervalo de Atualização</div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>Frequência de leitura das fontes ativas</div>
          </div>
          <select
            value={settings.pollIntervalMins}
            onChange={(e) => updateSetting({ pollIntervalMins: Number(e.target.value) })}
            style={{
              background: 'var(--surface-elevated)',
              color: 'var(--text-primary)',
              border: '1px solid var(--line)',
              borderRadius: '6px',
              padding: '4px 8px',
              fontSize: '11px',
            }}
          >
            <option value={5}>5 minutos</option>
            <option value={10}>10 minutos</option>
            <option value={15}>15 minutos</option>
            <option value={30}>30 minutos</option>
          </select>
        </div>

        <div style={{ height: '1px', background: 'var(--line)' }} />

        {/* Notifications */}
        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '12px', cursor: 'pointer' }}>
          <div>
            <div style={{ fontWeight: 500 }}>Alertas de Limite Próximo</div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>Notificar quando cota restante estiver baixa</div>
          </div>
          <input
            type="checkbox"
            checked={settings.notificationsEnabled}
            onChange={(e) => updateSetting({ notificationsEnabled: e.target.checked })}
          />
        </label>

        <div style={{ height: '1px', background: 'var(--line)' }} />

        {/* Google Live Opt-In */}
        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', fontSize: '12px', cursor: 'pointer' }}>
          <div style={{ paddingRight: '12px' }}>
            <div style={{ fontWeight: 500, color: 'var(--warn)' }}>Consulta Live do Google (Experimental)</div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-muted)', lineHeight: '1.3', marginTop: '2px' }}>
              Utiliza endpoint não documentado comunitário. Desligado por padrão para total segurança da sua conta Google.
            </div>
          </div>
          <input
            type="checkbox"
            checked={settings.googleLiveOptIn}
            onChange={(e) => {
              if (e.target.checked) {
                if (window.confirm('Atenção: A consulta Live do Google usa rotas não oficiais que podem expor sua conta a riscos de ToS. Manter apenas o cache local é o modo recomendado. Deseja realmente ativar?')) {
                  updateSetting({ googleLiveOptIn: true });
                }
              } else {
                updateSetting({ googleLiveOptIn: false });
              }
            }}
          />
        </label>
      </div>

      <div
        style={{
          padding: '10px',
          borderRadius: '8px',
          background: 'rgba(255, 255, 255, 0.02)',
          border: '1px solid var(--line)',
          fontSize: '11px',
          color: 'var(--text-muted)',
          lineHeight: '1.4',
        }}
      >
        🔒 <strong>Privacidade do QuotaDock:</strong> Roda 100% localmente no seu computador. Não possui servidores próprios, não requer logins adicionais e nunca envia nem armazena suas credenciais ou código dos seus projetos.
      </div>
    </div>
  );
};
