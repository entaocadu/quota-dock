import React from "react";
import { RefreshCw, Settings2, X, ArrowLeft } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
export const Header: React.FC<{
  refreshing: boolean;
  onRefresh: () => void;
  settingsOpen: boolean;
  onSettings: () => void;
}> = ({ refreshing, onRefresh, settingsOpen, onSettings }) => (
  <header className="header">
    <div
      className="header-title-group"
      onMouseDown={(e) => {
        if (e.button === 0) {
          e.preventDefault();
          void invoke("start_dragging");
        }
      }}
    >
      <span className="logo-mark" aria-hidden="true" />
      <h1>QuotaDock</h1>
      <span className="header-subtitle">OpenCode</span>
    </div>
    <div className="header-actions">
      <button
        className="icon-btn"
        disabled={refreshing}
        onClick={onRefresh}
        title="Atualizar quotas"
        aria-label="Atualizar quotas"
      >
        <RefreshCw size={14} className={refreshing ? "spinning" : ""} />
      </button>
      <button
        className="icon-btn"
        onClick={onSettings}
        aria-label={settingsOpen ? "Voltar às quotas" : "Ajustes"}
        title={settingsOpen ? "Voltar às quotas" : "Ajustes"}
      >
        {settingsOpen ? <ArrowLeft size={14} /> : <Settings2 size={14} />}
      </button>
      <button
        className="icon-btn"
        onClick={() => void invoke("hide_window")}
        aria-label="Ocultar na bandeja"
        title="Ocultar"
      >
        <X size={14} />
      </button>
    </div>
  </header>
);
