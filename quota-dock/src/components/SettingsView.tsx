import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PublicSettings, SafeSettingsPatch } from "../contracts";
export const SettingsView: React.FC = () => {
  const [settings, setSettings] = useState<PublicSettings | null>(null),
    [error, setError] = useState<string | null>(null),
    [saving, setSaving] = useState(false);
  const [connections, setConnections] = useState<
    { accountKey: string; accountLabel: string }[]
  >([]);
  const [connecting, setConnecting] = useState(false);
  const [loginMessage, setLoginMessage] = useState<string | null>(null);
  useEffect(() => {
    void invoke<PublicSettings>("get_settings")
      .then(setSettings)
      .catch(() => setError("Não foi possível carregar os ajustes."));
    void invoke<typeof connections>("openai_connections")
      .then(setConnections)
      .catch(() => {});
    const pending = listen<string>("openai-login-state", (e) => {
      setConnecting(false);
      setLoginMessage(e.payload);
      void invoke<typeof connections>("openai_connections").then(
        setConnections,
      );
    });
    return () => {
      void pending.then((f) => f());
    };
  }, []);
  const connect = async (accountKey?: string) => {
    setConnecting(true);
    setLoginMessage(
      "Conclua a conexão no navegador com a mesma conta do OpenCode.",
    );
    try {
      await invoke("connect_openai", { accountKey: accountKey ?? null });
    } catch (e) {
      setConnecting(false);
      setLoginMessage(
        typeof e === "string" ? e : "Não foi possível iniciar a conexão.",
      );
    }
  };
  const update = async (patch: SafeSettingsPatch) => {
    setSaving(true);
    try {
      setSettings(await invoke<PublicSettings>("update_settings", { patch }));
      setError(null);
    } catch {
      setError("Não foi possível salvar. Tente novamente.");
    } finally {
      setSaving(false);
    }
  };
  return (
    <div className="settings-view">
      <h2>Ajustes</h2>
      {error && (
        <p className="error-note" role="alert">
          {error}
        </p>
      )}
      {settings && (
        <>
          <label className="setting-row">
            <span>
              Iniciar com o Windows
              <small>Em segundo plano, apenas na bandeja.</small>
            </span>
            <input
              type="checkbox"
              disabled={saving}
              checked={settings.startWithWindows}
              onChange={(e) =>
                void update({ startWithWindows: e.target.checked })
              }
            />
          </label>
          <label className="setting-row">
            <span>
              Atualização automática
              <small>Também consulta ao abrir a janela.</small>
            </span>
            <select
              disabled={saving}
              value={settings.pollIntervalMins}
              onChange={(e) =>
                void update({ pollIntervalMins: Number(e.target.value) })
              }
            >
              <option value={1}>1 min</option>
              <option value={2}>2 min</option>
              <option value={5}>5 min</option>
              <option value={10}>10 min</option>
            </select>
          </label>
          <p className="source-note">
            As consultas vão diretamente à Google e à OpenAI. O QuotaDock não
            envia seus dados a um servidor próprio.
          </p>
          <p className="source-note">
            OpenAI:{" "}
            {connections.length > 0
              ? `${connections.length} conexão(ões) própria(s), com renovação automática.`
              : "leitura do login local da mesma conta, sem executar o Codex."}{" "}
            Limites específicos do compartilhamento com o OpenCode podem ser
            diferentes.
          </p>
          <div>
            <button
              className="connect-btn"
              onClick={() => void connect()}
              disabled={connecting}
            >
              {connecting ? "Aguardando navegador…" : "Adicionar conta OpenAI"}
            </button>
            {connections.length === 0 && (
              <p className="source-note">
                Conecte cada conta uma vez para renovar sua sessão sem depender
                do login salvo de outro app. Google: todas as contas do
                Antigravity são detectadas automaticamente.
              </p>
            )}
            {connections.map((account, i) => (
              <div className="connection-row" key={account.accountKey}>
                <span>
                  {i + 1} · {account.accountLabel}
                </span>
                <button
                  className="connect-btn"
                  disabled={connecting}
                  onClick={() => void connect(account.accountKey)}
                >
                  Reconectar
                </button>
              </div>
            ))}
            {loginMessage && (
              <p className="source-note" role="status">
                {loginMessage}
              </p>
            )}
          </div>
          <p className="source-note">
            Arraste o título para mover a janela. Ao reabrir, ela volta para
            perto da bandeja.
          </p>
        </>
      )}
    </div>
  );
};
