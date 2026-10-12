import React from "react";
import type { QuotaSnapshot, Availability } from "../contracts";
const labels: Record<Availability, string> = {
  ok: "Ao vivo",
  partial: "Parcial",
  stale: "Leitura anterior",
  disabled: "Desativada",
  not_configured: "Sem conta",
  not_installed: "Não instalado",
  unauthorized_scope: "Autenticação",
  forbidden: "Acesso bloqueado",
  rate_limited: "Limite atingido",
  offline: "Sem conexão",
  network_error: "Falha de conexão",
  source_changed: "Fonte alterada",
  no_data: "Sem leitura",
  unknown_error: "Falha na consulta",
};
const percent = (n: number | null) =>
  n !== null && Number.isFinite(n) && n >= 0 && n <= 100
    ? n.toLocaleString("pt-BR", { maximumFractionDigits: 1 }) + "%"
    : "Sem leitura";
const reset = (iso: string | null) => {
  if (!iso) return null;
  const d = new Date(iso);
  return Number.isFinite(d.getTime())
    ? "Renova " +
        d.toLocaleString("pt-BR", {
          day: "2-digit",
          month: "2-digit",
          hour: "2-digit",
          minute: "2-digit",
        })
    : null;
};
export const QuotaCard: React.FC<{
  snapshot: QuotaSnapshot;
  onOpenHelp?: (key: string) => void;
}> = ({ snapshot: s }) => {
  const google = s.providerId === "google-antigravity";
  const old =
    s.status === "stale" ||
    (!!s.expiresAt && new Date(s.expiresAt).getTime() < Date.now());
  const groups = new Map<string, typeof s.models>();
  for (const m of s.models) {
    const key = m.groupLabel || m.displayName;
    groups.set(key, [...(groups.get(key) || []), m]);
  }
  const rows = google
    ? [...groups].map(([label, models]) => {
        const m = models
          .filter(
            (x) =>
              x.remainingPercent !== null &&
              Number.isFinite(x.remainingPercent),
          )
          .sort((a, b) => a.remainingPercent! - b.remainingPercent!)[0];
        return {
          id: label,
          label,
          value: m?.remainingPercent ?? null,
          reset: m?.resetAt ?? null,
        };
      })
    : s.windows.map((w) => ({
        id: w.id,
        label: w.label,
        value: w.remainingPercent,
        reset: w.resetAt,
      }));
  const source =
    s.origin === "antigravity_plugin_cache"
      ? "Cache do OpenCode"
      : google
        ? "Google · consulta direta"
        : s.origin === "opencode_openai_usage"
          ? "OpenCode · consulta direta"
          : s.origin === "openai_quota_login"
            ? "OpenAI · conexão QuotaDock"
            : "OpenAI · login local do Codex";
  const date = s.observedAt ? new Date(s.observedAt) : null;
  return (
    <section
      className={"quota-card" + (old ? " stale" : "")}
      aria-label={(google ? "Google" : "OpenAI") + " " + s.accountLabel}
    >
      <div className="card-header">
        <span className="provider-name">
          {google ? "Google" : "OpenAI"}
          <span className="account-label">{s.accountLabel}</span>
        </span>
        <span
          className={"badge" + (old || s.status !== "ok" ? " attention" : "")}
        >
          {old ? "Leitura anterior" : labels[s.status]}
        </span>
      </div>
      <div className="quota-rows">
        {rows.map((r) => (
          <div
            className="quota-item"
            key={r.id}
            title={reset(r.reset) || undefined}
          >
            <div className="quota-item-header">
              <span className="quota-label">{r.label}</span>
              <span className="quota-value">{percent(r.value)}</span>
            </div>
            <div
              className="quota-progress-track"
              role={r.value === null ? undefined : "progressbar"}
              aria-label={
                r.label + ": saldo " + (old ? "anterior" : "restante")
              }
              aria-valuenow={r.value ?? undefined}
              aria-valuemin={0}
              aria-valuemax={100}
            >
              {r.value !== null &&
                Number.isFinite(r.value) &&
                r.value >= 0 &&
                r.value <= 100 && (
                  <div
                    className={
                      "quota-progress-fill" + (r.value <= 10 ? " critical" : "")
                    }
                    style={{ width: r.value + "%" }}
                  />
                )}
            </div>
            {r.reset && <span className="reset-time">{reset(r.reset)}</span>}
          </div>
        ))}
      </div>
      {rows.length === 0 && (
        <p className="source-note">
          {s.note || "A fonte não retornou limites nesta consulta."}
        </p>
      )}
      <details className="source-details">
        <summary>
          {source}
          {date && Number.isFinite(date.getTime())
            ? " · " +
              date.toLocaleTimeString("pt-BR", {
                hour: "2-digit",
                minute: "2-digit",
              })
            : ""}
        </summary>
        <p>
          Percentuais{" "}
          {old
            ? "da última leitura, que pode ter mudado"
            : "restantes, informados pelo provedor"}
          .{google ? " Resumo: menor saldo informado por família." : ""}
        </p>
        {rows.map(
          (r) =>
            r.reset && (
              <p key={r.id}>
                {r.label}: {reset(r.reset)}
              </p>
            ),
        )}
        {s.note && <p>{s.note}</p>}
        {s.warnings.map((w, i) => (
          <p key={i}>{w}</p>
        ))}
        {google &&
          s.models
            .filter((m) => m.sourceExact)
            .map((m) => (
              <div className="model-detail" key={m.modelId}>
                <span>{m.displayName}</span>
                <span>{percent(m.remainingPercent)}</span>
              </div>
            ))}
      </details>
    </section>
  );
};
