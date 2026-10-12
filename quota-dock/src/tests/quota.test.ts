import { describe, it, expect } from 'vitest';
import { QuotaSnapshot, Availability } from '../contracts';

describe('Quota Contracts and Formatting', () => {
  it('correctly models an OpenAI Codex snapshot', () => {
    const snap: QuotaSnapshot = {
      schemaVersion: 1,
      providerId: 'openai-codex',
      origin: 'codex_app_server',
      accountKey: 'mock_key',
      accountLabel: 'c***@example.com',
      status: 'ok' as Availability,
      observedAt: '2026-10-11T18:00:00Z',
      attemptedAt: '2026-10-11T18:00:00Z',
      expiresAt: null,
      windows: [
        {
          id: 'codex/primary',
          label: 'Janela de 5h',
          usedPercent: 0,
          remainingPercent: 100,
          windowSeconds: 18000,
          resetAt: '2026-10-11T23:00:00Z',
          allowed: true,
          scope: 'codex',
        },
        {
          id: 'codex/secondary',
          label: 'Janela de 7d',
          usedPercent: 15,
          remainingPercent: 85,
          windowSeconds: 604800,
          resetAt: '2026-10-18T18:00:00Z',
          allowed: true,
          scope: 'codex',
        },
      ],
      models: [],
      warnings: [],
    };

    expect(snap.providerId).toBe('openai-codex');
    expect(snap.windows.length).toBe(2);
    expect(snap.windows[0].remainingPercent).toBe(100);
    expect(snap.windows[1].remainingPercent).toBe(85);
  });

  it('correctly models a Google Antigravity snapshot', () => {
    const snap: QuotaSnapshot = {
      schemaVersion: 1,
      providerId: 'google-antigravity',
      origin: 'antigravity_plugin_cache',
      accountKey: 'google_key',
      accountLabel: 'g***@gmail.com',
      status: 'ok' as Availability,
      observedAt: '2026-10-11T17:45:00Z',
      attemptedAt: '2026-10-11T18:00:00Z',
      expiresAt: null,
      windows: [],
      models: [
        {
          modelId: 'antigravity-gemini-flash',
          displayName: 'Gemini 3.8 Flash',
          remainingPercent: 42,
          resetAt: '2026-10-11T23:30:00Z',
          groupLabel: 'Gemini Flash',
          sourceExact: true,
        },
      ],
      warnings: [],
    };

    expect(snap.providerId).toBe('google-antigravity');
    expect(snap.models[0].remainingPercent).toBe(42);
    expect(snap.models[0].displayName).toBe('Gemini 3.8 Flash');
  });
});
