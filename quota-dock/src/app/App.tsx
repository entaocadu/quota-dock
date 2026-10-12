import React, { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Header } from "../components/Header";
import { QuotaCard } from "../components/QuotaCard";
import { SettingsView } from "../components/SettingsView";
import type { QuotaSnapshot } from "../contracts";
export const App: React.FC = () => {
  const [settingsOpen, setSettingsOpen] = useState(false),
    [snapshots, setSnapshots] = useState<QuotaSnapshot[]>([]),
    [refreshing, setRefreshing] = useState(false),
    [error, setError] = useState<string | null>(null),
    [lastAttempt, setLastAttempt] = useState<string | null>(null);
  const loading = useRef(false);
  const content = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const element = content.current;
    if (!element) return;
    let frame = 0;
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        void invoke("resize_flyout", {
          height: Math.ceil(element.getBoundingClientRect().height + 78),
        }).catch(() => {});
      });
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
      cancelAnimationFrame(frame);
    };
  }, []);
  const loadOverview = useCallback(async (force = false) => {
    if (loading.current) return;
    loading.current = true;
    setRefreshing(true);
    try {
      setSnapshots(
        await invoke<QuotaSnapshot[]>(
          force ? "request_refresh" : "get_overview",
        ),
      );
      setError(null);
    } catch {
      setError("Falha ao atualizar. A leitura anterior pode ter mudado.");
      setSnapshots((prev) => prev.map((s) => ({ ...s, status: "stale" })));
    } finally {
      setLastAttempt(
        new Date().toLocaleTimeString("pt-BR", {
          hour: "2-digit",
          minute: "2-digit",
        }),
      );
      setRefreshing(false);
      loading.current = false;
    }
  }, []);
  useEffect(() => {
    void loadOverview();
    const escape = (e: KeyboardEvent) => {
      if (e.key === "Escape") void invoke("hide_window");
    };
    window.addEventListener("keydown", escape);
    const listeners = [
      listen<QuotaSnapshot[]>("quotas-updated", (e) => {
        setSnapshots(e.payload);
        setLastAttempt(
          new Date().toLocaleTimeString("pt-BR", {
            hour: "2-digit",
            minute: "2-digit",
          }),
        );
      }),
      listen("trigger-refresh", () => void loadOverview(true)),
      listen("flyout-opened", () => {
        setSettingsOpen(false);
        void loadOverview();
      }),
    ];
    return () => {
      window.removeEventListener("keydown", escape);
      listeners.forEach((p) => {
        void p.then((f) => f());
      });
    };
  }, [loadOverview]);
  return (
    <div className="window-container">
      <Header
        refreshing={refreshing}
        onRefresh={() => void loadOverview(true)}
        settingsOpen={settingsOpen}
        onSettings={() => setSettingsOpen((v) => !v)}
      />
      <main className="content-body" aria-busy={refreshing}>
        <div ref={content} className="content-inner">
          {settingsOpen ? (
            <SettingsView />
          ) : (
            <>
              {error && (
                <p role="alert" className="error-note">
                  {error}
                </p>
              )}
              {snapshots.length === 0 && (
                <div className="empty-state">
                  {refreshing
                    ? "Consultando suas contas…"
                    : "Nenhuma leitura recebida."}
                </div>
              )}
              {snapshots.map((s) => (
                <QuotaCard
                  key={s.providerId + "-" + s.accountKey}
                  snapshot={s}
                />
              ))}
            </>
          )}
        </div>
      </main>
      <footer className="footer-bar">
        <span>
          {refreshing
            ? "Consultando…"
            : lastAttempt
              ? "Consulta " + lastAttempt
              : "Aguardando consulta"}
        </span>
        <span>Clique fora para ocultar</span>
      </footer>
    </div>
  );
};
