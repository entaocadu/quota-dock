import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { QuotaCard } from "../components/QuotaCard";
import type { QuotaSnapshot } from "../contracts";

const snapshot: QuotaSnapshot = {
  schemaVersion: 1,
  providerId: "google-antigravity",
  origin: "antigravity_plugin_cache",
  accountKey: "test",
  accountLabel: "a***@example.com",
  status: "stale",
  observedAt: "2026-10-01T00:00:00Z",
  attemptedAt: null,
  expiresAt: null,
  windows: [],
  models: [
    {
      modelId: "gemini-flash",
      displayName: "Gemini Flash",
      remainingPercent: 0.4,
      resetAt: null,
      groupLabel: null,
      sourceExact: true,
    },
  ],
  warnings: [],
};
describe("QuotaCard truthful presentation", () => {
  it("identifies old values as a previous reading, not current remaining quota", () => {
    const html = renderToStaticMarkup(<QuotaCard snapshot={snapshot} />);
    expect(html).toContain("Leitura anterior");
    expect(html).toContain("0,4%");
    expect(html).not.toContain("0% restante");
  });
  it("does not invent an HTTP 401 for a generic authorization error", () => {
    const html = renderToStaticMarkup(
      <QuotaCard
        snapshot={{ ...snapshot, status: "unauthorized_scope", models: [] }}
      />,
    );
    expect(html).not.toContain("401");
    expect(html).toContain("Autenticação");
  });
  it("does not render a filled bar for missing values", () => {
    const html = renderToStaticMarkup(
      <QuotaCard
        snapshot={{
          ...snapshot,
          status: "ok",
          models: [{ ...snapshot.models[0], remainingPercent: null }],
        }}
      />,
    );
    expect(html).not.toContain("quota-progress-fill");
    expect(html).toContain("Sem leitura");
  });
});
